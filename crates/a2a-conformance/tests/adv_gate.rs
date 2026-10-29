//! Adversary cases on the gate over `unsupported.tsv`: an allowance file that drifts from the run
//! in a way a reader would not see (whitespace, a byte-order mark, a repeated line, a reason cut
//! short) never lets the run through. Attacked, not broken.

use a2a_conformance::{Results, ScenarioResult, Status, gate};

const FAR: &str = "the target is far";

fn run() -> Results {
    Results {
        completed_at: 1,
        results: vec![
            ScenarioResult {
                scenario_id: "a/pass".to_owned(),
                status: Status::Passed,
                message: None,
            },
            ScenarioResult {
                scenario_id: "a/far".to_owned(),
                status: Status::Unsupported,
                message: Some(FAR.to_owned()),
            },
        ],
    }
}

const REPORT: &str = r#"{"format":"ess-conformance-report/2","conformance_status":"failed","counts":{"error":0,"failed":0,"passed":1,"skipped":0,"total":2,"unsupported":1},"outcomes":{"error":[],"failed":[],"passed":["a/pass"],"skipped":[],"unsupported":["a/far"]}}"#;

#[test]
fn the_exact_allowance_passes_and_line_endings_do_not_matter() {
    for allowance in [
        "a/far\tthe target is far\n",
        "a/far\tthe target is far\r\n\r\n",
    ] {
        let held = gate::hold(REPORT, &run(), allowance);
        assert!(held.is_ok(), "{allowance:?}: {held:?}");
    }
}

#[test]
fn an_allowance_that_drifts_invisibly_from_the_run_fails_the_gate() {
    let drifted = [
        ("byte-order mark", "\u{feff}a/far\tthe target is far\n"),
        ("space after the id", "a/far \tthe target is far\n"),
        ("space before the id", " a/far\tthe target is far\n"),
        ("space after the reason", "a/far\tthe target is far \n"),
        ("tab after the reason", "a/far\tthe target is far\t\n"),
        ("reason cut short", "a/far\tthe target is\n"),
        ("reason in another case", "a/far\tThe target is far\n"),
        ("empty reason", "a/far\t\n"),
        ("no tab", "a/far the target is far\n"),
        (
            "the same line twice",
            "a/far\tthe target is far\na/far\tthe target is far\n",
        ),
        (
            "a passed scenario allowed too",
            "a/far\tthe target is far\na/pass\tthe target is far\n",
        ),
        (
            "a scenario the run does not hold",
            "a/far\tthe target is far\na/gone\tthe target is far\n",
        ),
        ("nothing allowed", "\n"),
    ];
    for (what, allowance) in drifted {
        let held = gate::hold(REPORT, &run(), allowance);
        assert!(
            held.is_err(),
            "{what}: the gate let the run through: {held:?}"
        );
    }
}
