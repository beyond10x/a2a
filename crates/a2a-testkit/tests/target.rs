//! Acceptance 1: `WireTarget` implements `a2a_conformance::Target` over `a2a-client` for an
//! endpoint given by base URL. The caller's tenant comes from the scenario's input. A step the wire
//! cannot observe (a published event, a forced external outcome, a command or read by an actor
//! other than the client, a view A2A has no operation for) answers `TargetError::Unsupported` with
//! that reason. story:testkit-wire-events: an event is observed where the answer carries it, and
//! the task views are read where the scenario created a task (`tests/wire_events.rs`); a task the
//! agent moved, and a view read before the scenario created a task, stay unsupported.

mod common;

use a2a::json::Value;
use a2a_conformance::{Reader, Target, TargetError};
use a2a_testkit::WireTarget;
use common::{echo, error_answer, message, recording, send, value};

const CLIENT: Option<&str> = Some("a2a.tasks.Client");

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

#[test]
fn a_refusal_reads_as_its_declared_outcome_and_publishes_nothing() {
    let server = echo();
    let mut target = target_of(&server);
    for command in ["a2a.tasks.GetTask", "a2a.tasks.CancelTask"] {
        let answer = target
            .execute(
                command,
                CLIENT,
                &value(r#"{"id":"id-1048576","tenant":"tenant"}"#),
            )
            .unwrap_or_else(|error| panic!("{command}: {error}"));
        assert_eq!(answer.outcome, "not-found", "{command}");
        assert_eq!(
            answer.error.as_deref(),
            Some("a2a.tasks.TaskNotFound"),
            "{command}"
        );
        assert_eq!(answer.response, None, "{command}");
        assert_eq!(target.published(), Ok(Vec::new()), "{command}");
    }
    target.end().expect("the context closes");
}

#[test]
fn an_accepting_answer_reads_as_its_outcome_with_its_response_spelled_as_ess_spells_it() {
    let server = echo();
    let task = server
        .client()
        .send_message(&send(message("hello")))
        .expect("the echo agent takes the message")
        .task
        .expect("into a task");
    let mut target = target_of(&server);
    let answer = target
        .execute(
            "a2a.tasks.GetTask",
            CLIENT,
            &value(&format!(r#"{{"id":"{}"}}"#, task.id.0)),
        )
        .expect("GetTask runs");
    assert_eq!(answer.outcome, "found");
    assert_eq!(answer.error, None);
    let response = answer.response.expect("found returns the task");
    let read = response.member("task").expect("the response's task");
    assert_eq!(read.member("id"), Some(&Value::Text(task.id.0.clone())));
    assert!(
        read.member("context_id").is_some(),
        "the task is ESS-named: {response:?}"
    );
    assert_eq!(target.published(), Ok(Vec::new()));
}

/// story:testkit-wire-events: the echo agent moves the task it created before SendMessage
/// answers, so what `task-created` publishes is not what the answer carries.
#[test]
fn what_an_accepting_outcome_publishes_is_unsupported_where_the_agent_moved_its_task() {
    let server = echo();
    let mut target = target_of(&server);
    let answer = target
        .execute(
            "a2a.tasks.SendMessage",
            CLIENT,
            &value(
                r#"{"message":{"message_id":"m-1","role":"ROLE_USER","parts":[{"text":"hello"}]}}"#,
            ),
        )
        .expect("SendMessage runs");
    assert_eq!(answer.outcome, "task-created");
    let reason = unsupported(target.published());
    assert!(
        reason.contains("a2a.tasks.SendMessage")
            && reason.contains("task-created")
            && reason.contains("the agent moved it"),
        "{reason}"
    );
}

#[test]
fn a_forced_external_outcome_is_unsupported() {
    let server = echo();
    let mut target = target_of(&server);
    let reason = unsupported(target.force("a2a.tasks.SendMessage", "direct-message"));
    assert!(
        reason.contains("a2a.tasks.SendMessage") && reason.contains("direct-message"),
        "{reason}"
    );
}

#[test]
fn an_outcome_decided_outside_the_request_is_unsupported_when_the_endpoint_takes_it() {
    let server = echo();
    let mut target = target_of(&server);
    let reason = unsupported(target.execute(
        "a2a.discovery.GetExtendedAgentCard",
        Some("a2a.discovery.Client"),
        &value("{}"),
    ));
    assert!(reason.contains("not-supported"), "{reason}");
}

#[test]
fn a_command_the_agent_issues_is_unsupported() {
    let server = echo();
    let mut target = target_of(&server);
    let reason =
        unsupported(target.await_outcome("a2a.tasks.CompleteTask", &Value::Text("t-1".to_owned())));
    assert!(reason.contains("a2a.tasks.CompleteTask"), "{reason}");
    let reason = unsupported(target.execute(
        "a2a.tasks.StartWorking",
        Some("a2a.tasks.Agent"),
        &value(r#"{"id":"t-1"}"#),
    ));
    assert!(reason.contains("a2a.tasks.StartWorking"), "{reason}");
}

#[test]
fn a_command_by_an_actor_other_than_the_client_is_unsupported() {
    let server = echo();
    let mut target = target_of(&server);
    for actor in [Some("a2a.tasks.Agent"), Some("a2a.discovery.Client"), None] {
        let reason = unsupported(target.execute(
            "a2a.tasks.GetTask",
            actor,
            &value(r#"{"id":"id-1048576"}"#),
        ));
        assert!(
            reason.contains(actor.unwrap_or("no actor")),
            "{actor:?}: {reason}"
        );
    }
    let reason = unsupported(target.execute(
        "a2a.discovery.GetExtendedAgentCard",
        Some("a2a.tasks.Agent"),
        &value("{}"),
    ));
    assert!(reason.contains("a2a.tasks.Agent"), "{reason}");
}

/// story:testkit-wire-events: `a2a.tasks.AllTasks` is read back with GetTask
/// (`tests/wire_events.rs`), so it is no longer among these views.
#[test]
fn a_view_a2a_has_no_operation_for_is_unsupported() {
    let server = echo();
    let mut target = target_of(&server);
    for view in [
        "a2a.tasks.PushNotificationConfigs",
        "a2a.tasks.TaskPushNotificationConfigs",
    ] {
        let reason = unsupported(target.query(view, Reader::Harness, &value("{}")));
        assert!(reason.contains(view), "{reason}");
    }
}

#[test]
fn a_read_by_an_actor_other_than_the_client_is_unsupported() {
    let server = echo();
    let mut target = target_of(&server);
    for (reader, named) in [
        (Reader::Actor("a2a.tasks.Agent"), "a2a.tasks.Agent"),
        (
            Reader::Actor("a2a.discovery.Client"),
            "a2a.discovery.Client",
        ),
        (Reader::Anonymous, "no actor"),
    ] {
        let reason = unsupported(target.query("a2a.tasks.Tasks", reader, &value("{}")));
        assert!(reason.contains(named), "{reason}");
    }
}

/// story:testkit-wire-events: the endpoint holds tasks no scenario created and cannot be reset, so
/// `a2a.tasks.Tasks` keeps the scenario's tasks only; before the scenario created one, the read
/// observes nothing the endpoint answered.
#[test]
fn a_view_read_before_the_scenario_created_a_task_is_unsupported() {
    let server = echo();
    let mut target = target_of(&server);
    let reason = unsupported(target.query(
        "a2a.tasks.Tasks",
        Reader::Actor("a2a.tasks.Client"),
        &value(r#"{"context_id":"ctx-1"}"#),
    ));
    assert!(
        reason.contains("a2a.tasks.Tasks") && reason.contains("created no task"),
        "{reason}"
    );
}

#[test]
fn the_tenant_comes_from_the_scenario_input() {
    let (socket, seen) = recording(error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND"));
    let mut target = target_of(&socket);
    let with = target
        .execute(
            "a2a.tasks.GetTask",
            CLIENT,
            &value(r#"{"id":"t-1","tenant":"acme"}"#),
        )
        .expect("GetTask runs");
    let without = target
        .execute("a2a.tasks.GetTask", CLIENT, &value(r#"{"id":"t-1"}"#))
        .expect("GetTask runs");
    assert_eq!(with.outcome, "not-found");
    assert_eq!(without.outcome, "not-found");
    let seen = seen.lock().expect("the list");
    let targets: Vec<&str> = seen.iter().map(|seen| seen.target.as_str()).collect();
    assert_eq!(targets, ["/acme/tasks/t-1", "/tasks/t-1"]);
}
