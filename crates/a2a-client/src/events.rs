//! A server-sent event stream (A2A v1.0.1 section 11.7) read as typed stream responses.

use std::io::{BufRead as _, BufReader, Read as _};

use a2a::json::{DecodeError, Value};

use crate::error::{Error, excerpt};
use crate::transport::{Body, MAX_BODY, read_error};

/// The stream responses of one `SendStreamingMessage` or `SubscribeToTask`, in the order the
/// server sent them. Each item is one event's `data`, decoded; the iterator ends when the server
/// closes the stream after a complete event. A stream that ends inside an event, or an event
/// that is not a stream response, yields one `Error::Protocol` and then ends.
pub struct Events<T> {
    reader: Option<BufReader<Body>>,
    decode: fn(&Value, &str) -> Result<T, DecodeError>,
}

impl<T> Events<T> {
    pub(crate) fn new(body: Body, decode: fn(&Value, &str) -> Result<T, DecodeError>) -> Self {
        Self {
            reader: Some(BufReader::new(body)),
            decode,
        }
    }

    /// The `data` of the next complete event; `None` where the stream closed between events.
    fn next_data(reader: &mut BufReader<Body>) -> Result<Option<String>, Error> {
        let mut data: Option<String> = None;
        loop {
            let mut bytes = Vec::new();
            let held = data.as_ref().map_or(0, String::len) as u64;
            reader
                .by_ref()
                .take((MAX_BODY + 1).saturating_sub(held))
                .read_until(b'\n', &mut bytes)
                .map_err(read_error)?;
            if bytes.is_empty() {
                return match data {
                    None => Ok(None),
                    Some(_) => Err(Error::Protocol(
                        "the stream ended inside an event: no blank line after its data".to_owned(),
                    )),
                };
            }
            if bytes.pop() != Some(b'\n') {
                return Err(Error::Protocol(
                    "the stream ended inside an event: a line is not ended, or the event is too long"
                        .to_owned(),
                ));
            }
            if bytes.last() == Some(&b'\r') {
                bytes.pop();
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
        let reader = self.reader.as_mut()?;
        let decoded = match Self::next_data(reader) {
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
