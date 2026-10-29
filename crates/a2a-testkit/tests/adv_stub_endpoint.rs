//! Adversary: endpoints that implement no A2A behaviour at all, run through the CLI. Each is a
//! stub of a few lines; the run should not exit 0 against it.

mod adv_support;
mod common;

use std::sync::Arc;

use a2a_conformance::Status;
use adv_support::{counts, run_against, stub, suite};
use common::{echo, error_answer, printed};

/// A stub that holds no task and creates none: it refuses every message with
/// VERSION_NOT_SUPPORTED (the client sent `A2A-Version: 1.0`), says it has no extended card, and
/// answers TASK_NOT_FOUND to everything else.
#[test]
fn a_stub_that_creates_no_task_gets_the_reference_servers_verdict() {
    let stub = stub(Arc::new(|_method: &str, target: &str| {
        let path = target.split('?').next().unwrap_or_default();
        if path.ends_with("/extendedAgentCard") {
            error_answer(
                400,
                "FAILED_PRECONDITION",
                "EXTENDED_AGENT_CARD_NOT_CONFIGURED",
            )
        } else if path.ends_with("/message:send") || path.ends_with("/message:stream") {
            error_answer(400, "FAILED_PRECONDITION", "VERSION_NOT_SUPPORTED")
        } else {
            error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND")
        }
    }));
    let suite = suite("adv_stub_endpoint");
    let (stubbed, stubbed_results) = run_against(&stub.url(), &suite, "adv-stub-a");
    let server = echo();
    let (reference, reference_results) = run_against(&server.url(), &suite, "adv-stub-ref");

    let same: Vec<bool> = stubbed_results
        .results
        .iter()
        .zip(&reference_results.results)
        .map(|(stubbed, reference)| stubbed.status == reference.status)
        .collect();
    assert!(
        stubbed_results.count(Status::Passed) > 0,
        "precondition: the stub passes the not-found scenarios"
    );
    assert_ne!(
        stubbed.status.code(),
        Some(0),
        "a stub that refuses every message exits 0 with the verdict a2a-server gets \
         (stub: {}; a2a-server: {}; identical per scenario: {})\n{}",
        counts(&stubbed),
        counts(&reference),
        same.iter().all(|same| *same),
        printed(&stubbed)
    );
}

/// A stub that refuses every operation with EXTENSION_SUPPORT_REQUIRED: no scenario passes, and
/// the run checked nothing.
#[test]
fn a_run_in_which_no_scenario_passed_does_not_exit_0() {
    let stub = stub(Arc::new(|_method: &str, _target: &str| {
        error_answer(400, "FAILED_PRECONDITION", "EXTENSION_SUPPORT_REQUIRED")
    }));
    let (output, results) = run_against(&stub.url(), &suite("adv_stub_endpoint"), "adv-stub-b");
    assert_eq!(
        results.count(Status::Passed),
        0,
        "precondition: nothing passes"
    );
    assert_ne!(
        output.status.code(),
        Some(0),
        "a run with 0 scenarios passed exits 0: {}",
        counts(&output)
    );
}
