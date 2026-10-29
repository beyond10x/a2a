//! `SendStreamingMessage` and `SubscribeToTask` as server-sent events (A2A v1.0.1 § 11.7): each
//! event one `data:` line holding a `StreamResponse`, a `Task` or a single `Message` first, the
//! stream closed at the end the specification names.

mod common;

use std::io::BufReader;

use a2a::json::Value;
use common::{at, card, echo, echo_with, message, next_frame, read_head, text};

fn kind(frame: &Value) -> &'static str {
    let Value::Object(members) = frame else {
        panic!("a frame is not an object: {frame:?}");
    };
    assert_eq!(
        members.len(),
        1,
        "a StreamResponse holds exactly one member: {frame:?}"
    );
    match members[0].0.as_str() {
        "task" => "task",
        "message" => "message",
        "statusUpdate" => "status",
        "artifactUpdate" => "artifact",
        other => panic!("not a StreamResponse member: {other}"),
    }
}

fn state(frame: &Value) -> String {
    match kind(frame) {
        "task" => text(frame, &["task", "status", "state"]),
        "status" => text(frame, &["statusUpdate", "status", "state"]),
        _ => String::new(),
    }
}

#[test]
fn send_streaming_message_streams_the_task_then_its_updates_until_it_completes() {
    let server = echo();
    let reply = server.call(
        "POST",
        "/message:stream",
        Some(&message("hello", None, None)),
    );
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert_eq!(reply.header("content-type"), Some("text/event-stream"));
    for event in reply.body.split("\n\n").filter(|event| !event.is_empty()) {
        assert!(
            event.starts_with("data: ") && !event.contains('\n'),
            "an event is exactly one `data:` line: {event:?}"
        );
    }
    let frames = reply.frames();
    let kinds: Vec<_> = frames.iter().map(kind).collect();
    assert_eq!(
        kinds,
        ["task", "status", "artifact", "status"],
        "{}",
        reply.body
    );
    assert_eq!(state(&frames[0]), "TASK_STATE_SUBMITTED");
    assert_eq!(state(&frames[1]), "TASK_STATE_WORKING");
    assert_eq!(
        text(
            &frames[2],
            &["artifactUpdate", "artifact", "parts", "0", "text"]
        ),
        "hello"
    );
    assert_eq!(
        at(&frames[2], &["artifactUpdate", "lastChunk"]),
        Some(&Value::Bool(true))
    );
    assert_eq!(state(&frames[3]), "TASK_STATE_COMPLETED");
    let task = text(&frames[0], &["task", "id"]);
    for frame in &frames[1..] {
        let update = at(frame, &["statusUpdate"])
            .or_else(|| at(frame, &["artifactUpdate"]))
            .expect("an update");
        assert_eq!(text(update, &["taskId"]), task);
        assert!(!text(update, &["contextId"]).is_empty());
    }
    let after = server.call("GET", &format!("/tasks/{task}"), None).json();
    assert_eq!(text(&after, &["status", "state"]), "TASK_STATE_COMPLETED");
}

/// One command, one instant: the status a step stores and the `TaskStatusUpdateEvent` it
/// publishes carry the same timestamp, so a client that streams and then reads the task sees one
/// status, not two.
#[test]
fn a_streamed_status_carries_the_timestamp_the_task_then_shows() {
    let server = echo();
    let reply = server.call(
        "POST",
        "/message:stream",
        Some(&message("hello", None, None)),
    );
    assert_eq!(reply.status, 200, "{}", reply.body);
    let frames = reply.frames();
    let task = text(&frames[0], &["task", "id"]);
    let last = frames.last().expect("frames");
    assert_eq!(state(last), "TASK_STATE_COMPLETED", "{}", reply.body);
    let streamed = text(last, &["statusUpdate", "status", "timestamp"]);
    let stored = server.call("GET", &format!("/tasks/{task}"), None).json();
    assert!(!streamed.is_empty(), "{}", reply.body);
    assert_eq!(
        streamed,
        text(&stored, &["status", "timestamp"]),
        "the stream's last status and the stored status differ in time"
    );
}

#[test]
fn send_streaming_message_closes_at_an_interrupted_state() {
    let server = echo();
    let reply = server.call("POST", "/message:stream", Some(&message("", None, None)));
    assert_eq!(reply.status, 200, "{}", reply.body);
    let frames = reply.frames();
    let last = frames.last().expect("frames");
    assert_eq!(state(last), "TASK_STATE_INPUT_REQUIRED", "{}", reply.body);
    assert!(
        !text(
            last,
            &["statusUpdate", "status", "message", "parts", "0", "text"]
        )
        .is_empty(),
        "the agent says what input it needs: {}",
        reply.body
    );
}

