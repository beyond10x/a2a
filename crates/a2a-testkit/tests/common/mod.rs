//! An `a2a-server` echo agent on port 0, the suite synthesized with the `ess` on `PATH`, the
//! `a2a-testkit` binary, a socket that records what it is sent, and a proxy that plants a
//! defect, shared by the integration tests.

#![allow(dead_code)]

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

use a2a::json::{self, Value};
use a2a::tasks::{Message, Part, Role, SendMessage, SendStreamingMessage};
use a2a_client::Client;
use a2a_conformance::Results;
use a2a_server::{EchoAgent, Settings};

/// The repository's root.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// An endpoint on a local port.
pub struct Endpoint {
    pub address: SocketAddr,
}

impl Endpoint {
    /// Its base URL.
    pub fn url(&self) -> String {
        format!("http://{}", self.address)
    }

    /// A client of it.
    pub fn client(&self) -> Client {
        Client::new(&self.url()).expect("an http:// base URL")
    }
}

/// `a2a-server` with the echo agent behind its built-in card, on port 0.
pub fn echo() -> Endpoint {
    let settings = Settings {
        card: a2a_server::default_card(),
        extended_card: None,
    };
    echo_with(settings)
}

/// `a2a-server` with the echo agent behind `settings`, on port 0.
pub fn echo_with(settings: Settings) -> Endpoint {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = listener.local_addr().expect("bound address");
    let accepted = settings.card.default_input_modes.clone();
    std::thread::spawn(move || {
        let _ = a2a_server::serve(listener, settings, Box::new(EchoAgent::new(accepted)));
    });
    Endpoint { address }
}

/// A directory of its own under this test target's temporary directory, emptied first.
pub fn scratch(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("the scratch directory");
    directory
}

/// Synthesizes the suite of `ess/` with the `ess` on `PATH`, as `task conform` does, into
/// `directory`, and answers its path.
pub fn synthesize(directory: &Path) -> PathBuf {
    let suite = directory.join("suite.json");
    let synthesized = Command::new("ess")
        .current_dir(root())
        .args([
            "verify",
            "conform",
            "synthesize",
            "--path",
            "ess",
            "--scenarios",
            "ess",
            "--out",
        ])
        .arg(&suite)
        .output()
        .expect("`ess` runs: the pinned ess must be on PATH");
    assert!(
        synthesized.status.success(),
        "ess verify conform synthesize: {}",
        String::from_utf8_lossy(&synthesized.stderr)
    );
    suite
}

/// Runs `a2a-testkit` with `arguments`.
pub fn testkit(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_a2a-testkit"))
        .args(arguments)
        .output()
        .expect("the a2a-testkit binary runs")
}

/// Runs `a2a-testkit run` against `url` with `suite`, writing the results to `results`.
pub fn run(url: &str, suite: &Path, results: &Path) -> Output {
    let suite = suite.to_str().expect("a UTF-8 path");
    let results = results.to_str().expect("a UTF-8 path");
    testkit(&["run", "--url", url, "--suite", suite, "--results", results])
}

/// The results document at `path`.
pub fn results(path: &Path) -> Results {
    Results::parse(&std::fs::read_to_string(path).expect("the results were written"))
        .expect("an ess-conformance-results/1 document")
}

