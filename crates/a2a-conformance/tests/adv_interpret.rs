//! Adversary cases on the interpreter, against a stub target: where a run can come out the same
//! while the target behaves differently.
//!
//! The stub publishes `t.Stamped` with a struct `status` for `t.Stamp`; `t.Restamp` publishes it
//! with one more member in `status`. `t.Create` publishes `t.Created`; `t.Quiet` publishes nothing.

use a2a_conformance::{
    Answer, Awaited, Event, Reader, Results, Status, Suite, Target, TargetError, Value, gate, run,
};

fn json(text: &str) -> Value {
    a2a::json::parse(text).expect("the test value is JSON")
}

#[derive(Default)]
struct Stub {
    pending: Vec<Event>,
}

impl Stub {
    fn publish(&mut self, name: &str, payload: &str) {
        self.pending.push(Event {
            name: name.to_owned(),
            payload: json(payload),
        });
    }
}

impl Target for Stub {
    fn begin(&mut self, _scenario: &str) -> Result<(), TargetError> {
        *self = Self::default();
        Ok(())
    }

    fn execute(
        &mut self,
        command: &str,
        _actor: Option<&str>,
        _input: &Value,
    ) -> Result<Answer, TargetError> {
        let outcome = match command {
            "t.Stamp" => {
                self.publish(
                    "t.Stamped",
                    r#"{"task_id":"x-1","status":{"state":"Open","at":"2026-01-01T00:00:00Z"}}"#,
                );
                "stamped"
            }
            "t.Restamp" => {
                self.publish(
                    "t.Stamped",
                    r#"{"task_id":"x-1","status":{"state":"Open","at":"2026-01-01T00:00:01Z","note":"injected"}}"#,
                );
                "stamped"
            }
            "t.Create" => {
                self.publish("t.Created", r#"{"task_id":"x-1"}"#);
                "created"
            }
            "t.Quiet" => "quiet",
            other => return Err(TargetError::Failed(format!("the stub cannot run {other}"))),
        };
        Ok(Answer {
            outcome: outcome.to_owned(),
            error: None,
            response: None,
        })
    }

    fn query(
        &mut self,
        view: &str,
        _reader: Reader<'_>,
        _params: &Value,
    ) -> Result<Vec<Value>, TargetError> {
        Err(TargetError::Failed(format!(
            "the stub holds no view {view}"
        )))
    }

    fn force(&mut self, command: &str, outcome: &str) -> Result<(), TargetError> {
        Err(TargetError::Failed(format!(
            "the stub cannot force {command} {outcome}"
        )))
    }

    fn published(&mut self) -> Result<Vec<Event>, TargetError> {
        Ok(std::mem::take(&mut self.pending))
    }

    fn await_outcome(&mut self, command: &str, _instance: &Value) -> Result<Awaited, TargetError> {
        Err(TargetError::Unsupported(format!(
            "the stub observes no {command}"
        )))
    }

    fn end(&mut self) -> Result<(), TargetError> {
        Ok(())
    }
}

fn results(scenarios: &str) -> Results {
    let suite = Suite::parse(&format!(
        r#"{{"provenance":{{"suite_version":"ess-conformance/52"}},"events":["t.Created","t.Stamped"],"scenarios":{{{scenarios}}}}}"#
    ))
    .expect("the test suite reads");
    run(&suite, &mut Stub::default())
}

fn command(name: &str) -> String {
    format!(r#"{{"step":"execute_command","command":"{name}","actor":"t.Client"}}"#)
}

/// The report `ess verify conform report` writes for a run that has only unsupported scenarios.
fn report_of_unsupported(ids: &[&str]) -> String {
    let quoted: Vec<String> = ids.iter().map(|id| format!("\"{id}\"")).collect();
    format!(
        r#"{{"format":"ess-conformance-report/2","conformance_status":"failed","counts":{{"error":0,"failed":0,"passed":0,"skipped":0,"total":{},"unsupported":{}}},"outcomes":{{"error":[],"failed":[],"passed":[],"skipped":[],"unsupported":[{}]}}}}"#,
        ids.len(),
        ids.len(),
        quoted.join(",")
    )
}

/// The authored-literal workaround pins its reason in `unsupported.tsv`, so a target that starts
/// carrying another member the literal leaves out is meant to change the reason and fail the
/// gate. The reason a scenario reports is the first unsupported check's, so a member that only a
/// later claim of the scenario carries never reaches it: `a2a.tasks/authored/
/// adv-agent-moves-carry-status` makes five such claims on `TaskStatusUpdated` and is
/// allowed with the reason of the first. Here the second claim's event carries `status.note`, the
/// first does not, and the gate, holding the allowance written for the clean run, still passes.
#[test]
fn a_member_only_a_later_authored_claim_leaves_out_changes_the_allowed_reason() {
    let claim =
        r#"{"step":"expect_event","event":"t.Stamped","payload":{"status":{"state":"Open"}}}"#;
    let (stamp, restamp) = (command("t.Stamp"), command("t.Restamp"));
    let clean = results(&format!(
        r#""t/authored/moves":{{"steps":[{stamp},{claim},{stamp},{claim}]}}"#
    ));
    let drifted = results(&format!(
        r#""t/authored/moves":{{"steps":[{stamp},{claim},{restamp},{claim}]}}"#
    ));
    let id = "t/authored/moves";
    assert_eq!(clean.status(id), Some(Status::Unsupported));
    assert_eq!(drifted.status(id), Some(Status::Unsupported));
    let allowed_reason = clean
        .get(id)
        .and_then(|result| result.message.clone())
        .expect("an unsupported scenario gives a reason");
    let allowance = format!("{id}\t{allowed_reason}\n");
    let held = gate::hold(&report_of_unsupported(&[id]), &drifted, &allowance);
    let drifted_reason = drifted.get(id).and_then(|result| result.message.clone());
    assert!(
        held.is_err(),
        "the gate let a run through whose target carries `status.note` on the second claim's \
         event, under the allowance written for a target that does not; the scenario's reason \
         stayed {drifted_reason:?}"
    );
}

/// Attacked, not broken: an event claim reads the last command's events only, so an event an
/// earlier command published does not satisfy a claim made after a command that published none.
#[test]
fn an_event_from_an_earlier_command_does_not_satisfy_a_later_claim() {
    let (create, quiet) = (command("t.Create"), command("t.Quiet"));
    let claim = r#"{"step":"expect_event","event":"t.Created"}"#;
    let values = r#"{"step":"expect_event_values","event":"t.Created","payload":{}}"#;
    let ran = results(&format!(
        r#""claim":{{"steps":[{create},{quiet},{claim}]}},
        "values":{{"steps":[{create},{quiet},{values}]}}"#
    ));
    assert_eq!(ran.status("claim"), Some(Status::Failed), "{ran:?}");
    assert_eq!(ran.status("values"), Some(Status::Failed), "{ran:?}");
}

/// Attacked, not broken: an outcome claim names its command; the right outcome of another
/// command fails, and a claim on the last command made before any command is not a pass.
#[test]
fn an_outcome_of_another_command_or_of_no_command_does_not_pass() {
    let create = command("t.Create");
    let ran = results(&format!(
        r#""other":{{"steps":[{create},{{"step":"expect_outcome","outcome":{{"command":"t.Quiet","outcome":"created"}}}}]}},
        "none":{{"steps":[{{"step":"expect_no_error"}}]}},
        "no-events":{{"steps":[{{"step":"expect_no_events"}}]}}"#
    ));
    assert_eq!(ran.status("other"), Some(Status::Failed), "{ran:?}");
    assert_eq!(ran.status("none"), Some(Status::Error), "{ran:?}");
    assert_eq!(ran.status("no-events"), Some(Status::Failed), "{ran:?}");
}

/// Attacked, not broken: an absent-event run checks the command before which it stands, so
/// `before: 2` after `t.Create` then `t.Quiet` reads `t.Create`'s events and fails, while
/// `before: 3` reads `t.Quiet`'s and passes.
#[test]
fn an_absent_event_run_reads_the_command_it_follows() {
    let (create, quiet) = (command("t.Create"), command("t.Quiet"));
    let outcome = r#"{"step":"expect_outcome","outcome":{"command":"t.Quiet","outcome":"quiet"}}"#;
    let ran = results(&format!(
        r#""after-create":{{"steps":[{create},{quiet},{outcome}],"absent_events":[{{"before":1,"except":["t.Stamped"]}}]}},
        "after-quiet":{{"steps":[{create},{quiet},{outcome}],"absent_events":[{{"before":3,"except":[]}}]}}"#
    ));
    assert_eq!(ran.status("after-create"), Some(Status::Failed), "{ran:?}");
    assert_eq!(ran.status("after-quiet"), Some(Status::Passed), "{ran:?}");
}
