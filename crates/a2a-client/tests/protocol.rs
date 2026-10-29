//! Acceptance 4 and 6: an answer that is not the binding's is `Error::Protocol`, never a panic,
//! fed from a socket that answers with exactly those bytes; only `http://` base URLs are taken.

mod common;

use a2a::tasks::{GetTask, SubscribeToTask, TaskId};
use a2a_client::{Client, Error};
use common::{answer, answering, client_of, message, send};

fn get_task() -> GetTask {
    GetTask {
        tenant: None,
        id: TaskId("t-1".to_owned()),
        history_length: None,
    }
}

fn is_protocol<T: std::fmt::Debug>(result: &Result<T, Error>) -> bool {
    matches!(result, Err(Error::Protocol(_)))
}

#[test]
fn a_body_that_is_not_json_is_a_protocol_error() {
    for status in [200, 404, 500] {
        let (address, _) = answering(answer(status, "text/html", "<html>no</html>"));
        let answered = client_of(address).get_task(&get_task());
        assert!(is_protocol(&answered), "{status}: {answered:?}");
    }
}

#[test]
fn an_error_body_that_is_not_a_google_rpc_status_is_a_protocol_error() {
    for body in [
        "{}",
        "{\"error\":\"no\"}",
        "{\"error\":{\"code\":\"404\"}}",
        "[]",
    ] {
        let (address, _) = answering(answer(404, "application/a2a+json", body));
        let answered = client_of(address).get_task(&get_task());
        assert!(is_protocol(&answered), "{body}: {answered:?}");
    }
}

#[test]
fn a_success_body_that_is_not_the_operation_s_type_is_a_protocol_error() {
    for body in ["{\"id\":5}", "{}", "[]", "\"task\""] {
        let (address, _) = answering(answer(200, "application/a2a+json", body));
        let client = client_of(address);
        let got = client.get_task(&get_task());
        assert!(is_protocol(&got), "GetTask {body}: {got:?}");
        let sent = client.send_message(&send(message("hello"), None));
        assert!(is_protocol(&sent), "SendMessage {body}: {sent:?}");
        let card = client.agent_card();
        assert!(is_protocol(&card), "card {body}: {card:?}");
    }
}

#[test]
fn a_send_message_answer_with_neither_a_task_nor_a_message_is_a_protocol_error() {
    let (address, _) = answering(answer(200, "application/a2a+json", "{}"));
    let sent = client_of(address).send_message(&send(message("hello"), None));
    assert!(is_protocol(&sent), "{sent:?}");
}

#[test]
fn an_answer_that_is_not_http_is_a_protocol_error() {
    for bytes in [
        &b"garbage\r\n\r\n"[..],
        b"",
        b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\n{}",
    ] {
        let (address, _) = answering(bytes.to_vec());
        let answered = client_of(address).get_task(&get_task());
        assert!(
            is_protocol(&answered),
            "{:?}: {answered:?}",
            String::from_utf8_lossy(bytes)
        );
    }
}

const TASK: &str = "{\"task\":{\"id\":\"t-1\",\"contextId\":\"c-1\",\"status\":{\"state\":\"TASK_STATE_WORKING\"}}}";

fn stream_answer(events: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n{events}"
    )
    .into_bytes()
}

#[test]
fn a_stream_that_ends_inside_an_event_is_a_protocol_error() {
    for cut in [
        format!("data: {TASK}\n\ndata: {{\"statusUpdate\""),
        format!("data: {TASK}\n\ndata: {TASK}\n"),
        format!("data: {TASK}\n\ndata: {TASK}"),
    ] {
        let (address, _) = answering(stream_answer(&cut));
        let mut events = client_of(address)
            .subscribe_to_task(&SubscribeToTask {
                tenant: None,
                id: TaskId("t-1".to_owned()),
            })
            .expect("the stream opens");
        let first = events.next().expect("a first event");
        assert!(first.is_ok(), "{cut:?}: {first:?}");
        let second = events.next().expect("the cut event");
        assert!(is_protocol(&second), "{cut:?}: {second:?}");
        assert!(events.next().is_none(), "{cut:?}: the stream goes on");
    }
}

#[test]
fn a_stream_event_that_is_not_a_stream_response_is_a_protocol_error() {
    for data in ["not json", "{\"task\":5}"] {
        let (address, _) = answering(stream_answer(&format!("data: {data}\n\n")));
        let mut events = client_of(address)
            .send_streaming_message(&common::stream(message("hello"), None))
            .expect("the stream opens");
        let first = events.next().expect("an event");
        assert!(is_protocol(&first), "{data}: {first:?}");
        assert!(events.next().is_none(), "{data}: the stream goes on");
    }
}

