//! A stream and its client's pace: a client that keeps up gets every event, however many come at
//! once; a client that falls behind loses its stream, and nobody waits for it.

mod common;

use std::io::{BufReader, Read as _};
use std::time::{Duration, Instant};

use a2a::json::Value;
use a2a::tasks::{Artifact, Message, Part, TaskSnapshot};
use a2a_server::{Agent, Step, Turn};
use common::{at, card, message, read_head, start, text};

/// Emits `chunks` artifacts of `size` bytes, each replacing the one before, then completes.
struct Chunks {
    chunks: usize,
    size: usize,
}

impl Agent for Chunks {
    fn turn(&mut self, _message: &Message, _task: Option<&TaskSnapshot>) -> Turn {
        let mut steps = vec![Step::Work(None)];
        for chunk in 0..self.chunks {
            steps.push(Step::Emit {
                artifact: Artifact {
                    artifact_id: "chunk".to_owned(),
                    name: None,
                    description: None,
                    parts: vec![Part {
                        text: Some("c".repeat(self.size)),
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
                append: false,
                last_chunk: chunk + 1 == self.chunks,
            });
        }
        steps.push(Step::Complete(None));
        Turn::Steps(steps)
    }
}

fn artifacts(frames: &[Value]) -> usize {
    frames
        .iter()
        .filter(|frame| at(frame, &["artifactUpdate"]).is_some())
        .count()
}

#[test]
fn a_client_that_keeps_up_gets_every_event_of_a_burst() {
    let server = start(card(|_| {}), || {
        Box::new(Chunks {
            chunks: 2_000,
            size: 16,
        })
    });
    let reply = server.call("POST", "/message:stream", Some(&message("go", None, None)));
    assert_eq!(reply.status, 200, "{}", reply.body);
    let frames = reply.frames();
    assert_eq!(artifacts(&frames), 2_000, "{} events", frames.len());
    let last = frames.last().expect("a frame");
    assert_eq!(
        text(last, &["statusUpdate", "status", "state"]),
        "TASK_STATE_COMPLETED"
    );
}

#[test]
fn a_client_that_falls_behind_loses_its_stream_and_holds_back_nobody() {
    let server = start(card(|_| {}), || {
        Box::new(Chunks {
            chunks: 240,
            size: 200_000,
        })
    });
    let stream = server.send(
        "POST",
        "/message:stream",
        Some(&message("go", None, None)),
        &[("A2A-Version", "1.0")],
    );
    let mut reader = BufReader::new(stream);
    let (status, _) = read_head(&mut reader);
    assert_eq!(status, 200);

    // The server runs a request's steps and hands out their events before it answers the next
    // request, so once the card is in, every event of the stream was queued or refused.
    let started = Instant::now();
    let card = server.call("GET", "/.well-known/agent-card.json", None);
    assert_eq!(card.status, 200);
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the card took {:?} while one stream's client read nothing",
        started.elapsed()
    );

    let started = Instant::now();
    let mut rest = Vec::new();
    let read = reader.read_to_end(&mut rest);
    let took = started.elapsed();
    assert!(
        read.is_ok() && took < Duration::from_secs(10),
        "the stream did not end: {read:?} after {took:?}"
    );
    let rest = String::from_utf8_lossy(&rest);
    let events = rest.matches("data: ").count();
    assert!(
        events < 1 + 240,
        "{events} events reached a client that read none while they came"
    );
    assert!(
        !rest.contains("TASK_STATE_COMPLETED"),
        "the stream of a client that fell behind ran to its end"
    );
}
