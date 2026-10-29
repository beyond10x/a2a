//! Acceptance 2, streams: `SendStreamingMessage` and `SubscribeToTask` yield the server-sent
//! events in order as typed stream responses, the task first and a terminal or interrupted state
//! last, and the iterator ends when the server closes the stream.

mod common;

use a2a::tasks::{MessageStreamResponse, SubscribeToTask, TaskSubscriptionResponse};
use common::{echo, message, message_of, send, state, stream, task_of, text_part, wire_state};

const TENANTS: [Option<&str>; 2] = [None, Some("acme")];

/// What a stream response carries, and the state it names where it names one.
fn kind_of(
    task: Option<&a2a::tasks::TaskSnapshot>,
    message: bool,
    status: Option<&a2a::tasks::TaskStatusUpdateEvent>,
    artifact: bool,
) -> (&'static str, String) {
    match (task, message, status, artifact) {
        (Some(task), false, None, false) => ("task", state(task)),
        (None, true, None, false) => ("message", String::new()),
        (None, false, Some(status), false) => ("status", wire_state(status.status.state)),
        (None, false, None, true) => ("artifact", String::new()),
        _ => panic!("a stream response holds exactly one member"),
    }
}

fn kind(event: &MessageStreamResponse) -> (&'static str, String) {
    kind_of(
        event.task.as_ref(),
        event.message.is_some(),
        event.status_update.as_ref(),
        event.artifact_update.is_some(),
    )
}

fn subscribed_kind(event: &TaskSubscriptionResponse) -> (&'static str, String) {
    kind_of(
        event.task.as_ref(),
        event.message.is_some(),
        event.status_update.as_ref(),
        event.artifact_update.is_some(),
    )
}

#[test]
fn send_streaming_message_yields_the_task_then_its_updates_until_it_completes() {
    let server = echo();
    let client = server.client();
    for tenant in TENANTS {
        let events: Vec<_> = client
            .send_streaming_message(&stream(message("hello"), tenant))
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"))
            .collect::<Result<_, _>>()
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"));
        let kinds: Vec<_> = events.iter().map(kind).collect();
        assert_eq!(
            kinds,
            [
                ("task", "TASK_STATE_SUBMITTED".to_owned()),
                ("status", "TASK_STATE_WORKING".to_owned()),
                ("artifact", String::new()),
                ("status", "TASK_STATE_COMPLETED".to_owned()),
            ],
            "{tenant:?}"
        );
        let task = events[0].task.as_ref().expect("the task first").id.clone();
        let artifact = events[2].artifact_update.as_ref().expect("an artifact");
        assert_eq!(artifact.task_id, task, "{tenant:?}");
        assert_eq!(
            artifact.artifact.parts[0].text.as_deref(),
            Some("hello"),
            "{tenant:?}"
        );
    }
}

#[test]
fn send_streaming_message_ends_at_an_interrupted_state() {
    let server = echo();
    let client = server.client();
    for tenant in TENANTS {
        let events: Vec<_> = client
            .send_streaming_message(&stream(message(""), tenant))
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"))
            .collect::<Result<_, _>>()
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"));
        assert_eq!(kind(&events[0]).0, "task", "{tenant:?}");
        assert_eq!(
            kind(events.last().expect("events")),
            ("status", "TASK_STATE_INPUT_REQUIRED".to_owned()),
            "{tenant:?}"
        );
    }
}

#[test]
fn subscribe_to_task_yields_the_task_then_what_another_request_does_until_it_completes() {
    let server = echo();
    let client = server.client();
    for tenant in TENANTS {
        let waiting = task_of(&client, &send(message(""), tenant));
        let mut events = client
            .subscribe_to_task(&SubscribeToTask {
                tenant: tenant.map(str::to_owned),
                id: waiting.id.clone(),
            })
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"));
        let first = events
            .next()
            .expect("the task first")
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"));
        assert_eq!(
            subscribed_kind(&first),
            ("task", "TASK_STATE_INPUT_REQUIRED".to_owned()),
            "{tenant:?}"
        );

        let context = waiting
            .context_id
            .as_ref()
            .map(|context| context.0.as_str());
        let continued = message_of(vec![text_part("now")], Some(&waiting.id), context);
        let answered = task_of(&client, &send(continued, tenant));
        assert_eq!(state(&answered), "TASK_STATE_COMPLETED", "{tenant:?}");

        let rest: Vec<_> = events
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"));
        let kinds: Vec<_> = rest.iter().map(subscribed_kind).collect();
        assert_eq!(
            kinds,
            [
                ("status", "TASK_STATE_WORKING".to_owned()),
                ("artifact", String::new()),
                ("status", "TASK_STATE_COMPLETED".to_owned()),
            ],
            "{tenant:?}"
        );
    }
}
