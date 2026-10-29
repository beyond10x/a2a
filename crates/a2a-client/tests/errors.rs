//! Acceptance 3: every error answer is `Error::Status`, with the HTTP status, the
//! `google.rpc.Status` code, status and message, and the `ErrorInfo` reason as a `Reason`. Each
//! reason `a2a-server` can be made to answer is provoked; the rest are decoded from answer bodies
//! written out here.

mod common;

use std::path::Path;

use a2a::discovery::{AgentExtension, GetExtendedAgentCard};
use a2a::tasks::{
    CancelTask, CreateTaskPushNotificationConfig, GetTask, Message, Part, SubscribeToTask, TaskId,
    TaskSnapshot,
};
use a2a_client::{Error, Reason};
use a2a_server::{Agent, Refusal, Turn};
use common::{
    answer, answering, card, client_of, echo, echo_with, message, message_of, send, start, stream,
    task_of, text_part,
};

/// The HTTP status, `google.rpc.Status` status and reason of an error answer.
fn status_of<T: std::fmt::Debug>(result: Result<T, Error>) -> (u16, String, Option<Reason>) {
    match result {
        Err(Error::Status {
            http,
            code,
            status,
            message,
            reason,
        }) => {
            assert_eq!(code, i64::from(http), "error.code is the HTTP status");
            assert!(!message.is_empty(), "an error answer carries a message");
            (http, status, reason)
        }
        other => panic!("not an error answer: {other:?}"),
    }
}

#[test]
fn an_unknown_task_is_task_not_found() {
    let server = echo();
    let answered = server.client().get_task(&GetTask {
        tenant: None,
        id: TaskId("no-such-task".to_owned()),
        history_length: None,
    });
    assert_eq!(
        status_of(answered),
        (404, "NOT_FOUND".to_owned(), Some(Reason::TaskNotFound))
    );
}

#[test]
fn canceling_a_completed_task_is_task_not_cancelable() {
    let server = echo();
    let client = server.client();
    let task = task_of(&client, &send(message("hello"), None));
    let answered = client.cancel_task(&CancelTask {
        tenant: None,
        id: task.id,
        metadata: None,
    });
    assert_eq!(
        status_of(answered),
        (
            400,
            "FAILED_PRECONDITION".to_owned(),
            Some(Reason::TaskNotCancelable)
        )
    );
}

#[test]
fn a_push_config_without_the_capability_is_push_notification_not_supported() {
    let server = echo_with(card(|card| {
        card.capabilities.push_notifications = Some(false);
    }));
    let client = server.client();
    let task = task_of(&client, &send(message(""), None));
    let answered = client.create_task_push_notification_config(&CreateTaskPushNotificationConfig {
        tenant: None,
        id: None,
        task_id: task.id,
        url: "https://example.invalid/hook".to_owned(),
        token: None,
        authentication: None,
    });
    assert_eq!(
        status_of(answered),
        (
            400,
            "FAILED_PRECONDITION".to_owned(),
            Some(Reason::PushNotificationNotSupported)
        )
    );
}

#[test]
fn subscribing_to_a_completed_task_is_unsupported_operation() {
    let server = echo();
    let client = server.client();
    let task = task_of(&client, &send(message("hello"), None));
    let answered = client.subscribe_to_task(&SubscribeToTask {
        tenant: None,
        id: task.id,
    });
    assert_eq!(
        status_of(answered.map(|_| ())),
        (
            400,
            "FAILED_PRECONDITION".to_owned(),
            Some(Reason::UnsupportedOperation)
        )
    );
}

#[test]
fn streaming_without_the_capability_is_unsupported_operation() {
    let server = echo_with(card(|card| card.capabilities.streaming = Some(false)));
    let answered = server
        .client()
        .send_streaming_message(&stream(message("hello"), None));
    assert_eq!(
        status_of(answered.map(|_| ())),
        (
            400,
            "FAILED_PRECONDITION".to_owned(),
            Some(Reason::UnsupportedOperation)
        )
    );
}

