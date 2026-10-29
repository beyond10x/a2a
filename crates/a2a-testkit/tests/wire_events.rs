//! story:testkit-wire-events, acceptance 1 to 5, against stub endpoints: the task events a
//! `WireTarget` reports are derived from what the endpoint answered and nothing else, and the two
//! task views are read back for the tasks the open scenario created.
//!
//! 1. A message answered with a task the scenario had not seen publishes `TaskCreated` with the
//!    answer's id; the task keeps the answer's context and the request's tenant.
//! 2. The status and artifact elements of a stream are `TaskStatusUpdated` and
//!    `TaskArtifactUpdated`, every field as the generated contract names it.
//! 3. An event the answer does not carry, or an outcome whose events the binding has no
//!    correspondence for, stays `unsupported`; a stub answering task-free successes gets no
//!    `passed`.
//! 4. `AllTasks` is each created task read back with GetTask under its tenant, `Tasks` is ListTasks
//!    under the parameter's tenant keeping the created tasks; every other read stays `unsupported`.
//! 5. A push notification configuration command on a task created under another tenant that the
//!    endpoint answers `TaskNotFound` is `unsupported` with the in-process target's reason.

mod adv_support;
mod common;

use std::sync::Arc;

use a2a::json::Value;
use a2a_conformance::{Event, Reader, Status, Target, TargetError};
use a2a_testkit::WireTarget;
use adv_support::{run_against, sse, suite};
use common::{Seen, answer, answering, error_answer, printed, value};

const CLIENT: Option<&str> = Some("a2a.tasks.Client");
const AGENT: Reader<'static> = Reader::Actor("a2a.tasks.Agent");

/// A task in `TASK_STATE_SUBMITTED`, the state a created task starts in.
const SUBMITTED: &str =
    r#"{"id":"t-9","contextId":"c-9","status":{"state":"TASK_STATE_SUBMITTED"}}"#;
/// The same task, moved to `TASK_STATE_WORKING`.
const WORKING: &str = r#"{"id":"t-9","contextId":"c-9","status":{"state":"TASK_STATE_WORKING"}}"#;
/// A task no scenario here creates.
const OTHER: &str =
    r#"{"id":"t-other","contextId":"c-9","status":{"state":"TASK_STATE_SUBMITTED"}}"#;

/// A status update that keeps the task in `TASK_STATE_SUBMITTED` and says why.
const STATUS_SUBMITTED: &str = r#"{"statusUpdate":{"taskId":"t-9","contextId":"c-9","status":{"state":"TASK_STATE_SUBMITTED","message":{"messageId":"s-1","role":"ROLE_AGENT","parts":[{"text":"queued"}]},"timestamp":"2026-10-29T00:00:00.000Z"}}}"#;
const STATUS_WORKING: &str = r#"{"statusUpdate":{"taskId":"t-9","contextId":"c-9","status":{"state":"TASK_STATE_WORKING"}}}"#;
const STATUS_COMPLETED: &str = r#"{"statusUpdate":{"taskId":"t-9","contextId":"c-9","status":{"state":"TASK_STATE_COMPLETED"}}}"#;
const ARTIFACT: &str = r#"{"artifactUpdate":{"taskId":"t-9","contextId":"c-9","artifact":{"artifactId":"a-1","parts":[{"text":"chunk"}]},"lastChunk":true}}"#;
const AGENT_MESSAGE: &str =
    r#"{"messageId":"r-1","role":"ROLE_AGENT","parts":[{"text":"a direct reply"}]}"#;