/// What the run printed, for an assertion message.
pub fn printed(output: &Output) -> String {
    format!(
        "exit {:?}\n--- stdout\n{}\n--- stderr\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// A part holding `text`.
pub fn text_part(text: &str) -> Part {
    Part {
        text: Some(text.to_owned()),
        raw: None,
        url: None,
        data: None,
        metadata: None,
        filename: None,
        media_type: None,
    }
}

/// A user message holding `text`.
pub fn message(text: &str) -> Message {
    Message {
        message_id: format!("m-{text}"),
        context_id: None,
        task_id: None,
        role: Role::ROLEUSER,
        parts: vec![text_part(text)],
        metadata: None,
        extensions: None,
        reference_task_ids: None,
    }
}

/// `SendMessage` of `message`, without a tenant.
pub fn send(message: Message) -> SendMessage {
    SendMessage {
        tenant: None,
        message,
        configuration: None,
        metadata: None,
    }
}

/// `SendStreamingMessage` of `message`, without a tenant.
pub fn stream(message: Message) -> SendStreamingMessage {
    SendStreamingMessage {
        tenant: None,
        message,
        configuration: None,
        metadata: None,
    }
}

/// A JSON value.
pub fn value(text: &str) -> Value {
    json::parse(text).expect("JSON")
}

/// One request a socket was sent: its method and target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    pub method: String,
    pub target: String,
}

/// A socket that answers every request with `answer` and records each request line.
pub fn recording(answer: String) -> (Endpoint, Arc<Mutex<Vec<Seen>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = listener.local_addr().expect("bound address");
    let seen = Arc::new(Mutex::new(Vec::new()));
    let kept = Arc::clone(&seen);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let Some((head, _)) = read_request(&mut stream) else {
                continue;
            };
            let mut words = head.lines().next().unwrap_or_default().split(' ');
            let method = words.next().unwrap_or_default().to_owned();
            let target = words.next().unwrap_or_default().to_owned();
            kept.lock().expect("the list").push(Seen { method, target });
            let _ = stream.write_all(answer.as_bytes());
        }
    });
    (Endpoint { address }, seen)
}

/// An HTTP/1.1 answer that closes the connection.
pub fn answer(status: u16, body: &str) -> String {
    format!(
        "HTTP/1.1 {status} Answer\r\nContent-Type: application/a2a+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// A `google.rpc.Status` error answer with the A2A reason `reason`.
pub fn error_answer(status: u16, code: &str, reason: &str) -> String {
    answer(
        status,
        &format!(
            r#"{{"error":{{"code":{status},"status":"{code}","message":"recorded","details":[{{"@type":"type.googleapis.com/google.rpc.ErrorInfo","reason":"{reason}","domain":"a2a-protocol.org"}}]}}}}"#
        ),
    )
}

/// One request read off `stream`: its head, and its body by `Content-Length`.
fn read_request(stream: &mut TcpStream) -> Option<(String, Vec<u8>)> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut head = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let end = line == "\r\n";
        head.push_str(&line);
        if end {
            break;
        }
    }
    let length = head
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    let mut body = vec![0; length];
    reader.read_exact(&mut body).ok()?;
    Some((head, body))
}

/// The task with an empty id the planted defect answers: the least the binding decodes as a
/// task.
pub const EMPTY_TASK: &str = r#"{"id":"","status":{"state":"TASK_STATE_SUBMITTED"}}"#;

/// Whether `target` is `GetTask`'s: `GET [/{tenant}]/tasks/{id}`, no verb, no deeper path.
fn get_task(method: &str, target: &str) -> bool {
    let path = target.split('?').next().unwrap_or_default();
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let task = match segments.as_slice() {
        ["tasks", task] | [_, "tasks", task] => *task,
        _ => return false,
    };
    method == "GET" && !task.is_empty() && !task.contains(':')
}

/// A proxy in front of `upstream` that plants a defect: it answers `GetTask` of an id the
/// upstream does not know (`404`) with `200` and [`EMPTY_TASK`]. Every other exchange passes as
/// it is.
pub fn planted(upstream: SocketAddr) -> Endpoint {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = listener.local_addr().expect("bound address");
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            std::thread::spawn(move || relay(stream, upstream));
        }
    });
    Endpoint { address }
}

fn relay(mut client: TcpStream, upstream: SocketAddr) {
    let Some((head, body)) = read_request(&mut client) else {
        return;
    };
    let Ok(mut server) = TcpStream::connect(upstream) else {
        return;
    };
    if server.write_all(head.as_bytes()).is_err() || server.write_all(&body).is_err() {
        return;
    }
    let mut words = head.lines().next().unwrap_or_default().split(' ');
    let method = words.next().unwrap_or_default();
    let target = words.next().unwrap_or_default();
    if !get_task(method, target) {
        let _ = std::io::copy(&mut server, &mut client);
        return;
    }
    let mut answered = Vec::new();
    let _ = server.read_to_end(&mut answered);
    let unknown = answered.starts_with(b"HTTP/1.1 404");
    let written = if unknown {
        answer(200, EMPTY_TASK).into_bytes()
    } else {
        answered
    };
    let _ = client.write_all(&written);
}