#[test]
fn an_extended_card_declared_but_not_configured_is_extended_agent_card_not_configured() {
    let server = echo_with(card(|card| {
        card.capabilities.extended_agent_card = Some(true);
    }));
    let answered = server
        .client()
        .extended_agent_card(&GetExtendedAgentCard { tenant: None });
    assert_eq!(
        status_of(answered),
        (
            400,
            "FAILED_PRECONDITION".to_owned(),
            Some(Reason::ExtendedAgentCardNotConfigured)
        )
    );
}

#[test]
fn a_request_without_a_required_extension_is_extension_support_required() {
    let uri = "https://example.invalid/ext/trace";
    let server = echo_with(card(|card| {
        card.capabilities.extensions = Some(vec![AgentExtension {
            uri: Some(uri.to_owned()),
            description: None,
            required: Some(true),
            params: None,
        }]);
    }));
    let without = server.client().send_message(&send(message("hello"), None));
    assert_eq!(
        status_of(without),
        (
            400,
            "FAILED_PRECONDITION".to_owned(),
            Some(Reason::ExtensionSupportRequired)
        )
    );
    let with = server
        .client()
        .with_extensions(&["https://example.invalid/ext/other", uri])
        .send_message(&send(message("hello"), None));
    assert!(with.is_ok(), "{with:?}");
}

#[test]
fn a_version_the_agent_does_not_support_is_version_not_supported() {
    let server = echo();
    let answered = server
        .client()
        .with_version("9.9")
        .send_message(&send(message("hello"), None));
    assert_eq!(
        status_of(answered),
        (
            400,
            "FAILED_PRECONDITION".to_owned(),
            Some(Reason::VersionNotSupported)
        )
    );
}

#[test]
fn a_part_of_an_unaccepted_media_type_is_content_type_not_supported() {
    let server = echo();
    let part = Part {
        url: Some("https://example.invalid/a.png".to_owned()),
        media_type: Some("image/png".to_owned()),
        ..text_part("")
    };
    let part = Part { text: None, ..part };
    let answered = server
        .client()
        .send_message(&send(message_of(vec![part], None, None), None));
    assert_eq!(
        status_of(answered),
        (
            400,
            "INVALID_ARGUMENT".to_owned(),
            Some(Reason::ContentTypeNotSupported)
        )
    );
}

#[test]
fn another_context_than_the_task_s_is_context_mismatch() {
    let server = echo();
    let client = server.client();
    let waiting = task_of(&client, &send(message(""), None));
    let continued = message_of(
        vec![text_part("now")],
        Some(&waiting.id),
        Some("another-context"),
    );
    let answered = client.send_message(&send(continued, None));
    assert_eq!(
        status_of(answered),
        (
            400,
            "INVALID_ARGUMENT".to_owned(),
            Some(Reason::ContextMismatch)
        )
    );
}

struct Picky;

impl Agent for Picky {
    fn turn(&mut self, _message: &Message, _task: Option<&TaskSnapshot>) -> Turn {
        Turn::Refuse(Refusal::Context)
    }
}

#[test]
fn a_context_the_agent_cannot_accept_is_context_not_accepted() {
    let server = start(card(|_| {}), || Box::new(Picky));
    let refused = message_of(vec![text_part("hello")], None, Some("foreign-context"));
    let answered = server.client().send_message(&send(refused, None));
    assert_eq!(
        status_of(answered),
        (
            400,
            "INVALID_ARGUMENT".to_owned(),
            Some(Reason::ContextNotAccepted)
        )
    );
}

#[test]
fn an_extended_card_without_the_token_is_unauthenticated_without_a_reason() {
    let mut settings = card(|card| card.capabilities.extended_agent_card = Some(true));
    settings.extended_card = Some(settings.card.clone());
    let server = common::start_with(
        settings,
        a2a_server::Limits::DEFAULT,
        a2a_server::Access {
            extended_card_token: a2a_server::BearerToken::new("t0ken"),
        },
    );
    let answered = server
        .client()
        .with_bearer("wrong")
        .extended_agent_card(&GetExtendedAgentCard { tenant: None });
    assert_eq!(
        status_of(answered),
        (401, "UNAUTHENTICATED".to_owned(), None)
    );
}

