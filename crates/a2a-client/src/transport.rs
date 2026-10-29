//! HTTP/1.1 over `std::net::TcpStream`: the base URL, one request per connection, and the answer's
//! head and body (by `Content-Length`, chunked, or to the end of the connection).

use std::io::{self, BufRead as _, BufReader, Read, Take, Write as _};
use std::net::{TcpStream, ToSocketAddrs as _};
use std::time::Duration;

use crate::error::Error;

/// The most bytes of an answer's head.
const MAX_HEAD: u64 = 64 * 1024;
/// The most header lines of an answer.
const MAX_HEADERS: usize = 100;
/// The most bytes of an answer's body, and of one server-sent event.
pub(crate) const MAX_BODY: u64 = 16 * 1024 * 1024;
/// The most bytes of a chunk-size line.
const MAX_CHUNK_LINE: u64 = 1024;

/// Where the agent is: an `http://` base URL taken apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Origin {
    /// `host[:port]` as given, for the `Host` header.
    pub(crate) authority: String,
    /// `host:port`, to connect to.
    pub(crate) address: String,
    /// The base URL's path without a trailing `/`: empty, or `/a2a/v1`.
    pub(crate) prefix: String,
}

/// Reads an `http://host[:port][/path]` base URL.
pub(crate) fn origin(url: &str) -> Result<Origin, Error> {
    let Some((scheme, rest)) = url.split_once("://") else {
        return Err(Error::InvalidRequest(format!(
            "`{url}` is not an absolute URL"
        )));
    };
    if !scheme.eq_ignore_ascii_case("http") {
        return Err(Error::UnsupportedScheme(scheme.to_owned()));
    }
    if let Some(byte) = rest.bytes().find(|byte| !byte.is_ascii_graphic()) {
        return Err(Error::InvalidRequest(format!(
            "`{url}` holds the byte {byte:#04x}"
        )));
    }
    if rest.contains(['?', '#']) {
        return Err(Error::InvalidRequest(format!(
            "`{url}` has a query or fragment; a base URL has neither"
        )));
    }
    let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    if authority.is_empty() || authority.contains('@') {
        return Err(Error::InvalidRequest(format!(
            "`{url}` names no host, or user information"
        )));
    }
    let (host, port) = match authority.rfind(':') {
        Some(at) if !authority[at..].contains(']') => (&authority[..at], &authority[at + 1..]),
        _ => (authority, ""),
    };
    let port = if port.is_empty() {
        80
    } else {
        port.parse::<u16>().map_err(|_| {
            Error::InvalidRequest(format!("`{url}` has a port that is not a number"))
        })?
    };
    if host.is_empty() {
        return Err(Error::InvalidRequest(format!("`{url}` names no host")));
    }
    Ok(Origin {
        authority: authority.to_owned(),
        address: format!("{host}:{port}"),
        prefix: path.trim_end_matches('/').to_owned(),
    })
}

/// `text` percent-encoded as one path segment: every byte but the unreserved ones (RFC 3986
/// section 2.3).
pub(crate) fn segment(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// A request to write: method, target (path and query, encoded), headers and body.
pub(crate) struct Call<'a> {
    pub(crate) method: &'static str,
    pub(crate) target: String,
    pub(crate) headers: Vec<(&'static str, &'a str)>,
    pub(crate) body: Option<String>,
}

/// An answer whose head is read: its status, headers and the body still on the connection.
pub(crate) struct Answer {
    pub(crate) status: u16,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Body,
}

impl Answer {
    /// The value of the header `name`, any case.
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// The whole body as text.
    pub(crate) fn text(self) -> Result<String, Error> {
        let mut bytes = Vec::new();
        let mut limited = self.body.take(MAX_BODY + 1);
        limited.read_to_end(&mut bytes).map_err(read_error)?;
        if bytes.len() as u64 > MAX_BODY {
            return Err(Error::Protocol(format!(
                "the body is over {MAX_BODY} bytes"
            )));
        }
        limited.into_inner().finish()?;
        String::from_utf8(bytes).map_err(|_| Error::Protocol("the body is not UTF-8".to_owned()))
    }
}

