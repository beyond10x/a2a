//! An `a2a-server` on port 0, sockets that answer with bytes a test writes, and request builders
//! over the generated types, shared by the integration tests.

#![allow(dead_code)]

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use a2a::tasks::{
    ContextId, Message, Part, Role, SendMessage, SendStreamingMessage, TaskId, TaskSnapshot,
};
use a2a_client::Client;
use a2a_server::{Access, Agent, EchoAgent, Limits, Settings};

/// A running server.
pub struct Server {
    pub address: SocketAddr,
}

impl Server {
    /// A client of this server.
    pub fn client(&self) -> Client {
        Client::new(&format!("http://{}", self.address)).expect("an http:// base URL")
    }
}

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

/// The built-in card with `change` applied.
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

/// A new user message with these parts, continuing `task` in `context` where given.
pub fn message_of(parts: Vec<Part>, task: Option<&TaskId>, context: Option<&str>) -> Message {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    Message {
        message_id: format!("m-{}", NEXT.fetch_add(1, Ordering::Relaxed)),
        context_id: context.map(|context| ContextId(context.to_owned())),
        task_id: task.cloned(),
        role: Role::ROLEUSER,
        parts,
        metadata: None,
        extensions: None,
        reference_task_ids: None,
    }
}

/// A new user message with one text part.
pub fn message(text: &str) -> Message {
    message_of(vec![text_part(text)], None, None)
}

/// `SendMessage` of `message` under `tenant`.
pub fn send(message: Message, tenant: Option<&str>) -> SendMessage {
    SendMessage {
        tenant: tenant.map(str::to_owned),
        message,
        configuration: None,
        metadata: None,
    }
}

/// `SendStreamingMessage` of `message` under `tenant`.
pub fn stream(message: Message, tenant: Option<&str>) -> SendStreamingMessage {
    SendStreamingMessage {
        tenant: tenant.map(str::to_owned),
        message,
        configuration: None,
        metadata: None,
    }
}

/// The task a sent message created or continued.
pub fn task_of(client: &Client, request: &SendMessage) -> TaskSnapshot {
    client
        .send_message(request)
        .expect("SendMessage")
        .task
        .expect("a task, not a direct message")
}

/// The wire name of a task's state, such as `TASK_STATE_COMPLETED`.
pub fn state(task: &TaskSnapshot) -> String {
    wire_state(task.status.state)
}

/// The wire name of a state.
pub fn wire_state(state: a2a::tasks::WireTaskState) -> String {
    let mut out = String::new();
    a2a::server::wire::encode_a2a_tasks_task_state_2(&state, &mut out);
    out.trim_matches('"').to_owned()
}

/// One request a socket read: method, path without its query, and the whole head.
#[derive(Debug, Clone)]
pub struct Seen {
    pub method: String,
    pub path: String,
    pub head: String,
    pub body: String,
}

/// Reads one HTTP/1.1 request off `stream`: its head, then as many body bytes as its
/// `Content-Length` names.
pub fn read_request(stream: &TcpStream) -> Option<Seen> {
    let mut reader = BufReader::new(stream);
    let mut head = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        head.push_str(&line);
        if line == "\r\n" {
            break;
        }
    }
    let length = head
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.trim().eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = vec![0; length];
    reader.read_exact(&mut body).ok()?;
    let mut words = head.split_whitespace();
    let method = words.next()?.to_owned();
    let target = words.next()?;
    let path = target.split('?').next().unwrap_or(target).to_owned();
    Some(Seen {
        method,
        path,
        head,
        body: String::from_utf8_lossy(&body).into_owned(),
    })
}

/// A socket on port 0 that answers every connection with `answer`, written as it is, then closes
/// it; every request it read is kept in the list it hands back.
pub fn answering(answer: Vec<u8>) -> (SocketAddr, Arc<Mutex<Vec<Seen>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = listener.local_addr().expect("bound address");
    let seen = Arc::new(Mutex::new(Vec::new()));
    let kept = Arc::clone(&seen);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            if let Some(request) = read_request(&stream) {
                kept.lock().expect("the list").push(request);
            }
            let _ = stream.write_all(&answer);
            let _ = stream.flush();
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    });
    (address, seen)
}

/// An HTTP/1.1 answer with this status, content type and body, closing its connection.
pub fn answer(status: u16, content_type: &str, body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status} X\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

/// A client of a socket [`answering`] made.
pub fn client_of(address: SocketAddr) -> Client {
    Client::new(&format!("http://{address}")).expect("an http:// base URL")
}
