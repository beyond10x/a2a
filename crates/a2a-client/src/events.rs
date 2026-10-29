//! A server-sent event stream (A2A v1.0.1 section 11.7) read as typed stream responses.

use std::io::{BufRead as _, BufReader};

use a2a::json::{DecodeError, Value};

use crate::error::{Error, excerpt};
use crate::transport::{Body, MAX_BODY, read_error};

/// The stream responses of one `SendStreamingMessage` or `SubscribeToTask`, in the order the
/// server sent them. Each item is one event's `data`, decoded; the iterator ends when the server
/// closes the stream after a complete event. A stream that ends inside an event, or short of the
/// length its head announced, or an event that is not a stream response, yields one
/// `Error::Protocol` and then ends.
pub struct Events<T> {
    reader: Option<BufReader<Body>>,
    /// The last line ended in a CR, so a LF that follows belongs to it.
    after_cr: bool,
    decode: fn(&Value, &str) -> Result<T, DecodeError>,
}

impl<T> Events<T> {
    pub(crate) fn new(body: Body, decode: fn(&Value, &str) -> Result<T, DecodeError>) -> Self {
        Self {
            reader: Some(BufReader::new(body)),
            after_cr: false,
            decode,
        }
    }

    /// One line without its break, which is CRLF, LF or a lone CR (WHATWG HTML, server-sent
    /// events), and whether it was ended; at most `limit` bytes.
    fn line(
        reader: &mut BufReader<Body>,
        after_cr: &mut bool,
        limit: u64,
    ) -> Result<(Vec<u8>, bool), Error> {
        let mut bytes = Vec::new();
        loop {
            let buffer = reader.fill_buf().map_err(read_error)?;
            if buffer.is_empty() {
                return Ok((bytes, false));
            }
            let skip = usize::from(std::mem::take(after_cr) && buffer[0] == b'\n');
            let rest = &buffer[skip..];
            let (taken, ended) = match rest.iter().position(|&byte| byte == b'\n' || byte == b'\r')
            {
                Some(at) => {
                    bytes.extend_from_slice(&rest[..at]);
                    *after_cr = rest[at] == b'\r';
                    (skip + at + 1, true)
                }
                None => {
                    bytes.extend_from_slice(rest);
                    (buffer.len(), false)
                }
            };
            reader.consume(taken);
            if bytes.len() as u64 > limit {
                return Err(Error::Protocol(format!(
                    "an event is over {MAX_BODY} bytes"
                )));
            }
            if ended {
                return Ok((bytes, true));
            }
        }
    }

    /// The `data` of the next complete event; `None` where the stream closed between events.
    fn next_data(&mut self) -> Result<Option<String>, Error> {
        let Some(reader) = self.reader.as_mut() else {
            return Ok(None);
        };
        let mut data: Option<String> = None;
        loop {
            let held = data.as_ref().map_or(0, String::len) as u64;
            let (bytes, ended) =
                Self::line(reader, &mut self.after_cr, MAX_BODY.saturating_sub(held))?;
            if !ended {
                if bytes.is_empty() && data.is_none() {
                    reader.get_ref().finish()?;
                    return Ok(None);
                }
                return Err(Error::Protocol(
                    "the stream ended inside an event: no blank line after its data".to_owned(),
                ));
            }
            let text = String::from_utf8(bytes)
                .map_err(|_| Error::Protocol("an event is not UTF-8".to_owned()))?;
            if text.is_empty() {
                if data.is_some() {
                    return Ok(data);
                }
                continue;
            }
            if text.starts_with(':') {
                continue;
            }
            let (field, value) = match text.split_once(':') {
                Some((field, value)) => (field, value.strip_prefix(' ').unwrap_or(value)),
                None => (text.as_str(), ""),
            };
            if field == "data" {
                match &mut data {
                    Some(held) => {
                        held.push('\n');
                        held.push_str(value);
                    }
                    None => data = Some(value.to_owned()),
                }
            }
        }
    }
}

impl<T> Iterator for Events<T> {
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.as_ref()?;
        let decoded = match self.next_data() {
            Ok(None) => None,
            Ok(Some(data)) => Some(
                a2a::json::parse(&data)
                    .map_err(|error| {
                        Error::Protocol(format!(
                            "an event's data is not JSON ({error}): {}",
                            excerpt(&data)
                        ))
                    })
                    .and_then(|value| {
                        (self.decode)(&value, "event").map_err(|error| {
                            Error::Protocol(format!("an event is not a stream response: {error}"))
                        })
                    }),
            ),
            Err(error) => Some(Err(error)),
        };
        if !matches!(decoded, Some(Ok(_))) {
            self.reader = None;
        }
        decoded
    }
}

impl<T> std::fmt::Debug for Events<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Events")
            .field("open", &self.reader.is_some())
            .finish()
    }
}
