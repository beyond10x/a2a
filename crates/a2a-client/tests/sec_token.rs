//! Security review: the bearer token leaves the client only in the `Authorization` header of the
//! requests of operations under the base path, to the configured origin. Not with the public
//! card, not to a redirect target, not in `Debug` or error text, and only in the RFC 6750
//! `b64token` form the server side (`a2a_server::BearerToken`) accepts.

mod common;

use std::net::SocketAddr;

use a2a::discovery::GetExtendedAgentCard;
use a2a::tasks::{
    CancelTask, CreateTaskPushNotificationConfig, DeleteTaskPushNotificationConfig, GetTask,
    GetTaskPushNotificationConfig, ListTaskPushNotificationConfigs, PushNotificationConfigId,
    SubscribeToTask, TaskId,
};
use a2a_client::{Client, Error, ListTasksRequest};
use common::{answer, answering, client_of, message, send, stream};

const TOKEN: &str = "s3cr3t-T0ken.value";

/// Calls every operation but `GetExtendedAgentCard` once, ignoring what each answers.
fn every_other_operation(client: &Client) {
    let task = TaskId("t-1".to_owned());
    let config = PushNotificationConfigId("c-1".to_owned());
    let _ = client.agent_card();
    let _ = client.send_message(&send(message("hello"), None));
    let _ = client
        .send_streaming_message(&stream(message("hello"), None))
        .map(|events| events.count());
    let _ = client.get_task(&GetTask {
        tenant: None,
        id: task.clone(),
        history_length: None,
    });
    let _ = client.list_tasks(&ListTasksRequest::default());
    let _ = client.cancel_task(&CancelTask {
        tenant: None,
        id: task.clone(),
        metadata: None,
    });
    let _ = client
        .subscribe_to_task(&SubscribeToTask {
            tenant: None,
            id: task.clone(),
        })
        .map(|events| events.count());
    let _ = client.create_task_push_notification_config(&CreateTaskPushNotificationConfig {
        tenant: None,
        id: None,
        task_id: task.clone(),
        url: "http://127.0.0.1:9/hook".to_owned(),
        token: None,
        authentication: None,
    });
    let _ = client.get_task_push_notification_config(&GetTaskPushNotificationConfig {
        tenant: None,
        task_id: task.clone(),
        id: config.clone(),
    });
    let _ = client.list_task_push_notification_configs(&ListTaskPushNotificationConfigs {
        tenant: None,
        task_id: task.clone(),
        page_size: None,
        page_token: None,
    });
    let _ = client.delete_task_push_notification_config(&DeleteTaskPushNotificationConfig {
        tenant: None,
        task_id: task,
        id: config,
    });
}

#[test]
fn the_bearer_token_is_sent_with_every_operation_but_not_the_public_card() {
    const CARD: &str = "/.well-known/agent-card.json";
    let (address, seen) = answering(answer(404, "application/json", "{}"));
    let client = client_of(address).with_bearer(TOKEN);
    let _ = client.extended_agent_card(&GetExtendedAgentCard { tenant: None });
    every_other_operation(&client);
    let seen = seen.lock().expect("the list");
    assert_eq!(seen.len(), 12, "every operation sent one request");
    assert!(
        seen[0].path == "/extendedAgentCard",
        "the first request is GetExtendedAgentCard: {}",
        seen[0].head
    );
    let wrong: Vec<String> = seen
        .iter()
        .filter(|request| request.head.contains(TOKEN) == (request.path == CARD))
        .map(|request| format!("{} {}", request.method, request.path))
        .collect();
    assert!(
        wrong.is_empty() && seen.iter().any(|request| request.path == CARD),
        "story:client: the bearer token goes with every operation under the base path and never \
         with the public card {CARD}; these requests broke that: {wrong:?}"
    );
}

/// A socket that answers every connection with a redirect to `location`.
fn redirecting(status: u16, location: &str) -> SocketAddr {
    let answer = format!(
        "HTTP/1.1 {status} Moved\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    answering(answer.into_bytes()).0
}

#[test]
fn the_bearer_token_is_not_followed_to_a_redirect_target() {
    let (elsewhere, reached) = answering(answer(200, "application/json", "{}"));
    for status in [301, 302, 303, 307, 308] {
        let from = redirecting(status, &format!("http://{elsewhere}/extendedAgentCard"));
        let answered = client_of(from)
            .with_bearer(TOKEN)
            .extended_agent_card(&GetExtendedAgentCard { tenant: None });
        assert!(answered.is_err(), "{status}: {answered:?}");
        let text = format!(
            "{answered:?} {}",
            answered
                .as_ref()
                .err()
                .map_or(String::new(), ToString::to_string)
        );
        assert!(
            !text.contains(TOKEN),
            "{status}: the token is in the error: {text}"
        );
    }
    assert!(
        reached.lock().expect("the list").is_empty(),
        "a redirect target was contacted"
    );
}

#[test]
fn the_bearer_token_never_appears_in_debug_or_error_text() {
    let client = Client::new("http://127.0.0.1:9")
        .expect("a base URL")
        .with_bearer(TOKEN);
    let shown = format!("{client:?}");
    assert!(!shown.contains(TOKEN), "Debug shows the token: {shown}");

    let refused = Client::new("http://127.0.0.1:9")
        .expect("a base URL")
        .with_bearer(&format!("{TOKEN}\r\nX: 1"))
        .extended_agent_card(&GetExtendedAgentCard { tenant: None });
    let error = refused.expect_err("a token with a line break is refused");
    let text = format!("{error} {error:?}");
    assert!(!text.contains(TOKEN), "the refusal shows the token: {text}");

    for answered in [
        answer(
            401,
            "application/json",
            "{\"error\":{\"code\":16,\"message\":\"no\"}}",
        ),
        answer(200, "application/json", "not json"),
        b"garbage\r\n\r\n".to_vec(),
    ] {
        let (address, _) = answering(answered);
        let error = client_of(address)
            .with_bearer(TOKEN)
            .extended_agent_card(&GetExtendedAgentCard { tenant: None })
            .expect_err("an error answer");
        let text = format!("{error} {error:?}");
        assert!(!text.contains(TOKEN), "the error shows the token: {text}");
    }
}

#[test]
fn a_bearer_token_that_is_not_an_rfc_6750_b64token_is_refused_before_anything_is_sent() {
    for token in ["two words", "t\u{f6}ken", "a,b", "\u{85}x"] {
        let (address, seen) = answering(answer(200, "application/json", "{}"));
        let answered = client_of(address)
            .with_bearer(token)
            .extended_agent_card(&GetExtendedAgentCard { tenant: None });
        assert!(
            matches!(answered, Err(Error::InvalidRequest(_))),
            "{token:?} is not a b64token (RFC 6750 section 2.1) and was not refused: {answered:?}"
        );
        assert!(
            seen.lock().expect("the list").is_empty(),
            "{token:?} was sent"
        );
    }
}