#[test]
fn send_streaming_message_answered_directly_is_one_message_event() {
    let server = echo();
    let body = "{\"message\":{\"messageId\":\"m-1\",\"role\":\"ROLE_USER\",\"parts\":[{\"text\":\"hi\"}],\"metadata\":{\"mode\":\"message\"}}}";
    let reply = server.call("POST", "/message:stream", Some(body));
    assert_eq!(reply.status, 200, "{}", reply.body);
    let frames = reply.frames();
    assert_eq!(frames.len(), 1, "{}", reply.body);
    assert_eq!(kind(&frames[0]), "message");
    assert_eq!(text(&frames[0], &["message", "role"]), "ROLE_AGENT");
}

#[test]
fn send_streaming_message_without_the_capability_is_unsupported_operation() {
    let server = echo_with(card(|card| card.capabilities.streaming = Some(false)));
    let reply = server.call(
        "POST",
        "/message:stream",
        Some(&message("hello", None, None)),
    );
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.reason(), "UNSUPPORTED_OPERATION");
}

#[test]
fn send_streaming_message_errors_answer_before_the_stream_opens() {
    let server = echo();
    let reply = server.call(
        "POST",
        "/message:stream",
        Some(&message("hello", Some("no-such-task"), None)),
    );
    assert_eq!(reply.status, 404, "{}", reply.body);
    assert_eq!(reply.header("content-type"), Some("application/a2a+json"));
    assert_eq!(reply.reason(), "TASK_NOT_FOUND");
}

#[test]
fn subscribe_to_task_streams_the_task_then_what_another_request_does_until_it_completes() {
    let server = echo();
    let (task, context) = server.waiting_task();
    let stream = server.send(
        "GET",
        &format!("/tasks/{task}:subscribe"),
        None,
        &[("A2A-Version", "1.0")],
    );
    let mut reader = BufReader::new(stream);
    let (status, headers) = read_head(&mut reader);
    assert_eq!(status, 200);
    assert!(
        headers
            .iter()
            .any(|(name, value)| name.eq_ignore_ascii_case("content-type")
                && value == "text/event-stream"),
        "{headers:?}"
    );
    let first = next_frame(&mut reader).expect("the task first");
    assert_eq!(kind(&first), "task");
    assert_eq!(state(&first), "TASK_STATE_INPUT_REQUIRED");

    let continued = server.call(
        "POST",
        "/message:send",
        Some(&message("now", Some(&task), Some(&context))),
    );
    assert_eq!(continued.status, 200, "{}", continued.body);

    let mut kinds = Vec::new();
    let mut last = None;
    while let Some(frame) = next_frame(&mut reader) {
        kinds.push(kind(&frame));
        last = Some(frame);
    }
    assert_eq!(kinds, ["status", "artifact", "status"]);
    assert_eq!(state(&last.expect("a last frame")), "TASK_STATE_COMPLETED");
}

#[test]
fn subscribe_to_task_sees_a_cancel() {
    let server = echo();
    let (task, _) = server.waiting_task();
    let stream = server.send(
        "GET",
        &format!("/tasks/{task}:subscribe"),
        None,
        &[("A2A-Version", "1.0")],
    );
    let mut reader = BufReader::new(stream);
    let (status, _) = read_head(&mut reader);
    assert_eq!(status, 200);
    let _ = next_frame(&mut reader).expect("the task first");
    let canceled = server.call("POST", &format!("/tasks/{task}:cancel"), None);
    assert_eq!(canceled.status, 200, "{}", canceled.body);
    let frame = next_frame(&mut reader).expect("the cancel");
    assert_eq!(state(&frame), "TASK_STATE_CANCELED");
    assert!(
        next_frame(&mut reader).is_none(),
        "the stream closes at a terminal state"
    );
}

#[test]
fn subscribe_to_a_terminal_task_is_unsupported_operation() {
    let server = echo();
    let task = server.completed_task();
    let reply = server.call("GET", &format!("/tasks/{task}:subscribe"), None);
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.reason(), "UNSUPPORTED_OPERATION");
}

#[test]
fn subscribe_to_an_unknown_task_is_task_not_found() {
    let server = echo();
    let reply = server.call("GET", "/tasks/no-such-task:subscribe", None);
    assert_eq!(reply.status, 404, "{}", reply.body);
    assert_eq!(reply.reason(), "TASK_NOT_FOUND");
}

#[test]
fn subscribe_without_the_capability_is_unsupported_operation() {
    let server = echo_with(card(|card| card.capabilities.streaming = Some(false)));
    let (task, _) = server.waiting_task();
    let reply = server.call("GET", &format!("/tasks/{task}:subscribe"), None);
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.reason(), "UNSUPPORTED_OPERATION");
}
