//! Acceptance 5: a planted-defect endpoint, `a2a-server` behind a proxy that answers `GetTask` of
//! an unknown id with `200` and an empty task. The CLI exits non-zero with
//! `a2a.tasks.GetTask/outcome/not-found` listed `failed`, and
//! `expect_error_reason(TASK_NOT_FOUND)` returns a `Mismatch`.

mod common;

use a2a::tasks::{GetTask, TaskId};
use a2a_client::Reason;
use a2a_conformance::Status;
use a2a_testkit::expect_error_reason;
use common::{echo, planted, printed, results, run, scratch, synthesize};

const GET_TASK: &str = "a2a.tasks.GetTask/outcome/not-found";

#[test]
fn the_run_fails_on_get_task_answering_an_unknown_id() {
    let server = echo();
    let proxy = planted(server.address);
    let directory = scratch("planted-defect");
    let suite = synthesize(&directory);
    let written = directory.join("results.json");

    let output = run(&proxy.url(), &suite, &written);
    assert_ne!(output.status.code(), Some(0), "{}", printed(&output));
    assert_eq!(output.status.code(), Some(1), "{}", printed(&output));

    let results = results(&written);
    assert_eq!(results.status(GET_TASK), Some(Status::Failed));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with(&format!("failed {GET_TASK}: "))),
        "{stdout}"
    );

    let unplanted = directory.join("unplanted.json");
    let output = run(&server.url(), &suite, &unplanted);
    assert_eq!(output.status.code(), Some(0), "{}", printed(&output));
    assert_eq!(status_of(&unplanted), Some(Status::Passed));
}

fn status_of(path: &std::path::Path) -> Option<Status> {
    results(path).status(GET_TASK)
}

#[test]
fn expect_error_reason_names_the_task_the_defect_answered() {
    let server = echo();
    let proxy = planted(server.address);
    let unknown = GetTask {
        tenant: None,
        id: TaskId("id-unknown".to_owned()),
        history_length: None,
    };

    let mismatch = expect_error_reason(&proxy.client(), &Reason::TaskNotFound, |client| {
        client.get_task(&unknown)
    })
    .expect_err("the defect answers a task");
    assert!(mismatch.expected.contains("TASK_NOT_FOUND"), "{mismatch}");
    assert!(mismatch.observed.contains("TaskSnapshot"), "{mismatch}");

    expect_error_reason(&server.client(), &Reason::TaskNotFound, |client| {
        client.get_task(&unknown)
    })
    .expect("without the proxy an unknown task is TASK_NOT_FOUND");
}
