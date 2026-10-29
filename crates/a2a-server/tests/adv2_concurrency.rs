//! Adversarial cases, pass 2, against the concurrency that fix pass 1 added to `a2a-server`
//! (story:server, wave 3). lib.rs:61-64 says "a slow client holds only its own connection".

mod common;

use std::io::{Read as _, Write as _};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use a2a::tasks::{Artifact, Message, Part, TaskSnapshot};
use a2a_server::{Agent, Step, Turn};
use common::{card, echo, message, start};

const LIST: &str = "/tasks?includeArtifacts=true&pageSize=100";

/// Asks for the agent card on a fresh connection and answers its status and how long it took.
fn card_after(address: SocketAddr) -> (u16, Duration) {
    let started = Instant::now();
    let mut stream = TcpStream::connect(address).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(120)))
        .expect("read timeout");
    stream
        .write_all(
            format!("GET /.well-known/agent-card.json HTTP/1.1\r\nHost: {address}\r\n\r\n")
                .as_bytes(),
        )
        .expect("write");
    let mut raw = Vec::new();
    let _ = stream.read_to_end(&mut raw);
    let status = String::from_utf8_lossy(&raw)
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    (status, started.elapsed())
}

/// router.rs:371-377 writes every plain answer on the one router thread (generated http.rs
/// `write`; `WRITE_TIMEOUT` bounds each stalled send, not the answer). A client that asks for a
/// large answer and reads none of it holds that thread, and every other client waits.
#[test]
fn adv2_a_client_that_does_not_read_its_answer_does_not_hold_back_another_client() {
    let server = echo();
    let text = "a".repeat(900_000);
    for _ in 0..5 {
        let reply = server.call("POST", "/message:send", Some(&message(&text, None, None)));
        assert_eq!(
            reply.status,
            200,
            "{}",
            &reply.body[..reply.body.len().min(300)]
        );
    }
    let started = Instant::now();
    let list = server.call("GET", LIST, None);
    let read_fully = started.elapsed();
    assert_eq!(
        list.status,
        200,
        "{}",
        &list.body[..list.body.len().min(300)]
    );

    let _unread = server.send("GET", LIST, None, &[("A2A-Version", "1.0")]);
    std::thread::sleep(Duration::from_millis(500));
    let (status, waited) = card_after(server.address);
    assert_eq!(status, 200);
    assert!(
        waited < read_fully + Duration::from_secs(2),
        "a second client waited {waited:?} for the agent card while one client held a {} byte \
         ListTasks answer unread (the same answer reads in full in {read_fully:?})",
        list.body.len()
    );
}

/// Emits one artifact in `chunks` chunks of `size` bytes, then completes, for every message.
struct Large {
    chunks: usize,
    size: usize,
}

impl Agent for Large {
    fn turn(&mut self, _message: &Message, _task: Option<&TaskSnapshot>) -> Turn {
        let mut steps = vec![Step::Work(None)];
        for chunk in 0..self.chunks {
            steps.push(Step::Emit {
                artifact: Artifact {
                    artifact_id: "large".to_owned(),
                    name: None,
                    description: None,
                    parts: vec![Part {
                        text: Some("b".repeat(self.size)),
                        raw: None,
                        url: None,
                        data: None,
                        metadata: None,
                        filename: None,
                        media_type: None,
                    }],
                    metadata: None,
                    extensions: None,
                },
                append: chunk > 0,
                last_chunk: chunk + 1 == self.chunks,
            });
        }
        steps.push(Step::Complete(None));
        Turn::Steps(steps)
    }
}

/// router.rs:422-446 `deliver` writes each event to each follower on the router thread
/// (`event`, router.rs:989). A `SendStreamingMessage` client that stops reading while its agent
/// streams a large artifact holds that thread, and every other client waits.
#[test]
fn adv2_a_stream_whose_client_does_not_read_does_not_hold_back_another_client() {
    let server = start(card(|_| {}), || {
        Box::new(Large {
            chunks: 8,
            size: 900_000,
        })
    });
    let started = Instant::now();
    let read = server.call("POST", "/message:stream", Some(&message("go", None, None)));
    let read_fully = started.elapsed();
    assert_eq!(
        read.status,
        200,
        "{}",
        &read.body[..read.body.len().min(300)]
    );

    let _unread = server.send(
        "POST",
        "/message:stream",
        Some(&message("go", None, None)),
        &[("A2A-Version", "1.0")],
    );
    std::thread::sleep(Duration::from_millis(500));
    let (status, waited) = card_after(server.address);
    assert_eq!(status, 200);
    assert!(
        waited < read_fully + Duration::from_secs(2),
        "a second client waited {waited:?} for the agent card while one client read none of a \
         {} byte event stream (the same stream reads in full in {read_fully:?})",
        read.body.len()
    );
}
