//! Acceptance 3: the assertion helpers over a connected endpoint, `expect_card`,
//! `expect_task_state`, `expect_error_reason`, `expect_stream_states` and
//! `expect_push_config_roundtrip`, each return a `Mismatch` naming the expected and the observed
//! value instead of panicking.

mod common;

use a2a::tasks::{
    CancelTask, CreateTaskPushNotificationConfig, GetTask, PushNotificationConfigId, TaskId,
    WireTaskState,
};
use a2a_client::Reason;
use a2a_server::Settings;
use a2a_testkit::{
    Mismatch, expect_card, expect_error_reason, expect_push_config_roundtrip, expect_stream_states,
    expect_task_state,
};
use common::{answer, echo, echo_with, error_answer, message, recording, send, stream};

fn names(mismatch: &Mismatch, expected: &str, observed: &str) {
    assert!(
        mismatch.expected.contains(expected),
        "expected `{expected}` in {mismatch:?}"
    );
    assert!(
        mismatch.observed.contains(observed),
        "observed `{observed}` in {mismatch:?}"
    );
    let shown = mismatch.to_string();
    assert!(
        shown.contains(&mismatch.expected) && shown.contains(&mismatch.observed),
        "{shown}"
    );
}

fn changed(change: impl FnOnce(&mut a2a::discovery::AgentCard)) -> Settings {
    let mut card = a2a_server::default_card();
    change(&mut card);
    Settings {
        card,
        extended_card: None,
    }
}

fn get(task: &TaskId) -> GetTask {
    GetTask {
        tenant: None,
        id: task.clone(),
        history_length: None,
    }
}

fn config(task: &TaskId) -> CreateTaskPushNotificationConfig {
    CreateTaskPushNotificationConfig {
        tenant: None,
        id: Some(PushNotificationConfigId("hook-1".to_owned())),
        task_id: task.clone(),
        url: "https://example.invalid/hook".to_owned(),
        token: None,
        authentication: None,
    }
}

#[test]
fn expect_card_names_the_name_and_binding_it_expected_and_the_ones_served() {
    let server = echo();
    let mismatch = expect_card(&server.client(), "another agent").expect_err("another name");
    names(&mismatch, "another agent", "a2a-server echo agent");

    let server = echo_with(changed(|card| {
        for interface in &mut card.supported_interfaces {
            "JSONRPC".clone_into(&mut interface.protocol_binding);
        }
    }));
    let mismatch =
        expect_card(&server.client(), "a2a-server echo agent").expect_err("no HTTP+JSON");
    names(&mismatch, "HTTP+JSON", "JSONRPC");

    let (socket, _) = recording(answer(404, "{}"));
    let mismatch = expect_card(&socket.client(), "a2a-server echo agent").expect_err("no card");
    names(&mismatch, "agent card", "404");
}

#[test]
fn expect_task_state_names_the_state_it_expected_and_the_one_held() {
    let server = echo();
    let client = server.client();
    let task = client
        .send_message(&send(message("hello")))
        .expect("the message")
        .task
        .expect("a task");
    let mismatch = expect_task_state(&client, &get(&task.id), WireTaskState::TASKSTATEWORKING)
        .expect_err("the task is completed");
    names(&mismatch, "TASK_STATE_WORKING", "TASK_STATE_COMPLETED");

    let mismatch = expect_task_state(
        &client,
        &get(&TaskId("id-unknown".to_owned())),
        WireTaskState::TASKSTATECOMPLETED,
    )
    .expect_err("no such task");
    names(&mismatch, "TASK_STATE_COMPLETED", "TASK_NOT_FOUND");
}

#[test]
fn expect_error_reason_names_the_reason_it_expected_and_the_one_answered() {
    let server = echo();
    let client = server.client();
    let task = client
        .send_message(&send(message("hello")))
        .expect("the message")
        .task
        .expect("a task");
    let mismatch = expect_error_reason(&client, &Reason::TaskNotFound, |client| {
        client.cancel_task(&CancelTask {
            tenant: None,
            id: task.id.clone(),
            metadata: None,
        })
    })
    .expect_err("a completed task is not cancelable");
    names(&mismatch, "TASK_NOT_FOUND", "TASK_NOT_CANCELABLE");

    let mismatch = expect_error_reason(&client, &Reason::TaskNotFound, |client| {
        client.get_task(&get(&task.id))
    })
    .expect_err("the task is there");
    names(&mismatch, "TASK_NOT_FOUND", "TaskSnapshot");
}

#[test]
fn expect_stream_states_names_the_states_it_expected_and_the_ones_streamed() {
    let server = echo();
    let mismatch = expect_stream_states(
        &server.client(),
        &stream(message("hello")),
        &[
            WireTaskState::TASKSTATESUBMITTED,
            WireTaskState::TASKSTATECOMPLETED,
        ],
    )
    .expect_err("the stream works on the task first");
    names(
        &mismatch,
        "TASK_STATE_SUBMITTED, TASK_STATE_COMPLETED",
        "TASK_STATE_SUBMITTED, TASK_STATE_WORKING, TASK_STATE_COMPLETED",
    );

    let server = echo_with(changed(|card| card.capabilities.streaming = Some(false)));
    let mismatch = expect_stream_states(
        &server.client(),
        &stream(message("hello")),
        &[WireTaskState::TASKSTATESUBMITTED],
    )
    .expect_err("no streaming");
    names(&mismatch, "TASK_STATE_SUBMITTED", "UNSUPPORTED_OPERATION");
}

#[test]
fn expect_push_config_roundtrip_names_the_step_and_value_that_differ() {
    let server = echo_with(changed(|card| {
        card.capabilities.push_notifications = Some(false);
    }));
    let client = server.client();
    let task = client
        .send_message(&send(message("hello")))
        .expect("the message")
        .task
        .expect("a task");
    let mismatch = expect_push_config_roundtrip(&client, &config(&task.id))
        .expect_err("no push notifications");
    names(
        &mismatch,
        "https://example.invalid/hook",
        "PUSH_NOTIFICATION_NOT_SUPPORTED",
    );

    let (socket, _) = recording(answer(
        200,
        r#"{"id":"hook-1","taskId":"t-1","url":"https://other.invalid/hook"}"#,
    ));
    let mismatch =
        expect_push_config_roundtrip(&socket.client(), &config(&TaskId("t-1".to_owned())))
            .expect_err("another url");
    names(
        &mismatch,
        "https://example.invalid/hook",
        "https://other.invalid/hook",
    );

    let (socket, _) = recording(error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND"));
    let mismatch =
        expect_push_config_roundtrip(&socket.client(), &config(&TaskId("t-1".to_owned())))
            .expect_err("no task");
    names(&mismatch, "https://example.invalid/hook", "TASK_NOT_FOUND");
}
