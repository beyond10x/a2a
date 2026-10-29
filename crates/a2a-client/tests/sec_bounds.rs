//! Security review: against a hostile server a call ends within a bound set by the client's
//! timeout, holds a bounded amount of memory, and fails with an error instead of a panic.

mod common;

use std::io::Write as _;
use std::net::{SocketAddr, TcpListener};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use a2a::tasks::{GetTask, SubscribeToTask, TaskId};
use a2a_client::{Client, ListTasksRequest};
use common::{answering, client_of, read_request};

/// The client's timeout in these cases.
const TIMEOUT: Duration = Duration::from_millis(500);
/// How long a call may take against a server that never finishes: a few timeouts.
const BOUND: Duration = Duration::from_secs(3);
/// How long the hostile server keeps going before it gives up and closes.
const SERVER_GIVES_UP: Duration = Duration::from_secs(12);

fn get_task() -> GetTask {
    GetTask {
        tenant: None,
        id: TaskId("t-1".to_owned()),
        history_length: None,
    }
}

/// A socket on port 0 that reads one request, writes `first`, then writes what `next` makes
/// every `pause` until it has run for [`SERVER_GIVES_UP`] or the client went away.
fn hostile(first: &'static [u8], next: fn() -> Vec<u8>, pause: Duration) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = listener.local_addr().expect("bound address");
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let _ = read_request(&stream);
        if stream.write_all(first).is_err() {
            return;
        }
        let started = Instant::now();
        while started.elapsed() < SERVER_GIVES_UP {
            if stream.write_all(&next()).and_then(|()| stream.flush()).is_err() {
                return;
            }
            if !pause.is_zero() {
                std::thread::sleep(pause);
            }
        }
    });
    address
}

/// Runs `call` on its own thread and answers how long it took, or `None` past [`BOUND`].
fn within_bound(call: impl FnOnce() -> String + Send + 'static) -> Result<Duration, String> {
    let (done, finished) = mpsc::channel();
    let started = Instant::now();
    std::thread::spawn(move || {
        let answered = call();
        let _ = done.send(answered);
    });
    match finished.recv_timeout(BOUND) {
        Ok(_) => Ok(started.elapsed()),
        Err(_) => Err(format!(
            "the call was still running after {BOUND:?} with a {TIMEOUT:?} timeout"
        )),
    }
}

#[test]
fn a_call_ends_within_a_bound_against_a_server_that_trickles_its_answer() {
    let address = hostile(b"HTTP/1.1 200 OK\r\nX-Slow: ", || b"a".to_vec(), TIMEOUT / 5);
    let took = within_bound(move || {
        let answered = client_of(address)
            .with_timeout(Some(TIMEOUT))
            .get_task(&get_task());
        format!("{answered:?}")
    });
    assert!(
        took.is_ok(),
        "{}: each read is bounded by the timeout, the whole call by nothing",
        took.unwrap_err()
    );
}

#[test]
fn a_chunked_body_s_trailer_section_is_bounded() {
    let address = hostile(
        b"HTTP/1.1 200 OK\r\nContent-Type: application/a2a+json\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n",
        || b"X-Trailer: y\r\n".repeat(256),
        Duration::ZERO,
    );
    let took = within_bound(move || {
        let answered = client_of(address)
            .with_timeout(Some(TIMEOUT))
            .get_task(&get_task());
        format!("{answered:?}")
    });
    assert!(
        took.is_ok(),
        "{}: trailer lines after the last chunk are read without a count or byte limit",
        took.unwrap_err()
    );
}

#[test]
fn hostile_answers_are_errors_and_never_panics() {
    let deep = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: 200000\r\n\r\n{}",
        "[".repeat(200_000)
    );
    let wide = format!(
        "HTTP/1.1 500 X\r\nContent-Length: {}\r\n\r\n{}",
        "é".repeat(300).len(),
        "é".repeat(300)
    );
    let numbers = "{\"tasks\":[],\"pageSize\":1e400,\"totalSize\":-99999999999999999999999}";
    let numbers = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{numbers}",
        numbers.len()
    );
    let code = "{\"error\":{\"code\":99999999999999999999999,\"details\":[{\"@type\":7}]}}";
    let code = format!("HTTP/1.1 400 X\r\nContent-Length: {}\r\n\r\n{code}", code.len());
    let answers: Vec<Vec<u8>> = vec![
        b"HTTP/1.1 200 OK\r\nContent-Length: 99999999999999999999999\r\n\r\n{}".to_vec(),
        b"HTTP/1.1 200 OK\r\nContent-Length: 18446744073709551615\r\n\r\n{}".to_vec(),
        b"HTTP/1.1 200 OK\r\nContent-Length: -1\r\n\r\n{}".to_vec(),
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nffffffffffffffffffff\r\nabc".to_vec(),
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nffffffffffffffff\r\nabc".to_vec(),
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nabcdef\r\n0\r\n\r\n".to_vec(),
        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\n\xff\xfe\xfd\xfc\xfb".to_vec(),
        b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\n{".to_vec(),
        b"HTTP/1.1 \xff\xff\xff OK\r\n\r\n".to_vec(),
        b"HTTP/1.1 0200 OK\r\n\r\n".to_vec(),
        b"HTTP/1.1 +20 OK\r\n\r\n".to_vec(),
        b"HTTP/1.1 200\r\n\r\n".to_vec(),
        b"HTTP/1.1 200 OK\r\nno colon here\r\n\r\n".to_vec(),
        b"HTTP/1.1 200 OK\r\n:\r\n\r\n".to_vec(),
        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\ndata: \xff\n\n".to_vec(),
        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\ndata: [[[[[[[[\n\n".to_vec(),
        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\ndata\n\n".to_vec(),
        b"\r\n\r\n".to_vec(),
        Vec::new(),
        deep.into_bytes(),
        wide.into_bytes(),
        numbers.into_bytes(),
        code.into_bytes(),
    ];
    for (at, bytes) in answers.into_iter().enumerate() {
        let (address, _) = answering(bytes);
        let client: Client = client_of(address).with_timeout(Some(TIMEOUT));
        let task = client.get_task(&get_task());
        assert!(task.is_err(), "answer {at}: {task:?}");
        let _ = client.list_tasks(&ListTasksRequest::default());
        if let Ok(events) = client.subscribe_to_task(&SubscribeToTask {
            tenant: None,
            id: TaskId("t-1".to_owned()),
        }) {
            for event in events.take(4) {
                let _ = event;
            }
        }
    }
}