#[test]
fn a_stream_answered_as_json_is_a_protocol_error() {
    let (address, _) = answering(answer(200, "application/a2a+json", TASK));
    let opened = client_of(address).subscribe_to_task(&SubscribeToTask {
        tenant: None,
        id: TaskId("t-1".to_owned()),
    });
    assert!(is_protocol(&opened.map(|_| ())));
}

#[test]
fn a_stream_reads_multi_line_data_comments_and_crlf() {
    let events = format!(
        ": a comment\r\nevent: message\r\ndata: {{\"task\":\r\ndata: {}}}\r\n\r\ndata: {TASK}\n\n",
        &TASK["{\"task\":".len()..TASK.len() - 1]
    );
    let (address, _) = answering(stream_answer(&events));
    let read: Vec<_> = client_of(address)
        .subscribe_to_task(&SubscribeToTask {
            tenant: None,
            id: TaskId("t-1".to_owned()),
        })
        .expect("the stream opens")
        .collect::<Result<_, _>>()
        .expect("two events");
    assert_eq!(read.len(), 2);
    assert_eq!(read[0], read[1]);
}

#[test]
fn a_chunked_body_is_read() {
    let body = "{\"id\":\"t-1\",\"status\":{\"state\":\"TASK_STATE_COMPLETED\"}}";
    let (head, tail) = body.split_at(10);
    let bytes = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/a2a+json\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{head}\r\n{:x};ext=1\r\n{tail}\r\n0\r\n\r\n",
        head.len(),
        tail.len()
    );
    let (address, _) = answering(bytes.into_bytes());
    let task = client_of(address)
        .get_task(&get_task())
        .expect("a chunked task");
    assert_eq!(task.id, TaskId("t-1".to_owned()));
}

#[test]
fn a_broken_chunked_body_is_a_protocol_error() {
    let bytes = "HTTP/1.1 200 OK\r\nContent-Type: application/a2a+json\r\nTransfer-Encoding: chunked\r\n\r\nzz\r\n{}\r\n0\r\n\r\n";
    let (address, _) = answering(bytes.as_bytes().to_vec());
    let answered = client_of(address).get_task(&get_task());
    assert!(is_protocol(&answered), "{answered:?}");
}

#[test]
fn an_https_url_is_refused_as_an_unsupported_scheme() {
    for url in [
        "https://127.0.0.1:8443",
        "HTTPS://agent.example",
        "ftp://agent.example",
    ] {
        match Client::new(url) {
            Err(Error::UnsupportedScheme(_)) => {}
            other => panic!("{url}: {other:?}"),
        }
    }
}

#[test]
fn a_base_url_that_is_not_one_is_refused() {
    for url in [
        "127.0.0.1:8080",
        "http://",
        "http://host:port",
        "http://a b",
    ] {
        match Client::new(url) {
            Err(Error::InvalidRequest(_)) => {}
            other => panic!("{url}: {other:?}"),
        }
    }
}

#[test]
fn requests_go_under_the_base_url_s_path_with_the_version_header() {
    let body = "{\"id\":\"t-1\",\"status\":{\"state\":\"TASK_STATE_COMPLETED\"}}";
    let (address, seen) = answering(answer(200, "application/a2a+json", body));
    let client = Client::new(&format!("http://{address}/a2a/v1/")).expect("a base URL");
    client
        .get_task(&GetTask {
            tenant: Some("a b/c".to_owned()),
            id: TaskId("t 1".to_owned()),
            history_length: Some(2),
        })
        .expect("the task");
    let seen = seen.lock().expect("the list");
    let request = &seen[0];
    assert_eq!(request.path, "/a2a/v1/a%20b%2Fc/tasks/t%201");
    assert!(
        request.head.contains("?historyLength=2 "),
        "{}",
        request.head
    );
    assert!(
        request.head.contains("\r\nA2A-Version: 1.0\r\n"),
        "{}",
        request.head
    );
}

#[test]
fn a_header_value_with_a_line_break_is_refused_before_anything_is_sent() {
    let (address, seen) = answering(answer(200, "application/json", "{}"));
    let answered = client_of(address)
        .with_bearer("t0ken\r\nX-Injected: 1")
        .get_task(&get_task());
    assert!(
        matches!(answered, Err(Error::InvalidRequest(_))),
        "{answered:?}"
    );
    assert!(seen.lock().expect("the list").is_empty());
}
