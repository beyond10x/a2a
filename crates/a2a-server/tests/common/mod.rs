//! A server on a real socket and a raw HTTP/1.1 client, shared by the integration tests.

#![allow(dead_code)]

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

use a2a::json::Value;
use a2a_server::{Access, Agent, EchoAgent, Limits, Settings};

/// Starts a server on port 0 with these settings and the agent `make` builds on its thread.
pub fn start(settings: Settings, make: impl FnOnce() -> Box<dyn Agent> + Send + 'static) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = listener.local_addr().expect("bound address");
    std::thread::spawn(move || {
        let _ = a2a_server::serve(listener, settings, make());
    });
    Server { address }
}

/// Starts the echo agent behind these settings on port 0, within `limits` and for `access`.
pub fn start_with(settings: Settings, limits: Limits, access: Access) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = listener.local_addr().expect("bound address");
    let accepted = settings.card.default_input_modes.clone();
    std::thread::spawn(move || {
        let agent = Box::new(EchoAgent::new(accepted));
        let _ = a2a_server::serve_with(listener, settings, agent, limits, access);
    });
    Server { address }
}

/// The built-in card with its capabilities changed by `change`.
pub fn card(change: impl FnOnce(&mut a2a::discovery::AgentCard)) -> Settings {
    let mut card = a2a_server::default_card();
    change(&mut card);
    Settings {
        card,
        extended_card: None,
    }
}

/// The echo agent behind the built-in card.
pub fn echo() -> Server {
    echo_with(card(|_| {}))
}

/// The echo agent behind these settings.
pub fn echo_with(settings: Settings) -> Server {
    let accepted = settings.card.default_input_modes.clone();
    start(settings, move || Box::new(EchoAgent::new(accepted)))
}

/// A running server.
pub struct Server {
    pub address: SocketAddr,
}

