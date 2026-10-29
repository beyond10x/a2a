//! Adversary: the `expect_*` helpers against stub endpoints that answer wrongly. Each test asserts
//! the helper returns a `Mismatch`; a red test is a wrong answer a helper accepts.

mod adv_support;
mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use a2a::tasks::{
    AuthenticationInfo, CreateTaskPushNotificationConfig, GetTask, PushNotificationConfigId,
    TaskId, WireTaskState,
};
use a2a_client::Reason;
use a2a_testkit::{
    expect_error_reason, expect_push_config_roundtrip, expect_stream_states, expect_task_state,
};
use adv_support::{sse, stub};
use common::{answer, error_answer, message, stream};

/// GetTask of `mine` answered with another task in the asked state.
#[test]
fn expect_task_state_names_a_task_other_than_the_one_read() {
    let endpoint = stub(Arc::new(|_method: &str, _target: &str| {
        answer(
            200,
            r#"{"id":"someone-elses","status":{"state":"TASK_STATE_COMPLETED"}}"#,
        )
    }));
    let read = expect_task_state(
        &endpoint.client(),
        &GetTask {
            tenant: None,
            id: TaskId("mine".to_owned()),
            history_length: None,
        },
        WireTaskState::TASKSTATECOMPLETED,
    );
    let mismatch = read.expect_err("the endpoint answered task `someone-elses` for `mine`");
    assert!(mismatch.observed.contains("someone-elses"), "{mismatch}");
}

/// TASK_NOT_FOUND carried on `500 INTERNAL` (A2A v1.0.1 section 5.4: `NOT_FOUND`, `404`).
#[test]
fn expect_error_reason_names_the_wrong_status_of_a_task_not_found() {
    let endpoint = stub(Arc::new(|_method: &str, _target: &str| {
        error_answer(500, "INTERNAL", "TASK_NOT_FOUND")
    }));
    let checked = expect_error_reason(&endpoint.client(), &Reason::TaskNotFound, |client| {
        client.get_task(&GetTask {
            tenant: None,
            id: TaskId("id-unknown".to_owned()),
            history_length: None,
        })
    });
    let mismatch = checked.expect_err("TASK_NOT_FOUND answered as 500 INTERNAL");
    assert!(mismatch.observed.contains("500"), "{mismatch}");
}

/// A stream that never carries its task, whose status updates each belong to a different task.
#[test]
fn expect_stream_states_names_updates_of_other_tasks_and_no_task_first() {
    let endpoint = stub(Arc::new(|_method: &str, _target: &str| {
        sse(&[
            r#"{"statusUpdate":{"taskId":"a","contextId":"c","status":{"state":"TASK_STATE_SUBMITTED"}}}"#,
            r#"{"statusUpdate":{"taskId":"b","contextId":"c","status":{"state":"TASK_STATE_WORKING"}}}"#,
            r#"{"statusUpdate":{"taskId":"c","contextId":"c","status":{"state":"TASK_STATE_COMPLETED"}}}"#,
        ])
    }));
    let streamed = expect_stream_states(
        &endpoint.client(),
        &stream(message("streamed")),
        &[
            WireTaskState::TASKSTATESUBMITTED,
            WireTaskState::TASKSTATEWORKING,
            WireTaskState::TASKSTATECOMPLETED,
        ],
    );
    streamed.expect_err(
        "the stream opens with no task (A2A v1.0.1 section 3.1.2; authored scenario \
         a2a.tasks/authored/stream-starts-with-task) and its updates name three tasks",
    );
}

/// The config served back without the authentication it was created with, and without its task.
#[test]
fn expect_push_config_roundtrip_names_the_authentication_the_endpoint_dropped() {
    let deleted = Arc::new(AtomicBool::new(false));
    let kept = Arc::clone(&deleted);
    let config = r#"{"id":"hook-1","url":"https://hook.invalid/","token":"token-1"}"#;
    let endpoint = stub(Arc::new(move |method: &str, target: &str| {
        let path = target.split('?').next().unwrap_or_default();
        match method {
            "POST" => answer(200, config),
            "DELETE" => {
                kept.store(true, Ordering::SeqCst);
                answer(200, "{}")
            }
            "GET" if path.ends_with("/pushNotificationConfigs") => {
                answer(200, &format!(r#"{{"configs":[{config}]}}"#))
            }
            "GET" if kept.load(Ordering::SeqCst) => {
                error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND")
            }
            _ => answer(200, config),
        }
    }));
    let checked = expect_push_config_roundtrip(
        &endpoint.client(),
        &CreateTaskPushNotificationConfig {
            tenant: None,
            id: Some(PushNotificationConfigId("hook-1".to_owned())),
            task_id: TaskId("task-1".to_owned()),
            url: "https://hook.invalid/".to_owned(),
            token: Some("token-1".to_owned()),
            authentication: Some(AuthenticationInfo {
                scheme: "Bearer".to_owned(),
                credentials: Some("credential-1".to_owned()),
            }),
        },
    );
    assert!(
        deleted.load(Ordering::SeqCst),
        "precondition: the round trip ran to the delete"
    );
    checked.expect_err("the endpoint served the config without its authentication");
}