/// An error answer with this status and `ErrorInfo` reason, as A2A v1.0.1 section 11.6 writes it.
fn written(http: u16, status: &str, reason: &str) -> Vec<u8> {
    let body = format!(
        "{{\"error\":{{\"code\":{http},\"status\":\"{status}\",\"message\":\"written out\",\"details\":[{{\"@type\":\"type.googleapis.com/google.rpc.ErrorInfo\",\"reason\":\"{reason}\",\"domain\":\"a2a-protocol.org\"}}]}}}}"
    );
    answer(http, "application/a2a+json", &body)
}

#[test]
fn a_reason_the_server_cannot_be_made_to_answer_is_decoded_from_a_written_body() {
    for (http, status, reason, expected) in [
        (
            500,
            "INTERNAL",
            "INVALID_AGENT_RESPONSE",
            Reason::InvalidAgentResponse,
        ),
        (
            400,
            "FAILED_PRECONDITION",
            "SOMETHING_NEW",
            Reason::Other("SOMETHING_NEW".to_owned()),
        ),
    ] {
        let (address, _) = answering(written(http, status, reason));
        let answered = client_of(address).get_task(&GetTask {
            tenant: None,
            id: TaskId("t-1".to_owned()),
            history_length: None,
        });
        assert_eq!(
            status_of(answered),
            (http, status.to_owned(), Some(expected)),
            "{reason}"
        );
    }
}

#[test]
fn every_reason_has_its_wire_name() {
    let all = [
        (Reason::TaskNotFound, "TASK_NOT_FOUND"),
        (Reason::TaskNotCancelable, "TASK_NOT_CANCELABLE"),
        (
            Reason::PushNotificationNotSupported,
            "PUSH_NOTIFICATION_NOT_SUPPORTED",
        ),
        (Reason::UnsupportedOperation, "UNSUPPORTED_OPERATION"),
        (
            Reason::ContentTypeNotSupported,
            "CONTENT_TYPE_NOT_SUPPORTED",
        ),
        (Reason::InvalidAgentResponse, "INVALID_AGENT_RESPONSE"),
        (
            Reason::ExtendedAgentCardNotConfigured,
            "EXTENDED_AGENT_CARD_NOT_CONFIGURED",
        ),
        (
            Reason::ExtensionSupportRequired,
            "EXTENSION_SUPPORT_REQUIRED",
        ),
        (Reason::VersionNotSupported, "VERSION_NOT_SUPPORTED"),
        (Reason::ContextMismatch, "CONTEXT_MISMATCH"),
        (Reason::ContextNotAccepted, "CONTEXT_NOT_ACCEPTED"),
    ];
    for (reason, wire) in all {
        assert_eq!(reason.as_str(), wire);
        assert_eq!(Reason::from(wire), reason);
    }
    assert_eq!(Reason::from("ELSE").as_str(), "ELSE");
}

/// The class, not the instances: every `ErrorInfo` reason `a2a-server`'s router can write decodes
/// to a variant of its own, never to `Reason::Other`.
#[test]
fn every_reason_the_server_answers_has_a_variant() {
    let router = Path::new(env!("CARGO_MANIFEST_DIR")).join("../a2a-server/src/router.rs");
    let text = std::fs::read_to_string(&router).expect("the server's router");
    let mut reasons = Vec::new();
    for piece in text.split("Some(\"").skip(1) {
        let Some((reason, _)) = piece.split_once("\")") else {
            continue;
        };
        if !reason.is_empty()
            && reason
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
        {
            reasons.push(reason.to_owned());
        }
    }
    assert!(
        reasons.len() >= 10,
        "the server's reasons were not read: {reasons:?}"
    );
    for reason in reasons {
        assert!(
            !matches!(Reason::from(reason.as_str()), Reason::Other(_)),
            "a2a-server answers `{reason}`, which has no variant"
        );
    }
}