/// An answer.
#[derive(Debug)]
pub struct Reply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Reply {
    pub fn json(&self) -> Value {
        a2a::json::parse(&self.body)
            .unwrap_or_else(|error| panic!("body is not JSON ({error}): {}", self.body))
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// The `google.rpc.ErrorInfo` reason of an A2A error answer.
    pub fn reason(&self) -> String {
        let value = self.json();
        let details = value
            .member("error")
            .and_then(|error| error.member("details"))
            .unwrap_or_else(|| panic!("not a google.rpc.Status: {}", self.body));
        let Value::Array(details) = details else {
            panic!("details is not an array: {}", self.body);
        };
        details
            .iter()
            .find_map(|detail| match detail.member("reason") {
                Some(Value::Text(reason)) => Some(reason.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// The `error.status` of a google.rpc.Status answer.
    pub fn rpc_status(&self) -> String {
        text(&self.json(), &["error", "status"])
    }

    /// The `data:` payloads of a server-sent event stream.
    pub fn frames(&self) -> Vec<Value> {
        frames(&self.body)
    }
}

impl Server {
    /// A request with `A2A-Version: 1.0`.
    pub fn call(&self, method: &str, path: &str, body: Option<&str>) -> Reply {
        self.call_with(method, path, body, &[("A2A-Version", "1.0")])
    }

    /// A request with exactly these extra headers.
    pub fn call_with(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        headers: &[(&str, &str)],
    ) -> Reply {
        let mut stream = self.send(method, path, body, headers);
        let mut raw = Vec::new();
        let _ = stream.read_to_end(&mut raw);
        parse(&String::from_utf8_lossy(&raw))
    }

    /// Writes a request and hands back the open connection.
    pub fn send(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        headers: &[(&str, &str)],
    ) -> TcpStream {
        let mut stream = TcpStream::connect(self.address).expect("connect");
        stream
            .set_read_timeout(Some(Duration::from_secs(20)))
            .expect("read timeout");
        let body = body.unwrap_or("");
        let mut head = format!("{method} {path} HTTP/1.1\r\nHost: {}\r\n", self.address);
        for (name, value) in headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        if !body.is_empty() {
            head.push_str("Content-Type: application/a2a+json\r\n");
        }
        head.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
        stream.write_all(head.as_bytes()).expect("write head");
        stream.write_all(body.as_bytes()).expect("write body");
        stream.flush().expect("flush");
        stream
    }

    /// Sends `text` as a new user message and answers the `SendMessageResponse`.
    pub fn send_text(&self, text: &str) -> Value {
        let reply = self.call("POST", "/message:send", Some(&message(text, None, None)));
        assert_eq!(reply.status, 200, "{}", reply.body);
        reply.json()
    }

    /// Creates a completed task and answers its id.
    pub fn completed_task(&self) -> String {
        let answer = self.send_text("hello");
        assert_eq!(
            text(&answer, &["task", "status", "state"]),
            "TASK_STATE_COMPLETED"
        );
        text(&answer, &["task", "id"])
    }

    /// Creates a task waiting for input and answers its id and context id.
    pub fn waiting_task(&self) -> (String, String) {
        let answer = self.send_text("");
        assert_eq!(
            text(&answer, &["task", "status", "state"]),
            "TASK_STATE_INPUT_REQUIRED"
        );
        (
            text(&answer, &["task", "id"]),
            text(&answer, &["task", "contextId"]),
        )
    }
}

/// A `SendMessageRequest` body with one text part.
pub fn message(text: &str, task_id: Option<&str>, context_id: Option<&str>) -> String {
    let mut message = format!(
        "{{\"messageId\":\"{}\",\"role\":\"ROLE_USER\",\"parts\":[{{\"text\":{}}}]",
        uuid::Uuid::new_v4(),
        quoted(text)
    );
    if let Some(task_id) = task_id {
        message.push_str(&format!(",\"taskId\":{}", quoted(task_id)));
    }
    if let Some(context_id) = context_id {
        message.push_str(&format!(",\"contextId\":{}", quoted(context_id)));
    }
    message.push('}');
    format!("{{\"message\":{message}}}")
}

pub fn quoted(text: &str) -> String {
    let mut out = String::new();
    a2a::json::push_text(&mut out, text);
    out
}

/// The text at `path` under `value`, or an empty string.
pub fn text(value: &Value, path: &[&str]) -> String {
    match at(value, path) {
        Some(Value::Text(text)) => text.clone(),
        _ => String::new(),
    }
}

/// The value at `path` under `value`.
pub fn at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut here = value;
    for step in path {
        here = match (here, step.parse::<usize>()) {
            (Value::Array(items), Ok(index)) => items.get(index)?,
            _ => here.member(step)?,
        };
    }
    Some(here)
}

/// The length of the array at `path`, 0 when absent.
pub fn count(value: &Value, path: &[&str]) -> usize {
    match at(value, path) {
        Some(Value::Array(items)) => items.len(),
        _ => 0,
    }
}

/// The `data:` payloads of a server-sent event stream body.
pub fn frames(body: &str) -> Vec<Value> {
    body.split("\n\n")
        .filter(|event| !event.trim().is_empty())
        .map(|event| {
            let data = event
                .strip_prefix("data: ")
                .unwrap_or_else(|| panic!("an event is not one `data:` line: {event:?}"));
            a2a::json::parse(data).unwrap_or_else(|error| panic!("{error}: {data}"))
        })
        .collect()
}

/// Reads one server-sent event from an open stream whose head is already read.
pub fn next_frame(reader: &mut BufReader<TcpStream>) -> Option<Value> {
    let mut event = String::new();
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line).ok()?;
        if read == 0 {
            return None;
        }
        if line == "\n" || line == "\r\n" {
            if event.is_empty() {
                continue;
            }
            break;
        }
        event.push_str(&line);
    }
    let data = event.trim_end().strip_prefix("data: ")?;
    a2a::json::parse(data).ok()
}

/// Reads a response head from an open stream: status and headers.
pub fn read_head(reader: &mut BufReader<TcpStream>) -> (u16, Vec<(String, String)>) {
    let mut status_line = String::new();
    reader.read_line(&mut status_line).expect("status line");
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("header line");
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_owned(), value.trim().to_owned()));
        }
    }
    (status, headers)
}

fn parse(raw: &str) -> Reply {
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw, ""));
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
        .collect();
    Reply {
        status,
        headers,
        body: body.to_owned(),
    }
}