/// An error reading an answer: the bytes are not the binding's, or the connection failed.
pub(crate) fn read_error(error: io::Error) -> Error {
    match error.kind() {
        io::ErrorKind::InvalidData | io::ErrorKind::UnexpectedEof => {
            Error::Protocol(error.to_string())
        }
        _ => Error::Transport(error),
    }
}

/// Sends `call` to `origin` on a new connection and reads the answer's head.
pub(crate) fn exchange(
    origin: &Origin,
    timeout: Option<Duration>,
    call: &Call<'_>,
) -> Result<Answer, Error> {
    for (name, value) in &call.headers {
        if value
            .bytes()
            .any(|byte| byte.is_ascii_control() && byte != b'\t')
        {
            return Err(Error::InvalidRequest(format!(
                "the {name} header holds a control character"
            )));
        }
    }
    let mut head = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\n",
        call.method, call.target, origin.authority
    );
    for (name, value) in &call.headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    if let Some(body) = &call.body {
        head.push_str("Content-Type: application/a2a+json\r\n");
        head.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    head.push_str("Connection: close\r\n\r\n");

    let mut stream = connect(&origin.address, timeout)?;
    stream.set_read_timeout(timeout).map_err(Error::Transport)?;
    stream
        .set_write_timeout(timeout)
        .map_err(Error::Transport)?;
    stream
        .write_all(head.as_bytes())
        .map_err(Error::Transport)?;
    if let Some(body) = &call.body {
        stream
            .write_all(body.as_bytes())
            .map_err(Error::Transport)?;
    }
    stream.flush().map_err(Error::Transport)?;
    read_head(BufReader::new(stream))
}

fn connect(address: &str, timeout: Option<Duration>) -> Result<TcpStream, Error> {
    let addresses = address.to_socket_addrs().map_err(Error::Transport)?;
    let mut last = None;
    for socket in addresses {
        let connected = match timeout {
            Some(timeout) => TcpStream::connect_timeout(&socket, timeout),
            None => TcpStream::connect(socket),
        };
        match connected {
            Ok(stream) => return Ok(stream),
            Err(error) => last = Some(error),
        }
    }
    Err(Error::Transport(last.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("`{address}` resolves to no address"),
        )
    })))
}

/// One line of at most `limit` bytes, without its line break; `None` at the end of the stream.
fn line(reader: &mut BufReader<TcpStream>, limit: u64) -> Result<Option<String>, Error> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(limit + 1)
        .read_until(b'\n', &mut bytes)
        .map_err(read_error)?;
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.pop() != Some(b'\n') {
        return Err(Error::Protocol(format!(
            "a line of the answer is over {limit} bytes or not ended"
        )));
    }
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| Error::Protocol("the answer's head is not UTF-8".to_owned()))
}

