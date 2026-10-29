//! Adversary: mutants of `a2a-server` in the area of the seven scenarios the wire observes (the
//! task lookups of an unknown id). Each mutant is a proxy that rewrites the answer to one
//! operation for an id the server does not hold; each test asserts the run reports that
//! operation's not-found scenario `failed` and exits 1. A red test is a wrong endpoint the run
//! lets through.

mod adv_support;
mod common;

use adv_support::{assert_caught, assert_rejected, mutant, run_against, sse, suite};
use common::{answer, echo, error_answer};

const GET_TASK: &str = "a2a.tasks.GetTask/outcome/not-found";
const CANCEL_TASK: &str = "a2a.tasks.CancelTask/outcome/not-found";
const SUBSCRIBE: &str = "a2a.tasks.SubscribeToTask/outcome/not-found";
const GET_CONFIG: &str = "a2a.tasks.GetTaskPushNotificationConfig/outcome/not-found";
const LIST_CONFIGS: &str = "a2a.tasks.ListTaskPushNotificationConfigs/outcome/task-not-found";
const DELETE_CONFIG: &str = "a2a.tasks.DeleteTaskPushNotificationConfig/outcome/task-not-found";

/// The unknown task id the synthesized not-found scenarios name.
const UNKNOWN: &str = "id-1048576";
/// The unknown task id the push-config not-found scenarios name.
const UNKNOWN_TASK: &str = "task_id-1048576";

fn get_task(method: &str, target: &str) -> bool {
    method == "GET" && target.ends_with(&format!("/tasks/{UNKNOWN}"))
}

fn cancel_task(method: &str, target: &str) -> bool {
    method == "POST" && target.ends_with(&format!("/tasks/{UNKNOWN}:cancel"))
}

fn subscribe(method: &str, target: &str) -> bool {
    method == "GET" && target.ends_with(&format!("/tasks/{UNKNOWN}:subscribe"))
}

fn get_config(method: &str, target: &str) -> bool {
    method == "GET"
        && target.ends_with(&format!("/tasks/{UNKNOWN_TASK}/pushNotificationConfigs/id"))
}

fn list_configs(method: &str, target: &str) -> bool {
    method == "GET" && target.ends_with(&format!("/tasks/{UNKNOWN_TASK}/pushNotificationConfigs"))
}

fn delete_config(method: &str, target: &str) -> bool {
    method == "DELETE"
        && target.contains(&format!("/tasks/{UNKNOWN_TASK}/pushNotificationConfigs/"))
}

fn check(name: &str, matches: adv_support::Matches, rewritten: String, scenario: &str) {
    let server = echo();
    let proxy = mutant(server.address, matches, rewritten);
    let (output, results) = run_against(&proxy.url(), &suite("adv_mutants"), name);
    assert_caught(&output, &results, scenario);
}

// Mutants the run catches today: the controls that show the proxy plants what it claims.

#[test]
fn m07_subscribe_to_an_unknown_task_answers_a_stream_is_caught() {
    check(
        "adv-m07",
        subscribe,
        sse(&[&format!(
            r#"{{"task":{{"id":"{UNKNOWN}","status":{{"state":"TASK_STATE_WORKING"}}}}}}"#
        )]),
        SUBSCRIBE,
    );
}

#[test]
fn m08_get_config_of_an_unknown_task_answers_a_config_is_caught() {
    check(
        "adv-m08",
        get_config,
        answer(
            200,
            &format!(r#"{{"id":"id","taskId":"{UNKNOWN_TASK}","url":"https://hook.invalid/"}}"#),
        ),
        GET_CONFIG,
    );
}

#[test]
fn m09_list_configs_of_an_unknown_task_answers_an_empty_list_is_caught() {
    check(
        "adv-m09",
        list_configs,
        answer(200, r#"{"configs":[]}"#),
        LIST_CONFIGS,
    );
}

/// Caught as `error`, not `failed`: the target answers `TargetError::Failed` for an undeclared
/// error, which the interpreter files as the target's failure. The run still exits 1.
#[test]
fn m11_get_config_of_an_unknown_task_answers_404_without_error_info_is_caught() {
    let server = echo();
    let proxy = mutant(
        server.address,
        get_config,
        answer(
            404,
            r#"{"error":{"code":404,"status":"NOT_FOUND","message":"no such route"}}"#,
        ),
    );
    let (output, results) = run_against(&proxy.url(), &suite("adv_mutants"), "adv-m11");
    assert_rejected(&output, &results, GET_CONFIG);
}

// Mutants the run lets through.

/// TASK_NOT_FOUND carried on `500 INTERNAL`: A2A v1.0.1 section 5.4 maps it to `404 NOT_FOUND`.
#[test]
fn m02_get_task_of_an_unknown_id_answers_task_not_found_as_500_internal() {
    check(
        "adv-m02",
        get_task,
        error_answer(500, "INTERNAL", "TASK_NOT_FOUND"),
        GET_TASK,
    );
}

/// An unknown id refused as VERSION_NOT_SUPPORTED, to a request carrying `A2A-Version: 1.0`,
/// while the same endpoint serves the ids it holds.
#[test]
fn m03_get_task_of_an_unknown_id_answers_version_not_supported() {
    check(
        "adv-m03",
        get_task,
        error_answer(400, "FAILED_PRECONDITION", "VERSION_NOT_SUPPORTED"),
        GET_TASK,
    );
}

/// CancelTask of an id nothing holds answers 200 with a canceled task of that id.
#[test]
fn m04_cancel_of_an_unknown_id_answers_a_canceled_task() {
    check(
        "adv-m04",
        cancel_task,
        answer(
            200,
            &format!(r#"{{"id":"{UNKNOWN}","status":{{"state":"TASK_STATE_CANCELED"}}}}"#),
        ),
        CANCEL_TASK,
    );
}

/// CancelTask of an id nothing holds answers TASK_NOT_CANCELABLE: no task, so nothing to be
/// not cancelable.
#[test]
fn m05_cancel_of_an_unknown_id_answers_task_not_cancelable() {
    check(
        "adv-m05",
        cancel_task,
        error_answer(400, "FAILED_PRECONDITION", "TASK_NOT_CANCELABLE"),
        CANCEL_TASK,
    );
}

/// SubscribeToTask of an id nothing holds answers UNSUPPORTED_OPERATION, from an endpoint whose
/// card declares streaming and which streams the tasks it holds.
#[test]
fn m06_subscribe_to_an_unknown_id_answers_unsupported_operation() {
    check(
        "adv-m06",
        subscribe,
        error_answer(400, "FAILED_PRECONDITION", "UNSUPPORTED_OPERATION"),
        SUBSCRIBE,
    );
}

/// DeleteTaskPushNotificationConfig of a task nothing holds answers 200, as if it deleted.
#[test]
fn m10_delete_config_of_an_unknown_task_answers_deleted() {
    check("adv-m10", delete_config, answer(200, "{}"), DELETE_CONFIG);
}