/// A message input, ESS-named, under `tenant` where one is given.
fn message_input(tenant: Option<&str>) -> Value {
    let tenant = tenant.map_or_else(String::new, |tenant| format!(r#","tenant":"{tenant}""#));
    value(&format!(
        r#"{{"message":{{"message_id":"m-1","context_id":"c-9","role":"ROLE_USER","parts":[]}}{tenant}}}"#
    ))
}

fn target_of(endpoint: &common::Endpoint) -> WireTarget {
    let mut target = WireTarget::new(endpoint.client());
    target.begin("a scenario").expect("a context opens");
    target
}

fn unsupported<T: std::fmt::Debug>(result: Result<T, TargetError>) -> String {
    match result {
        Err(TargetError::Unsupported(reason)) => reason,
        other => panic!("expected an unsupported step, got {other:?}"),
    }
}

fn names(events: &[Event]) -> Vec<&str> {
    events.iter().map(|event| event.name.as_str()).collect()
}

/// The member at the dotted `path` of `value`.
fn at<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.')
        .try_fold(value, |value, member| value.member(member))
}

fn text(value: &str) -> Value {
    Value::Text(value.to_owned())
}

fn targets(seen: &std::sync::Mutex<Vec<Seen>>) -> Vec<String> {
    seen.lock()
        .expect("the list")
        .iter()
        .map(|seen| format!("{} {}", seen.method, seen.target))
        .collect()
}

#[test]
fn a_message_answered_with_a_new_task_publishes_task_created_with_the_answers_id() {
    let (stub, seen) = answering(Arc::new(|method: &str, target: &str| {
        match (method, target) {
            ("POST", "/acme/message:send") => answer(200, &format!(r#"{{"task":{SUBMITTED}}}"#)),
            ("GET", "/acme/tasks/t-9") => answer(200, SUBMITTED),
            _ => error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND"),
        }
    }));
    let mut target = target_of(&stub);
    let answer = target
        .execute(
            "a2a.tasks.SendMessage",
            CLIENT,
            &message_input(Some("acme")),
        )
        .expect("SendMessage runs");
    assert_eq!(answer.outcome, "task-created");
    let published = target.published().expect("TaskCreated is observed");
    assert_eq!(names(&published), ["a2a.tasks.TaskCreated"]);
    assert_eq!(published[0].payload, value(r#"{"task_id":"t-9"}"#));
    assert_eq!(target.published(), Ok(Vec::new()), "reported once");

    let rows = target
        .query("a2a.tasks.AllTasks", AGENT, &value("{}"))
        .expect("AllTasks reads the created task back");
    assert_eq!(rows.len(), 1, "{rows:?}");
    let row = &rows[0];
    assert_eq!(row.member("id"), Some(&text("t-9")));
    assert_eq!(
        row.member("context_id"),
        Some(&text("c-9")),
        "the answer's context"
    );
    assert_eq!(
        row.member("tenant"),
        Some(&text("acme")),
        "the request's tenant"
    );
    assert_eq!(row.member("state"), Some(&text("Submitted")));
    assert!(
        targets(&seen).contains(&"GET /acme/tasks/t-9".to_owned()),
        "read back under the tenant it was created under: {:?}",
        targets(&seen)
    );
}

#[test]
fn a_streamed_message_publishes_task_created_and_its_first_element() {
    let (stub, _) = answering(Arc::new(|method: &str, target: &str| {
        match (method, target) {
            ("POST", "/message:stream") => sse(&[&format!(r#"{{"task":{SUBMITTED}}}"#)]),
            _ => error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND"),
        }
    }));
    let mut target = target_of(&stub);
    let answer = target
        .execute(
            "a2a.tasks.SendStreamingMessage",
            CLIENT,
            &message_input(None),
        )
        .expect("SendStreamingMessage runs");
    assert_eq!(answer.outcome, "task-created");
    let published = target
        .published()
        .expect("the stream's events are observed");
    assert_eq!(
        names(&published),
        ["a2a.tasks.TaskCreated", "a2a.tasks.MessageStreamResponse"]
    );
    assert_eq!(published[0].payload, value(r#"{"task_id":"t-9"}"#));
    assert_eq!(at(&published[1].payload, "task.id"), Some(&text("t-9")));
    assert_eq!(
        at(&published[1].payload, "task.context_id"),
        Some(&text("c-9"))
    );
}

#[test]
fn a_direct_message_derives_no_event() {
    let (stub, _) = answering(Arc::new(|_method: &str, _target: &str| {
        answer(200, &format!(r#"{{"message":{AGENT_MESSAGE}}}"#))
    }));
    let mut target = target_of(&stub);
    let reason = unsupported(target.execute("a2a.tasks.SendMessage", CLIENT, &message_input(None)));
    assert!(reason.contains("direct-message"), "{reason}");
    assert_eq!(target.published(), Ok(Vec::new()));
}

#[test]
fn the_status_and_artifact_elements_of_a_message_stream_are_their_events() {
    let (stub, _) = answering(Arc::new(|method: &str, target: &str| {
        match (method, target) {
            ("POST", "/message:stream") => sse(&[
                &format!(r#"{{"task":{SUBMITTED}}}"#),
                STATUS_SUBMITTED,
                ARTIFACT,
            ]),
            _ => error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND"),
        }
    }));
    let mut target = target_of(&stub);
    target
        .execute(
            "a2a.tasks.SendStreamingMessage",
            CLIENT,
            &message_input(None),
        )
        .expect("SendStreamingMessage runs");
    let published = target
        .published()
        .expect("the stream's events are observed");
    assert_eq!(
        names(&published),
        [
            "a2a.tasks.TaskCreated",
            "a2a.tasks.MessageStreamResponse",
            "a2a.tasks.TaskStatusUpdated",
            "a2a.tasks.TaskArtifactUpdated",
        ]
    );
    let status = &published[2].payload;
    assert_eq!(status.member("task_id"), Some(&text("t-9")));
    assert_eq!(status.member("context_id"), Some(&text("c-9")));
    assert_eq!(
        at(status, "status.state"),
        Some(&text("TASK_STATE_SUBMITTED"))
    );
    assert_eq!(at(status, "status.message.message_id"), Some(&text("s-1")));
    assert_eq!(
        at(status, "status.timestamp"),
        Some(&text("2026-10-29T00:00:00.000Z"))
    );
    let artifact = &published[3].payload;
    assert_eq!(artifact.member("task_id"), Some(&text("t-9")));
    assert_eq!(artifact.member("context_id"), Some(&text("c-9")));
    assert_eq!(at(artifact, "artifact.artifact_id"), Some(&text("a-1")));
    assert_eq!(artifact.member("last_chunk"), Some(&Value::Bool(true)));
}

#[test]
fn the_status_and_artifact_elements_of_a_subscription_are_their_events() {
    let (stub, _) = answering(Arc::new(|method: &str, target: &str| {
        match (method, target) {
            ("GET", "/tasks/t-9:subscribe") => sse(&[
                &format!(r#"{{"task":{WORKING}}}"#),
                ARTIFACT,
                STATUS_WORKING,
            ]),
            _ => error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND"),
        }
    }));
    let mut target = target_of(&stub);
    let answer = target
        .execute(
            "a2a.tasks.SubscribeToTask",
            CLIENT,
            &value(r#"{"id":"t-9"}"#),
        )
        .expect("SubscribeToTask runs");
    assert_eq!(answer.outcome, "subscribed");
    let published = target
        .published()
        .expect("the stream's events are observed");
    assert_eq!(
        names(&published),
        [
            "a2a.tasks.TaskArtifactUpdated",
            "a2a.tasks.TaskStatusUpdated"
        ]
    );
    assert_eq!(
        at(&published[1].payload, "status.state"),
        Some(&text("TASK_STATE_WORKING"))
    );
}

/// A status element that moves the task is a step of the agent (`a2a.tasks.StartWorking`,
/// `a2a.tasks.CompleteTask`, …), a command the wire neither arranges nor tells apart from the one
/// the scenario ran.
#[test]
fn a_stream_element_that_moves_the_task_is_the_agents_step_and_unsupported() {
    let (stub, _) = answering(Arc::new(|method: &str, target: &str| {
        match (method, target) {
            ("POST", "/message:stream") => sse(&[
                &format!(r#"{{"task":{SUBMITTED}}}"#),
                ARTIFACT,
                STATUS_WORKING,
            ]),
            ("GET", "/tasks/t-9:subscribe") => {
                sse(&[&format!(r#"{{"task":{WORKING}}}"#), STATUS_COMPLETED])
            }
            _ => error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND"),
        }
    }));
    let mut target = target_of(&stub);
    target
        .execute(
            "a2a.tasks.SendStreamingMessage",
            CLIENT,
            &message_input(None),
        )
        .expect("SendStreamingMessage runs");
    let reason = unsupported(target.published());
    assert!(
        reason.contains("t-9") && reason.contains("TASK_STATE_WORKING"),
        "{reason}"
    );

    let mut target = target_of(&stub);
    target
        .execute(
            "a2a.tasks.SubscribeToTask",
            CLIENT,
            &value(r#"{"id":"t-9"}"#),
        )
        .expect("SubscribeToTask runs");
    let reason = unsupported(target.published());
    assert!(
        reason.contains("t-9") && reason.contains("TASK_STATE_COMPLETED"),
        "{reason}"
    );
}

#[test]
fn a_canceled_task_answered_is_its_status_update() {
    let canceled = r#"{"id":"t-9","contextId":"c-9","status":{"state":"TASK_STATE_CANCELED","timestamp":"2026-10-29T00:00:00.000Z"}}"#;
    let (stub, _) = answering(Arc::new(move |method: &str, target: &str| {
        match (method, target) {
            ("POST", "/tasks/t-9:cancel") => answer(200, canceled),
            _ => error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND"),
        }
    }));
    let mut target = target_of(&stub);
    let answer = target
        .execute("a2a.tasks.CancelTask", CLIENT, &value(r#"{"id":"t-9"}"#))
        .expect("CancelTask runs");
    assert_eq!(answer.outcome, "canceled");
    let published = target.published().expect("the status update is observed");
    assert_eq!(names(&published), ["a2a.tasks.TaskStatusUpdated"]);
    let update = &published[0].payload;
    assert_eq!(update.member("task_id"), Some(&text("t-9")));
    assert_eq!(update.member("context_id"), Some(&text("c-9")));
    assert_eq!(
        at(update, "status.state"),
        Some(&text("TASK_STATE_CANCELED"))
    );
    assert_eq!(
        at(update, "status.timestamp"),
        Some(&text("2026-10-29T00:00:00.000Z"))
    );
}

#[test]
fn an_outcome_whose_events_the_binding_has_no_correspondence_for_stays_unsupported() {
    let (stub, _) = answering(Arc::new(|method: &str, _target: &str| match method {
        "POST" => answer(
            200,
            r#"{"id":"c-1","taskId":"t-9","url":"https://hook.invalid/"}"#,
        ),
        _ => answer(200, "{}"),
    }));
    let mut target = target_of(&stub);
    target
        .execute(
            "a2a.tasks.CreateTaskPushNotificationConfig",
            CLIENT,
            &value(r#"{"id":"c-1","task_id":"t-9","url":"https://hook.invalid/"}"#),
        )
        .expect("the config is created");
    let reason = unsupported(target.published());
    assert!(
        reason.contains("a2a.tasks.PushNotificationConfigCreated"),
        "{reason}"
    );
    target
        .execute(
            "a2a.tasks.DeleteTaskPushNotificationConfig",
            CLIENT,
            &value(r#"{"id":"c-1","task_id":"t-9"}"#),
        )
        .expect("the config is deleted");
    let reason = unsupported(target.published());
    assert!(
        reason.contains("a2a.tasks.PushNotificationConfigDeleted"),
        "{reason}"
    );
}

#[test]
fn a_stream_element_that_is_no_update_is_unsupported() {
    let (stub, _) = answering(Arc::new(|_method: &str, _target: &str| {
        sse(&[
            &format!(r#"{{"task":{SUBMITTED}}}"#),
            &format!(r#"{{"message":{AGENT_MESSAGE}}}"#),
        ])
    }));
    let mut target = target_of(&stub);
    target
        .execute(
            "a2a.tasks.SendStreamingMessage",
            CLIENT,
            &message_input(None),
        )
        .expect("SendStreamingMessage runs");
    let reason = unsupported(target.published());
    assert!(reason.contains("message"), "{reason}");
}

#[test]
fn a_task_its_agent_moved_before_the_answer_publishes_what_the_answer_does_not_carry() {
    let (stub, _) = answering(Arc::new(|_method: &str, _target: &str| {
        answer(200, &format!(r#"{{"task":{WORKING}}}"#))
    }));
    let mut target = target_of(&stub);
    let answer = target
        .execute("a2a.tasks.SendMessage", CLIENT, &message_input(None))
        .expect("SendMessage runs");
    assert_eq!(answer.outcome, "task-created");
    let reason = unsupported(target.published());
    assert!(
        reason.contains("t-9") && reason.contains("TASK_STATE_WORKING"),
        "{reason}"
    );
}

#[test]
fn a_task_its_agent_moved_since_it_was_observed_makes_the_task_views_unsupported() {
    let (stub, _) = answering(Arc::new(|method: &str, target: &str| {
        match (method, target) {
            ("POST", "/message:send") => answer(200, &format!(r#"{{"task":{SUBMITTED}}}"#)),
            ("GET", "/tasks/t-9") => answer(200, WORKING),
            ("GET", _) => answer(200, &format!(r#"{{"tasks":[{WORKING}]}}"#)),
            _ => error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND"),
        }
    }));
    let mut target = target_of(&stub);
    target
        .execute("a2a.tasks.SendMessage", CLIENT, &message_input(None))
        .expect("SendMessage runs");
    target.published().expect("TaskCreated is observed");
    for (view, reader) in [
        ("a2a.tasks.AllTasks", AGENT),
        ("a2a.tasks.Tasks", Reader::Actor("a2a.tasks.Client")),
    ] {
        let reason = unsupported(target.query(view, reader, &value("{}")));
        assert!(
            reason.contains("t-9") && reason.contains("TASK_STATE_WORKING"),
            "{view}: {reason}"
        );
    }
}

#[test]
fn tasks_is_listed_under_the_parameters_tenant_keeping_the_scenarios_tasks() {
    let (stub, seen) = answering(Arc::new(|method: &str, target: &str| {
        match (method, target) {
            ("POST", "/acme/message:send") => answer(200, &format!(r#"{{"task":{SUBMITTED}}}"#)),
            ("GET", "/acme/tasks/t-9") => answer(200, SUBMITTED),
            ("GET", "/acme/tasks?contextId=c-9&includeArtifacts=true") => answer(
                200,
                &format!(
                    r#"{{"tasks":[{OTHER}],"nextPageToken":"page-2","pageSize":1,"totalSize":2}}"#
                ),
            ),
            ("GET", "/acme/tasks?contextId=c-9&pageToken=page-2&includeArtifacts=true") => answer(
                200,
                &format!(
                    r#"{{"tasks":[{SUBMITTED}],"nextPageToken":"","pageSize":1,"totalSize":2}}"#
                ),
            ),
            ("GET", "/tasks?contextId=c-9&includeArtifacts=true") => {
                answer(200, &format!(r#"{{"tasks":[{OTHER}]}}"#))
            }
            _ => error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND"),
        }
    }));
    let mut target = target_of(&stub);
    target
        .execute(
            "a2a.tasks.SendMessage",
            CLIENT,
            &message_input(Some("acme")),
        )
        .expect("SendMessage runs");
    target.published().expect("TaskCreated is observed");
    let rows = target
        .query(
            "a2a.tasks.Tasks",
            Reader::Actor("a2a.tasks.Client"),
            &value(r#"{"context_id":"c-9","tenant":"acme"}"#),
        )
        .expect("Tasks is listed");
    let ids: Vec<Option<&Value>> = rows.iter().map(|row| row.member("id")).collect();
    assert_eq!(
        ids,
        [Some(&text("t-9"))],
        "only the scenario's task: {rows:?}"
    );
    assert_eq!(rows[0].member("context_id"), Some(&text("c-9")));
    assert_eq!(rows[0].member("state"), Some(&text("Submitted")));
    let listed = targets(&seen);
    assert!(
        listed.contains(
            &"GET /acme/tasks?contextId=c-9&pageToken=page-2&includeArtifacts=true".to_owned()
        ),
        "every page is read: {listed:?}"
    );

    let rows = target
        .query(
            "a2a.tasks.Tasks",
            Reader::Actor("a2a.tasks.Client"),
            &value(r#"{"context_id":"c-9"}"#),
        )
        .expect("Tasks is listed without a tenant");
    assert!(rows.is_empty(), "no task of the scenario: {rows:?}");
    assert_eq!(
        targets(&seen).last().map(String::as_str),
        Some("GET /tasks?contextId=c-9&includeArtifacts=true"),
        "listed without the tenant prefix"
    );
}

#[test]
fn a_view_without_a_read_back_stays_unsupported() {
    let (stub, _) = answering(Arc::new(|_method: &str, _target: &str| {
        error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND")
    }));
    let mut target = target_of(&stub);
    for view in [
        "a2a.tasks.PushNotificationConfigs",
        "a2a.tasks.TaskPushNotificationConfigs",
    ] {
        let reason = unsupported(target.query(view, Reader::Harness, &value("{}")));
        assert!(reason.contains(view), "{reason}");
    }
    for (reader, named) in [
        (Reader::Actor("a2a.tasks.Client"), "a2a.tasks.Client"),
        (Reader::Anonymous, "no actor"),
    ] {
        let reason = unsupported(target.query("a2a.tasks.AllTasks", reader, &value("{}")));
        assert!(
            reason.contains("a2a.tasks.AllTasks") && reason.contains(named),
            "{reason}"
        );
    }
    for (view, reader) in [
        ("a2a.tasks.AllTasks", Reader::Actor("a2a.tasks.Agent")),
        ("a2a.tasks.Tasks", Reader::Actor("a2a.tasks.Client")),
    ] {
        let reason = unsupported(target.query(view, reader, &value("{}")));
        assert!(
            reason.contains(view) && reason.contains("created no task"),
            "{reason}"
        );
    }
}

#[test]
fn a_config_command_on_a_task_of_another_tenant_is_unsupported_as_in_process() {
    let (stub, _) = answering(Arc::new(|method: &str, target: &str| {
        match (method, target) {
            ("POST", "/acme/message:send") => answer(200, &format!(r#"{{"task":{SUBMITTED}}}"#)),
            _ => error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND"),
        }
    }));
    let mut target = target_of(&stub);
    target
        .execute(
            "a2a.tasks.SendMessage",
            CLIENT,
            &message_input(Some("acme")),
        )
        .expect("SendMessage runs");
    target.published().expect("TaskCreated is observed");
    for (command, input) in [
        (
            "a2a.tasks.CreateTaskPushNotificationConfig",
            r#"{"tenant":"other","task_id":"t-9","url":"https://hook.invalid/"}"#,
        ),
        (
            "a2a.tasks.GetTaskPushNotificationConfig",
            r#"{"tenant":"other","task_id":"t-9","id":"c-1"}"#,
        ),
        (
            "a2a.tasks.DeleteTaskPushNotificationConfig",
            r#"{"task_id":"t-9","id":"c-1"}"#,
        ),
    ] {
        let reason = unsupported(target.execute(command, CLIENT, &value(input)));
        assert_eq!(
            reason,
            a2a_conformance::across_tenants(command),
            "{command}"
        );
    }
    let same = target
        .execute(
            "a2a.tasks.CreateTaskPushNotificationConfig",
            CLIENT,
            &value(r#"{"tenant":"acme","task_id":"t-9","url":"https://hook.invalid/"}"#),
        )
        .expect("a configuration under the task's own tenant is read as answered");
    assert_eq!(same.outcome, "task-not-found");
    assert_eq!(same.error.as_deref(), Some("a2a.tasks.TaskNotFound"));
}

/// Every operation answers a success that holds no task where the operation allows one, and an
/// empty object where it does not.
#[test]
fn a_stub_answering_task_free_successes_gets_no_passed() {
    let stub = adv_support::stub(Arc::new(|method: &str, target: &str| {
        let path = target.split('?').next().unwrap_or_default();
        if path.ends_with("/message:send") {
            answer(200, &format!(r#"{{"message":{AGENT_MESSAGE}}}"#))
        } else if path.ends_with("/message:stream") {
            sse(&[&format!(r#"{{"message":{AGENT_MESSAGE}}}"#)])
        } else if path.ends_with("/pushNotificationConfigs") && method == "GET" {
            answer(200, r#"{"configs":[]}"#)
        } else if path.ends_with("/pushNotificationConfigs") {
            answer(
                200,
                r#"{"id":"c-1","taskId":"t-9","url":"https://hook.invalid/"}"#,
            )
        } else if path.ends_with("/tasks") {
            answer(200, r#"{"tasks":[]}"#)
        } else {
            answer(200, "{}")
        }
    }));
    let (output, results) = run_against(&stub.url(), &suite("wire_events"), "wire-events-stub");
    assert_eq!(
        results.count(Status::Passed),
        0,
        "a stub answering no task passes nothing\n{}",
        printed(&output)
    );
    assert_ne!(output.status.code(), Some(0), "{}", printed(&output));
}
