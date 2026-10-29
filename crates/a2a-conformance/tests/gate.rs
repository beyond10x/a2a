//! The gate `task conform` holds a run to: the report has 0 failed and 0 error, and the scenarios
//! that came out `unsupported` are exactly those `unsupported.tsv` names, each with the reason the
//! run gave.

use a2a_conformance::{Results, ScenarioResult, Status, gate};

fn result(id: &str, status: Status, message: Option<&str>) -> ScenarioResult {
    ScenarioResult {
        scenario_id: id.to_owned(),
        status,
        message: message.map(str::to_owned),
    }
}

/// A run of three scenarios: `a/pass` passed, `a/far` and `a/near` unsupported.
fn run() -> Results {
    Results {
        completed_at: 1,
        results: vec![
            result("a/pass", Status::Passed, None),
            result("a/far", Status::Unsupported, Some("the target is far")),
            result("a/near", Status::Unsupported, Some("the target is near")),
        ],
    }
}

/// The report `ess verify conform report` writes for a run: counts and the scenarios by status.
fn report(failed: &[&str], error: &[&str], unsupported: &[&str], passed: &[&str]) -> String {
    let list = |ids: &[&str]| {
        let quoted: Vec<String> = ids.iter().map(|id| format!("\"{id}\"")).collect();
        format!("[{}]", quoted.join(","))
    };
    let total = failed.len() + error.len() + unsupported.len() + passed.len();
    format!(
        r#"{{"format":"ess-conformance-report/2","conformance_status":"failed","counts":{{"error":{},"failed":{},"passed":{},"skipped":0,"total":{total},"unsupported":{}}},"outcomes":{{"error":{},"failed":{},"passed":{},"skipped":[],"unsupported":{}}}}}"#,
        error.len(),
        failed.len(),
        passed.len(),
        unsupported.len(),
        list(error),
        list(failed),
        list(passed),
        list(unsupported),
    )
}

fn clean() -> String {
    report(&[], &[], &["a/far", "a/near"], &["a/pass"])
}

const ALLOWED: &str = "a/far\tthe target is far\na/near\tthe target is near\n";

fn refused(outcome: Result<String, String>) -> String {
    match outcome {
        Ok(summary) => panic!("the gate let the run through: {summary}"),
        Err(refusal) => refusal,
    }
}

#[test]
fn a_run_with_no_failure_and_every_unsupported_scenario_named_passes() {
    let summary = gate::hold(&clean(), &run(), ALLOWED).expect("the run passes the gate");
    assert!(summary.contains("1 passed"), "{summary}");
    assert!(summary.contains("2 unsupported"), "{summary}");
}

#[test]
fn a_failed_or_error_scenario_fails_the_gate() {
    let failed = Results {
        completed_at: 1,
        results: vec![
            result("a/pass", Status::Failed, Some("it broke")),
            result("a/far", Status::Unsupported, Some("the target is far")),
            result("a/near", Status::Unsupported, Some("the target is near")),
        ],
    };
    let refusal = refused(gate::hold(
        &report(&["a/pass"], &[], &["a/far", "a/near"], &[]),
        &failed,
        ALLOWED,
    ));
    assert!(
        refusal.contains("1 failed") && refusal.contains("a/pass"),
        "{refusal}"
    );

    let errored = Results {
        completed_at: 1,
        results: vec![
            result("a/pass", Status::Error, Some("it could not run")),
            result("a/far", Status::Unsupported, Some("the target is far")),
            result("a/near", Status::Unsupported, Some("the target is near")),
        ],
    };
    let refusal = refused(gate::hold(
        &report(&[], &["a/pass"], &["a/far", "a/near"], &[]),
        &errored,
        ALLOWED,
    ));
    assert!(
        refusal.contains("1 error") && refusal.contains("a/pass"),
        "{refusal}"
    );
}

#[test]
fn an_unsupported_scenario_the_file_does_not_name_fails_the_gate() {
    let refusal = refused(gate::hold(&clean(), &run(), "a/far\tthe target is far\n"));
    assert!(
        refusal.contains("a/near") && refusal.contains("the target is near"),
        "{refusal}"
    );
}

#[test]
fn a_reason_other_than_the_one_the_run_gave_fails_the_gate() {
    let refusal = refused(gate::hold(
        &clean(),
        &run(),
        "a/far\tthe target is far\na/near\tthe target is close\n",
    ));
    assert!(
        refusal.contains("a/near")
            && refusal.contains("the target is close")
            && refusal.contains("the target is near"),
        "{refusal}"
    );
}

#[test]
fn a_named_scenario_that_was_not_unsupported_fails_the_gate() {
    let refusal = refused(gate::hold(
        &clean(),
        &run(),
        &format!("{ALLOWED}a/pass\tit is not there\n"),
    ));
    assert!(
        refusal.contains("a/pass") && refusal.contains("passed"),
        "{refusal}"
    );
    let refusal = refused(gate::hold(
        &clean(),
        &run(),
        &format!("{ALLOWED}a/gone\tit is not in the suite\n"),
    ));
    assert!(refusal.contains("a/gone"), "{refusal}");
}

#[test]
fn a_file_line_without_a_reason_or_named_twice_is_refused() {
    let refusal = refused(gate::hold(
        &clean(),
        &run(),
        "a/far\na/near\tthe target is near\n",
    ));
    assert!(refusal.contains("line 1"), "{refusal}");
    let refusal = refused(gate::hold(
        &clean(),
        &run(),
        &format!("{ALLOWED}a/far\tthe target is far\n"),
    ));
    assert!(
        refusal.contains("line 3") && refusal.contains("a/far"),
        "{refusal}"
    );
}

#[test]
fn a_report_that_is_not_of_these_results_is_refused() {
    let refusal = refused(gate::hold(
        &report(&[], &[], &["a/far"], &["a/pass", "a/near"]),
        &run(),
        ALLOWED,
    ));
    assert!(refusal.contains("a/near"), "{refusal}");
    let refusal = refused(gate::hold(
        &clean().replace("ess-conformance-report/2", "ess-conformance-report/1"),
        &run(),
        ALLOWED,
    ));
    assert!(refusal.contains("ess-conformance-report/2"), "{refusal}");
}
