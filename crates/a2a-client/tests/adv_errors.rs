//! Adversary, story:client: error answers whose `google.rpc.Status` has unexpected shapes, and an
//! error answered before the request body is read.

mod common;

use a2a::tasks::{GetTask, TaskId};
use a2a_client::{Error, Reason};
use common::{answer, answering, client_of, echo, message_of, send, text_part};

fn get_task() -> GetTask {
    GetTask {
        tenant: None,
        id: TaskId("t-1".to_owned()),
        history_length: None,
    }
}

fn answered_with(status: u16, body: &str) -> Result<a2a::tasks::TaskSnapshot, Error> {
    let (address, _) = answering(answer(status, "application/a2a+json", body));
    client_of(address).get_task(&get_task())
}

const INFO: &str = "\"@type\":\"type.googleapis.com/google.rpc.ErrorInfo\"";

/// Guards the `@type` gate in `status_error`: a detail that is not an `ErrorInfo` carries no
/// reason, even with a `reason` member.
#[test]
fn a_detail_that_is_not_an_error_info_gives_no_reason() {
    let body = "{\"error\":{\"code\":404,\"status\":\"NOT_FOUND\",\"message\":\"m\",\"details\":[{\"@type\":\"type.googleapis.com/google.rpc.BadRequest\",\"reason\":\"TASK_NOT_FOUND\"},{\"reason\":\"TASK_NOT_FOUND\"}]}}";
    match answered_with(404, body) {
        Err(Error::Status { reason, http, .. }) => {
            assert_eq!(http, 404);
            assert_eq!(reason, None);
        }
        other => panic!("{other:?}"),
    }
}

/// Guards "the first `ErrorInfo`": of two, the first one's reason is taken.
#[test]
fn the_first_error_info_s_reason_is_taken() {
    let body = format!(
        "{{\"error\":{{\"code\":400,\"status\":\"FAILED_PRECONDITION\",\"message\":\"m\",\"details\":[{{\"@type\":\"x.BadRequest\"}},{{{INFO},\"reason\":\"TASK_NOT_CANCELABLE\"}},{{{INFO},\"reason\":\"TASK_NOT_FOUND\"}}]}}}}"
    );
    match answered_with(400, &body) {
        Err(Error::Status { reason, .. }) => assert_eq!(reason, Some(Reason::TaskNotCancelable)),
        other => panic!("{other:?}"),
    }
}

/// Unexpected shapes of `details` and of an `ErrorInfo` never panic: the answer stays an
/// `Error::Status` with its HTTP status and code, and no reason.
#[test]
fn unexpected_detail_shapes_keep_the_status_without_a_reason() {
    for details in [
        "",
        ",\"details\":null",
        ",\"details\":{}",
        ",\"details\":\"TASK_NOT_FOUND\"",
        ",\"details\":[null,1,\"x\",[]]",
        ",\"details\":[{\"@type\":5,\"reason\":\"TASK_NOT_FOUND\"}]",
        ",\"details\":[{\"@type\":\"type.googleapis.com/google.rpc.ErrorInfo\",\"reason\":5}]",
        ",\"details\":[{\"@type\":\"type.googleapis.com/google.rpc.ErrorInfo\"}]",
    ] {
        let body = format!("{{\"error\":{{\"code\":409{details}}}}}");
        match answered_with(409, &body) {
            Err(Error::Status {
                http,
                code,
                status,
                message,
                reason,
            }) => {
                assert_eq!((http, code), (409, 409), "{details}");
                assert_eq!((status.as_str(), message.as_str()), ("", ""), "{details}");
                assert_eq!(reason, None, "{details}");
            }
            other => panic!("{details}: {other:?}"),
        }
    }
}

/// Guards the string checks: `error.status` or `error.message` that is not a string, and a code
/// that is not an integer, are no `google.rpc.Status`.
#[test]
fn a_status_with_mistyped_members_is_a_protocol_error() {
    for error in [
        "{\"code\":404,\"status\":5}",
        "{\"code\":404,\"message\":{}}",
        "{\"code\":404.5}",
        "{\"code\":1e3}",
        "{\"code\":99999999999999999999}",
        "{\"code\":null}",
    ] {
        let answered = answered_with(404, &format!("{{\"error\":{error}}}"));
        assert!(
            matches!(answered, Err(Error::Protocol(_))),
            "{error}: {answered:?}"
        );
    }
}

/// An error body that is not UTF-8 (a proxy's Latin-1 page) is a protocol error, not a panic.
#[test]
fn a_non_utf8_error_body_is_a_protocol_error() {
    let mut bytes =
        b"HTTP/1.1 502 Bad Gateway\r\nContent-Type: text/html\r\nContent-Length: 6\r\n\r\n"
            .to_vec();
    bytes.extend_from_slice(&[b'<', 0xE9, 0xFF, 0xFE, b'>', b'\n']);
    let (address, _) = answering(bytes);
    let answered = client_of(address).get_task(&get_task());
    assert!(matches!(answered, Err(Error::Protocol(_))), "{answered:?}");
}

/// a2a-server refuses a body over its 1 MiB bound with 413 after reading only the head; the
/// client, still writing, reads that answer as `Error::Status`, not a transport failure.
#[test]
fn an_oversized_message_is_answered_413_by_a2a_server() {
    let server = echo();
    let big = "x".repeat(2 * 1024 * 1024);
    let answered = server
        .client()
        .send_message(&send(message_of(vec![text_part(&big)], None, None), None));
    match answered {
        Err(Error::Status { http, .. }) => assert_eq!(http, 413),
        other => panic!("{other:?}"),
    }
}
