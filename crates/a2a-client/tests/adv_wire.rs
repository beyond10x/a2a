//! Adversary, story:client: HTTP/1.1 framing and server-sent event parsing against a hostile or
//! unusual server. Each socket here writes exactly the bytes a case names; a flood counts the bytes
//! it got onto the connection before the client hung up.

mod common;

use std::io::Write as _;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread::JoinHandle;
use std::time::Duration;

use a2a::tasks::{GetTask, SubscribeToTask, TaskId};
use a2a_client::{Client, Error};
use common::{answering, client_of, read_request};

/// The client's bound on a body and on one event (`transport.rs` `MAX_BODY`).
const MAX_BODY: u64 = 16 * 1024 * 1024;
/// How much a flood offers: four times the bound.
const FLOOD: u64 = 4 * MAX_BODY;
/// What a bounded client may let through: the bound twice over, plus socket buffers.
const BOUNDED: u64 = 2 * MAX_BODY + 8 * 1024 * 1024;

const TASK_BODY: &str = "{\"id\":\"t-1\",\"status\":{\"state\":\"TASK_STATE_COMPLETED\"}}";
const EVENT: &str = "{\"task\":{\"id\":\"t-1\",\"contextId\":\"c-1\",\"status\":{\"state\":\"TASK_STATE_WORKING\"}}}";

fn get_task() -> GetTask {
    GetTask {
        tenant: None,
        id: TaskId("t-1".to_owned()),
        history_length: None,
    }
}

fn subscribe() -> SubscribeToTask {
    SubscribeToTask {
        tenant: None,
        id: TaskId("t-1".to_owned()),
    }
}

/// A client that gives up after ten seconds of silence, so a defect cannot hang the suite.
fn bounded_client(address: SocketAddr) -> Client {
    client_of(address).with_timeout(Some(Duration::from_secs(10)))
}

/// A socket on port 0 that takes one connection, reads its request, then runs `script` on it and
/// hands back what the script answers (the bytes it wrote).
fn scripted(
    script: impl FnOnce(&mut TcpStream) -> u64 + Send + 'static,
) -> (SocketAddr, JoinHandle<u64>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = listener.local_addr().expect("bound address");
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("a connection");
        let _ = read_request(&stream);
        let written = script(&mut stream);
        let _ = stream.flush();
        let _ = stream.shutdown(std::net::Shutdown::Both);
        written
    });
    (address, handle)
}

/// Writes `head`, then `unit` over and over until `FLOOD` bytes are out or the client hung up,
/// then `tail`; answers the bytes written.
fn flood(stream: &mut TcpStream, head: &[u8], unit: &[u8], tail: &[u8]) -> u64 {
    let mut written = 0_u64;
    if stream.write_all(head).is_err() {
        return written;
    }
    written += head.len() as u64;
    let mut block = Vec::with_capacity(64 * 1024);
    while block.len() + unit.len() <= 64 * 1024 {
        block.extend_from_slice(unit);
    }
    while written < FLOOD {
        if stream.write_all(&block).is_err() {
            return written;
        }
        written += block.len() as u64;
    }
    if stream.write_all(tail).is_ok() {
        written += tail.len() as u64;
    }
    written
}

// ---- red: defects --------------------------------------------------------------------------

