//! Acceptance 2 and 4: `a2a-testkit run --url <base> --suite <suite.json> [--results <out>]`
//! against `a2a-server` with the echo agent on port 0, the suite synthesized with the `ess` on
//! `PATH` into this test target's temporary directory. It prints one line per scenario and the
//! counts, writes `ess-conformance-results/1` with `--results`, and exits 0: no scenario is
//! `failed` or `error`, and GetTask and CancelTask of an unknown id are `passed`. Every assertion
//! helper returns no `Mismatch` against the same server.

mod common;

use a2a::tasks::{
    CreateTaskPushNotificationConfig, GetTask, PushNotificationConfigId, TaskId, WireTaskState,
};
use a2a_client::Reason;
use a2a_conformance::Status;
use a2a_testkit::{
    expect_card, expect_error_reason, expect_push_config_roundtrip, expect_stream_states,
    expect_task_state,
};
use common::{echo, message, printed, results, run, scratch, send, stream, synthesize, testkit};

const GET_TASK: &str = "a2a.tasks.GetTask/outcome/not-found";
const CANCEL_TASK: &str = "a2a.tasks.CancelTask/outcome/not-found";

#[test]
fn the_suite_runs_against_a2a_server_with_no_failed_or_error_scenario() {
    let server = echo();
    let directory = scratch("endpoint");
    let suite = synthesize(&directory);
    let written = directory.join("results.json");

    let output = run(&server.url(), &suite, &written);
    assert_eq!(output.status.code(), Some(0), "{}", printed(&output));

    let results = results(&written);
    assert_eq!(results.status(GET_TASK), Some(Status::Passed), "{GET_TASK}");
    assert_eq!(
        results.status(CANCEL_TASK),
        Some(Status::Passed),
        "{CANCEL_TASK}"
    );
    assert_eq!(results.count(Status::Failed), 0);
    assert_eq!(results.count(Status::Error), 0);
    for result in &results.results {
        if result.status == Status::Unsupported {
            assert!(
                result
                    .message
                    .as_deref()
                    .is_some_and(|reason| !reason.is_empty()),
                "{} is unsupported without a reason",
                result.scenario_id
            );
        }
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        results.results.len() + 1,
        "one line per scenario and the counts: {stdout}"
    );
    for (line, result) in lines.iter().zip(&results.results) {
        assert!(
            line.starts_with(&format!("{} {}", result.status, result.scenario_id)),
            "{line}"
        );
    }
    let counts = format!(
        "{} scenario(s): {} passed, {} failed, {} error, {} unsupported",
        results.results.len(),
        results.count(Status::Passed),
        results.count(Status::Failed),
        results.count(Status::Error),
        results.count(Status::Unsupported)
    );
    assert_eq!(lines.last().copied(), Some(counts.as_str()));
}

#[test]
fn without_results_nothing_is_written_and_the_lines_are_printed() {
    let server = echo();
    let directory = scratch("endpoint-no-results");
    let suite = synthesize(&directory);
    let output = testkit(&[
        "run",
        "--url",
        &server.url(),
        "--suite",
        suite.to_str().expect("a UTF-8 path"),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", printed(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout
            .lines()
            .any(|line| line == format!("passed {GET_TASK}")),
        "{stdout}"
    );
    let entries: Vec<_> = std::fs::read_dir(&directory)
        .expect("the scratch directory")
        .map(|entry| entry.expect("an entry").file_name())
        .collect();
    assert_eq!(entries, ["suite.json"]);
}

#[test]
fn a_scenario_in_error_makes_the_run_exit_non_zero() {
    let closed = std::net::TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = closed.local_addr().expect("bound address");
    drop(closed);
    let directory = scratch("endpoint-closed");
    let suite = synthesize(&directory);
    let written = directory.join("results.json");
    let output = run(&format!("http://{address}"), &suite, &written);
    assert_eq!(output.status.code(), Some(1), "{}", printed(&output));
    let results = results(&written);
    assert_eq!(results.status(GET_TASK), Some(Status::Error));
    assert_eq!(results.count(Status::Failed), 0);
}

#[test]
fn an_https_url_is_refused_as_a2a_client_refuses_it() {
    let directory = scratch("endpoint-https");
    let suite = synthesize(&directory);
    let output = testkit(&[
        "run",
        "--url",
        "https://127.0.0.1:1",
        "--suite",
        suite.to_str().expect("a UTF-8 path"),
    ]);
    assert_ne!(output.status.code(), Some(0), "{}", printed(&output));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unsupported URL scheme `https`"),
        "{stderr}"
    );
    assert!(output.stdout.is_empty(), "{}", printed(&output));
}

#[test]
fn every_helper_holds_against_a2a_server() {
    let server = echo();
    let client = server.client();

    let card = expect_card(&client, "a2a-server echo agent").expect("the card");
    assert_eq!(card.name, "a2a-server echo agent");

    let task = client
        .send_message(&send(message("hello")))
        .expect("the echo agent takes the message")
        .task
        .expect("into a task");
    let read = expect_task_state(
        &client,
        &GetTask {
            tenant: None,
            id: task.id.clone(),
            history_length: None,
        },
        WireTaskState::TASKSTATECOMPLETED,
    )
    .expect("the task is completed");
    assert_eq!(read.id, task.id);

    expect_error_reason(&client, &Reason::TaskNotFound, |client| {
        client.get_task(&GetTask {
            tenant: None,
            id: TaskId("id-unknown".to_owned()),
            history_length: None,
        })
    })
    .expect("an unknown task is TASK_NOT_FOUND");

    let events = expect_stream_states(
        &client,
        &stream(message("streamed")),
        &[
            WireTaskState::TASKSTATESUBMITTED,
            WireTaskState::TASKSTATEWORKING,
            WireTaskState::TASKSTATECOMPLETED,
        ],
    )
    .expect("the stream's states");
    assert_eq!(events.len(), 4, "the task, two updates and the artifact");

    let config = expect_push_config_roundtrip(
        &client,
        &CreateTaskPushNotificationConfig {
            tenant: None,
            id: Some(PushNotificationConfigId("hook-1".to_owned())),
            task_id: task.id,
            url: "https://example.invalid/hook".to_owned(),
            token: Some("token-1".to_owned()),
            authentication: None,
        },
    )
    .expect("the config round trip");
    assert_eq!(
        config.id,
        Some(PushNotificationConfigId("hook-1".to_owned()))
    );
}