fn read_head(mut reader: BufReader<TcpStream>) -> Result<Answer, Error> {
    let Some(status_line) = line(&mut reader, MAX_HEAD)? else {
        return Err(Error::Protocol(
            "the server closed the connection without an answer".to_owned(),
        ));
    };
    let mut words = status_line.splitn(3, ' ');
    let version = words.next().unwrap_or_default();
    let code = words.next().unwrap_or_default();
    let status = match code.parse::<u16>() {
        Ok(status) if version.starts_with("HTTP/1.") && code.len() == 3 && status >= 100 => status,
        _ => {
            return Err(Error::Protocol(format!(
                "not an HTTP/1.x status line: {}",
                crate::error::excerpt(&status_line)
            )));
        }
    };
    let mut headers = Vec::new();
    let mut read = status_line.len() as u64;
    loop {
        let Some(text) = line(&mut reader, MAX_HEAD)? else {
            return Err(Error::Protocol(
                "the answer's head ended before its blank line".to_owned(),
            ));
        };
        read += text.len() as u64 + 2;
        if read > MAX_HEAD || headers.len() > MAX_HEADERS {
            return Err(Error::Protocol(format!(
                "the answer's head is over {MAX_HEAD} bytes or {MAX_HEADERS} lines"
            )));
        }
        if text.is_empty() {
            break;
        }
        let Some((name, value)) = text.split_once(':') else {
            return Err(Error::Protocol(format!(
                "not a header line: {}",
                crate::error::excerpt(&text)
            )));
        };
        headers.push((name.trim().to_owned(), value.trim().to_owned()));
    }
    let find = |name: &str| {
        headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    };
    let chunked = find("Transfer-Encoding").is_some_and(|value| {
        value
            .rsplit(',')
            .next()
            .is_some_and(|last| last.trim().eq_ignore_ascii_case("chunked"))
    });
    let body = if status == 204 || status == 304 || (100..200).contains(&status) {
        Body::Length(reader.take(0), 0)
    } else if chunked {
        Body::Chunked(Chunked {
            reader,
            left: 0,
            done: false,
        })
    } else if let Some(length) = find("Content-Length") {
        let length = length
            .parse::<u64>()
            .map_err(|_| Error::Protocol(format!("Content-Length is not a number: {length}")))?;
        Body::Length(reader.take(length), length)
    } else {
        Body::Close(reader)
    };
    Ok(Answer {
        status,
        headers,
        body,
    })
}

/// An answer's body as it arrives.
pub(crate) enum Body {
    /// `Content-Length` bytes; the second is how many.
    Length(Take<BufReader<TcpStream>>, u64),
    /// Everything until the server closes the connection.
    Close(BufReader<TcpStream>),
    /// `Transfer-Encoding: chunked` (RFC 9112 section 7.1).
    Chunked(Chunked),
}

impl Body {
    /// Fails where the body ended before all the bytes its head announced.
    pub(crate) fn finish(&self) -> Result<(), Error> {
        match self {
            Self::Length(take, length) if take.limit() > 0 => Err(Error::Protocol(format!(
                "the body ended after {} of its {length} bytes",
                length - take.limit()
            ))),
            _ => Ok(()),
        }
    }
}

impl Read for Body {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Length(take, _) => take.read(buffer),
            Self::Close(reader) => reader.read(buffer),
            Self::Chunked(chunked) => chunked.read(buffer),
        }
    }
}

/// A chunked body, decoded as it is read.
pub(crate) struct Chunked {
    reader: BufReader<TcpStream>,
    /// Bytes left in the current chunk.
    left: u64,
    done: bool,
}

impl Chunked {
    fn invalid(detail: &str) -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("chunked body: {detail}"),
        )
    }

    fn line(&mut self) -> io::Result<String> {
        let mut bytes = Vec::new();
        (&mut self.reader)
            .take(MAX_CHUNK_LINE + 1)
            .read_until(b'\n', &mut bytes)?;
        if bytes.pop() != Some(b'\n') {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "chunked body: a line ended early or is too long",
            ));
        }
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
        String::from_utf8(bytes).map_err(|_| Self::invalid("a line is not UTF-8"))
    }
}

impl Read for Chunked {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.done || buffer.is_empty() {
            return Ok(0);
        }
        if self.left == 0 {
            let size_line = self.line()?;
            let size = size_line.split(';').next().unwrap_or_default().trim();
            let size = u64::from_str_radix(size, 16)
                .ok()
                .filter(|_| !size.is_empty() && size.bytes().all(|byte| byte.is_ascii_hexdigit()))
                .ok_or_else(|| Self::invalid(&format!("not a chunk size: `{size_line}`")))?;
            if size == 0 {
                while !self.line()?.is_empty() {}
                self.done = true;
                return Ok(0);
            }
            self.left = size;
        }
        let wanted = usize::try_from(self.left)
            .unwrap_or(usize::MAX)
            .min(buffer.len());
        let read = self.reader.read(&mut buffer[..wanted])?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "chunked body: the connection closed inside a chunk",
            ));
        }
        self.left -= read as u64;
        if self.left == 0 && !self.line()?.is_empty() {
            return Err(Self::invalid("a chunk is longer than its size"));
        }
        Ok(read)
    }
}