/// RFC 9110 section 15.2: "A client MUST be able to parse one or more 1xx responses received
/// prior to a final response, even if the client does not expect one." A 100 or 103 before the
/// 200 is skipped and the 200 is the answer.
#[test]
fn an_interim_1xx_answer_before_the_final_one_is_skipped() {
    let mut wrong = Vec::new();
    for interim in [
        "HTTP/1.1 100 Continue\r\n\r\n",
        "HTTP/1.1 103 Early Hints\r\nLink: </style.css>; rel=preload\r\n\r\n",
    ] {
        let bytes = format!(
            "{interim}HTTP/1.1 200 OK\r\nContent-Type: application/a2a+json\r\nContent-Length: {}\r\n\r\n{TASK_BODY}",
            TASK_BODY.len()
        );
        let (address, _) = answering(bytes.into_bytes());
        let answered = bounded_client(address).get_task(&get_task());
        match answered {
            Ok(task) => assert_eq!(task.id, TaskId("t-1".to_owned()), "{interim:?}"),
            Err(error) => wrong.push(format!(
                "{interim:?}: the final 200 was not read: {error:?}"
            )),
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// A chunked answer's trailer section is not counted against any bound: after the last chunk a
/// server can write trailer lines without end, each under the 1024-byte line bound, and the client
/// reads all of them. Offered four times the body bound, a bounded client hangs up long before.
#[test]
fn a_chunked_trailer_flood_is_bounded() {
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/a2a+json\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{TASK_BODY}\r\n0\r\n",
        TASK_BODY.len()
    );
    let (address, server) = scripted(move |stream| {
        flood(
            stream,
            head.as_bytes(),
            b"X-Trailer: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r\n",
            b"\r\n",
        )
    });
    let answered = bounded_client(address).get_task(&get_task());
    let written = server.join().expect("the server thread");
    assert!(
        written < BOUNDED,
        "the client read {written} bytes of trailers (bound {MAX_BODY}) and answered {answered:?}"
    );
    assert!(
        matches!(answered, Err(Error::Protocol(_))),
        "a trailer flood is not the binding's: {answered:?}"
    );
}

/// RFC 9110 section 8.6: `Content-Length = 1*DIGIT`; RFC 9112 section 6.3: a message with
/// several `Content-Length` values that differ is an unrecoverable error. The client takes `+N`
/// (`u64::from_str` accepts a sign) and the first of two differing values.
#[test]
fn a_content_length_that_is_not_one_decimal_number_is_a_protocol_error() {
    let n = TASK_BODY.len();
    let mut wrong = Vec::new();
    for head in [
        format!("Content-Length: +{n}\r\n"),
        format!("Content-Length: {n}\r\nContent-Length: {}\r\n", n + 7),
    ] {
        let bytes = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/a2a+json\r\n{head}\r\n{TASK_BODY}"
        );
        let (address, _) = answering(bytes.into_bytes());
        let answered = bounded_client(address).get_task(&get_task());
        if !matches!(answered, Err(Error::Protocol(_))) {
            wrong.push(format!("{head:?}: {answered:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// A stream framed by `Content-Length` that the server cuts between two events, short of its
/// announced length, ends as if complete: `Events` never checks the body's `finish`, which
/// `Answer::text` does for every unary answer.
#[test]
fn a_content_length_stream_cut_between_events_is_a_protocol_error() {
    let events = format!("data: {EVENT}\n\n");
    let bytes = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n{events}",
        events.len() * 2
    );
    let (address, _) = answering(bytes.into_bytes());
    let mut stream = bounded_client(address)
        .subscribe_to_task(&subscribe())
        .expect("the stream opens");
    let first = stream.next().expect("the first event");
    assert!(first.is_ok(), "{first:?}");
    let second = stream.next();
    assert!(
        matches!(second, Some(Err(Error::Protocol(_)))),
        "the body ended {} bytes short of its Content-Length and the stream ended clean: {second:?}",
        events.len()
    );
}

/// WHATWG HTML, server-sent events: "Lines must be separated by either a U+000D CARRIAGE RETURN
/// U+000A LINE FEED (CRLF) character pair, a single U+000A LINE FEED (LF) character, or a single
/// U+000D CARRIAGE RETURN (CR) character." The client splits on LF only.
#[test]
fn a_stream_with_cr_only_line_breaks_is_read() {
    let bytes = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\ndata: {EVENT}\r\rdata: {EVENT}\r\r"
    );
    let (address, _) = answering(bytes.into_bytes());
    let read: Result<Vec<_>, _> = bounded_client(address)
        .subscribe_to_task(&subscribe())
        .expect("the stream opens")
        .collect();
    match read {
        Ok(events) => assert_eq!(events.len(), 2, "{events:?}"),
        Err(error) => panic!("two CR-separated events are not read: {error:?}"),
    }
}

// ---- green: guards the suite does not have ---------------------------------------------------

/// Guards `transport.rs` `MAX_HEADERS` and `MAX_HEAD`: a head without end is cut.
#[test]
fn an_endless_head_is_cut_at_its_bound() {
    let (address, server) = scripted(|stream| {
        flood(
            stream,
            b"HTTP/1.1 200 OK\r\n",
            b"X-Filler: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r\n",
            b"\r\n",
        )
    });
    let answered = bounded_client(address).get_task(&get_task());
    let written = server.join().expect("the server thread");
    assert!(matches!(answered, Err(Error::Protocol(_))), "{answered:?}");
    assert!(written < BOUNDED, "the client read {written} bytes of head");
}

/// Guards `Answer::text`'s `MAX_BODY`: a close-delimited body without end is cut at 16 MiB.
#[test]
fn an_endless_body_is_cut_at_its_bound() {
    let (address, server) = scripted(|stream| {
        flood(
            stream,
            b"HTTP/1.1 200 OK\r\nContent-Type: application/a2a+json\r\nConnection: close\r\n\r\n",
            b"                                                                ",
            b"{}",
        )
    });
    let answered = bounded_client(address).get_task(&get_task());
    let written = server.join().expect("the server thread");
    assert!(matches!(answered, Err(Error::Protocol(_))), "{answered:?}");
    assert!(written < BOUNDED, "the client read {written} bytes of body");
}

/// Guards `events.rs`' per-event bound: one stream line without end is cut at 16 MiB.
#[test]
fn an_endless_stream_line_is_cut_at_its_bound() {
    let (address, server) = scripted(|stream| {
        flood(
            stream,
            b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\ndata: ",
            b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            b"\n\n",
        )
    });
    let mut events = bounded_client(address)
        .subscribe_to_task(&subscribe())
        .expect("the stream opens");
    let first = events.next();
    let written = server.join().expect("the server thread");
    assert!(matches!(first, Some(Err(Error::Protocol(_)))), "{first:?}");
    assert!(events.next().is_none(), "the stream goes on");
    assert!(
        written < BOUNDED,
        "the client read {written} bytes of one line"
    );
}

/// Guards `Chunked::read`'s check after a chunk's data: a chunk longer than its size line says
/// is not taken as the body.
#[test]
fn a_chunk_longer_than_its_size_is_a_protocol_error() {
    let bytes = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/a2a+json\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{TASK_BODY}junk!\r\n0\r\n\r\n",
        TASK_BODY.len()
    );
    let (address, _) = answering(bytes.into_bytes());
    let answered = bounded_client(address).get_task(&get_task());
    assert!(matches!(answered, Err(Error::Protocol(_))), "{answered:?}");
}

/// Guards RFC 9112 section 6.3 item 3: `Transfer-Encoding: chunked` overrides `Content-Length`,
/// and the last coding is the one that counts.
#[test]
fn chunked_framing_wins_over_content_length() {
    for coding in ["chunked", "gzip, chunked", "Chunked"] {
        let bytes = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/a2a+json\r\nContent-Length: 3\r\nTransfer-Encoding: {coding}\r\n\r\n{:x}\r\n{TASK_BODY}\r\n0\r\n\r\n",
            TASK_BODY.len()
        );
        let (address, _) = answering(bytes.into_bytes());
        let task = bounded_client(address)
            .get_task(&get_task())
            .unwrap_or_else(|error| panic!("{coding}: {error:?}"));
        assert_eq!(task.id, TaskId("t-1".to_owned()), "{coding}");
    }
}

/// Guards `events.rs`' `held.push('\n')`: the lines of one event's `data` are joined by a line
/// feed. Split inside a JSON string, the joined data holds a raw line feed in that string, so it
/// is either refused or decoded with it; read as `t-1`, the lines were joined with nothing.
#[test]
fn multi_line_data_is_joined_by_a_line_feed() {
    let (start, rest) = EVENT.split_at(EVENT.find("t-1").expect("the id") + 2);
    let bytes = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\ndata: {start}\ndata: {rest}\n\n"
    );
    let (address, _) = answering(bytes.into_bytes());
    let first = bounded_client(address)
        .subscribe_to_task(&subscribe())
        .expect("the stream opens")
        .next()
        .expect("an event");
    let shown = format!("{first:?}");
    assert!(
        !shown.contains("\"t-1\""),
        "the data lines were joined without a line feed: {shown}"
    );
    assert!(
        matches!(first, Err(Error::Protocol(_))) || shown.contains("t-\\n1"),
        "{shown}"
    );
}

/// Guards `transport.rs` `read > MAX_HEAD` on its own: fifty header lines of 60 KiB each, under the
/// line count and each under the per-line bound, make a 3 MiB head, which is over 64 KiB.
#[test]
fn a_head_of_few_long_lines_over_its_byte_bound_is_a_protocol_error() {
    let filler = "a".repeat(60 * 1024);
    let mut bytes = String::from("HTTP/1.1 200 OK\r\nContent-Type: application/a2a+json\r\n");
    for at in 0..50 {
        bytes.push_str(&format!("X-Filler-{at}: {filler}\r\n"));
    }
    bytes.push_str(&format!(
        "Content-Length: {}\r\n\r\n{TASK_BODY}",
        TASK_BODY.len()
    ));
    let (address, _) = answering(bytes.into_bytes());
    let answered = bounded_client(address).get_task(&get_task());
    assert!(matches!(answered, Err(Error::Protocol(_))), "{answered:?}");
}
