//! Adversary, story:client: the timeout bounds a server that goes silent, before its answer and
//! between two events of a stream.

mod common;

use std::io::Write as _;
use std::net::{SocketAddr, TcpListener};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use a2a::tasks::{GetTask, SubscribeToTask, TaskId};
use a2a_client::Error;
use common::{client_of, read_request};

const EVENT: &str = "{\"task\":{\"id\":\"t-1\",\"contextId\":\"c-1\",\"status\":{\"state\":\"TASK_STATE_WORKING\"}}}";

/// A socket that takes one connection, reads its request, writes `bytes`, and then holds the
/// connection open without a word until the returned sender is dropped, or five seconds pass and
/// it closes (so a client without a read timeout reads an end, not a hang).
fn silent_after(bytes: Vec<u8>) -> (SocketAddr, mpsc::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = listener.local_addr().expect("bound address");
    let (hold, held) = mpsc::channel::<()>();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("a connection");
        let _ = read_request(&stream);
        let _ = stream.write_all(&bytes);
        let _ = stream.flush();
        let _ = held.recv_timeout(Duration::from_secs(5));
    });
    (address, hold)
}

fn timed_out(error: &Error) -> bool {
    matches!(error, Error::Transport(io) if matches!(io.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut))
}

/// A server that reads the request and never answers is a transport timeout after about the
/// timeout, not a hang.
#[test]
fn a_silent_server_is_a_transport_timeout() {
    let (address, hold) = silent_after(Vec::new());
    let started = Instant::now();
    let answered = client_of(address)
        .with_timeout(Some(Duration::from_millis(300)))
        .get_task(&GetTask {
            tenant: None,
            id: TaskId("t-1".to_owned()),
            history_length: None,
        });
    let took = started.elapsed();
    drop(hold);
    match answered {
        Err(error) => assert!(timed_out(&error), "{error:?}"),
        Ok(task) => panic!("{task:?}"),
    }
    assert!(took < Duration::from_secs(5), "took {took:?}");
}

/// A server that stops answering inside its head is a timeout too.
#[test]
fn a_server_silent_inside_its_head_is_a_transport_timeout() {
    let (address, hold) = silent_after(b"HTTP/1.1 200 OK\r\nContent-Ty".to_vec());
    let answered = client_of(address)
        .with_timeout(Some(Duration::from_millis(300)))
        .get_task(&GetTask {
            tenant: None,
            id: TaskId("t-1".to_owned()),
            history_length: None,
        });
    drop(hold);
    match answered {
        Err(error) => assert!(timed_out(&error), "{error:?}"),
        Ok(task) => panic!("{task:?}"),
    }
}

/// A stream that goes silent after one event yields that event, then one transport timeout, and
/// ends.
#[test]
fn a_stream_silent_between_events_ends_with_a_timeout() {
    let (address, hold) = silent_after(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\ndata: {EVENT}\n\n"
        )
        .into_bytes(),
    );
    let mut events = client_of(address)
        .with_timeout(Some(Duration::from_millis(300)))
        .subscribe_to_task(&SubscribeToTask {
            tenant: None,
            id: TaskId("t-1".to_owned()),
        })
        .expect("the stream opens");
    let first = events.next().expect("the event");
    assert!(first.is_ok(), "{first:?}");
    match events.next() {
        Some(Err(error)) => assert!(timed_out(&error), "{error:?}"),
        other => panic!("{other:?}"),
    }
    assert!(events.next().is_none(), "the stream goes on");
    drop(hold);
}
