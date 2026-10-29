//! The interpreter's step handling on its own, against a stub target: for every step kind one
//! scenario that passes and one that does not, a scenario the target cannot carry out, and a step
//! the interpreter does not know.
//!
//! The stub holds one item, `x-1`, in the view `t.Items`, and answers a few commands:
//! `t.Create` (outcome `created`, publishes `t.Created`), `t.Refuse` (outcome `missing`, error
//! `t.Missing`), `t.Bump` (grows the item), `t.Twice` (publishes two `t.Created`), `t.Leak`
//! (publishes, then refuses its caller) and `t.Broken` (the target fails). `t.Nobody` is granted
//! nothing. The system issues `t.Close` on `x-1` only.

use std::collections::BTreeMap;

use a2a_conformance::{
    Answer, Awaited, Event, Reader, Results, Status, Suite, Target, TargetError, Value, run,
};

fn json(text: &str) -> Value {
    a2a::json::parse(text).expect("the test value is JSON")
}

#[derive(Default)]
struct Stub {
    size: u32,
    forced: Option<String>,
    pending: Vec<Event>,
    held: Vec<Value>,
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
        *self = Self {
            size: 2,
            ..Self::default()
        };
        Ok(())
    }

    fn execute(
        &mut self,
        command: &str,
        actor: Option<&str>,
        input: &Value,
    ) -> Result<Answer, TargetError> {
        if command == "t.Leak" {
            self.publish("t.Created", r#"{"task_id":"x-9"}"#);
        }
        match actor {
            Some("t.Nobody") => return Err(TargetError::NotGranted(Some("t.Nobody".to_owned()))),
            None => return Err(TargetError::NotGranted(None)),
            Some(_) => {}
        }
        let answer = |outcome: &str, error: Option<&str>, response: Option<&str>| Answer {
            outcome: outcome.to_owned(),
            error: error.map(str::to_owned),
            response: response.map(json),
        };
        match command {
            "t.Create" => {
                if let Some(outcome) = self.forced.take() {
                    return Ok(answer(&outcome, None, None));
                }
                if !matches!(input, Value::Object(members) if members.is_empty()) {
                    return Err(TargetError::Failed(format!(
                        "t.Create takes no input, and was sent {input:?}"
                    )));
                }
                self.publish("t.Created", r#"{"task_id":"x-1","note":"hi","size":2}"#);
                Ok(answer("created", None, Some(r#"{"id":"x-1","size":2}"#)))
            }
            "t.Refuse" => Ok(answer("missing", Some("t.Missing"), None)),
            "t.Bump" => {
                self.size += 1;
                self.publish("t.Bumped", r#"{"task_id":"x-1"}"#);
                Ok(answer("bumped", None, Some("{}")))
            }
            "t.Stamp" => {
                self.publish(
                    "t.Stamped",
                    r#"{"task_id":"x-1","status":{"state":"Open","at":"2026-01-01T00:00:00Z"}}"#,
                );
                Ok(answer("stamped", None, None))
            }
            "t.Twice" => {
                self.publish("t.Created", r#"{"task_id":"x-1"}"#);
                self.publish("t.Created", r#"{"task_id":"x-2"}"#);
                Ok(answer("created", None, None))
            }
            other => Err(TargetError::Failed(format!("the stub cannot run {other}"))),
        }
    }

    fn query(
        &mut self,
        view: &str,
        reader: Reader<'_>,
        params: &Value,
    ) -> Result<Vec<Value>, TargetError> {
        if view != "t.Items" {
            return Err(TargetError::Failed(format!(
                "the stub holds no view {view}"
            )));
        }
        match reader {
            Reader::Actor("t.Nobody") => {
                return Err(TargetError::NotGranted(Some("t.Nobody".to_owned())));
            }
            Reader::Anonymous => return Err(TargetError::NotGranted(None)),
            Reader::Harness | Reader::Actor(_) => {}
        }
        let row = json(&format!(
            r#"{{"id":"x-1","state":"Open","size":{}}}"#,
            self.size
        ));
        let wanted = params.member("state");
        Ok(std::iter::once(row)
            .chain(self.held.iter().cloned())
            .filter(|row| wanted.is_none_or(|state| row.member("state") == Some(state)))
            .collect())
    }

    fn establish(
        &mut self,
        entity: &str,
        identity: &Value,
        fields: &Value,
        state: &str,
    ) -> Result<(), TargetError> {
        if entity != "t.Item" {
            return Err(TargetError::Unsupported(format!(
                "the stub holds no {entity}"
            )));
        }
        let mut row = String::from("{");
        a2a::json::member(&mut row, "id");
        a2a::json::push_value(&mut row, identity);
        a2a::json::member(&mut row, "state");
        a2a::json::push_text(&mut row, state);
        if let Some(size) = fields.member("size") {
            a2a::json::member(&mut row, "size");
            a2a::json::push_value(&mut row, size);
        }
        row.push('}');
        self.held.push(json(&row));
        Ok(())
    }

    fn force(&mut self, command: &str, outcome: &str) -> Result<(), TargetError> {
        match (command, outcome) {
            ("t.Create", "boom") => {
                self.forced = Some(outcome.to_owned());
                Ok(())
            }
            ("t.Create", "elsewhere") => Err(TargetError::Unsupported(
                "the stub cannot force t.Create elsewhere".to_owned(),
            )),
            _ => Err(TargetError::Failed(format!(
                "the stub has no way to force {command} {outcome}"
            ))),
        }
    }

    fn published(&mut self) -> Result<Vec<Event>, TargetError> {
        Ok(std::mem::take(&mut self.pending))
    }

    fn await_outcome(&mut self, command: &str, instance: &Value) -> Result<Awaited, TargetError> {
        match command {
            "t.Close" if *instance == json(r#""x-1""#) => {
                self.publish("t.Closed", r#"{"task_id":"x-1"}"#);
                Ok(Awaited::Reached {
                    answer: Answer {
                        outcome: "closed".to_owned(),
                        error: None,
                        response: None,
                    },
                    input: json(r#"{"id":"x-1","note":null}"#),
                })
            }
            "t.Close" => Ok(Awaited::NotIssued),
            other => Err(TargetError::Unsupported(format!(
                "the stub observes no {other}"
            ))),
        }
    }

    fn end(&mut self) -> Result<(), TargetError> {
        Ok(())
    }
}

const CATALOG: &str = r#"["t.Bumped","t.Closed","t.Created","t.Stamped"]"#;

fn results(scenarios: &str) -> Results {
    let suite = Suite::parse(&format!(
        r#"{{"provenance":{{"suite_version":"ess-conformance/52"}},"events":{CATALOG},"scenarios":{{{scenarios}}}}}"#
    ))
    .expect("the test suite reads");
    run(&suite, &mut Stub::default())
}

fn statuses(results: &Results) -> BTreeMap<String, Status> {
    results
        .results
        .iter()
        .map(|result| (result.scenario_id.clone(), result.status))
        .collect()
}

fn expect(scenarios: &str, wanted: &[(&str, Status)]) -> Results {
    let results = results(scenarios);
    let got = statuses(&results);
    for (id, status) in wanted {
        assert_eq!(
            got.get(*id),
            Some(status),
            "{id}: {:?}",
            results.get(id).and_then(|result| result.message.clone())
        );
    }
    assert_eq!(got.len(), wanted.len(), "{got:?}");
    results
}

const CREATE: &str = r#"{"step":"execute_command","command":"t.Create","actor":"t.Client"}"#;
const ITEMS: &str = r#"{"step":"query_view","view":"t.Items"}"#;
const CAPTURE: &str = r#"{"step":"capture_instance","instance":"item","entity":"t.Item","event":"t.Created","field":"task_id"}"#;

#[test]
fn execute_command_runs_the_command_or_errs_on_an_input_nothing_bound() {
    let results = expect(
        &format!(
            r#""pass":{{"steps":[{CREATE},{{"step":"expect_outcome","outcome":{{"command":"t.Create","outcome":"created"}}}}]}},
            "fail":{{"steps":[{{"step":"execute_command","command":"t.Create","actor":"t.Client","input":{{"id":{{"kind":"instance","instance":"nope"}}}}}}]}},
            "broken":{{"steps":[{{"step":"execute_command","command":"t.Broken","actor":"t.Client"}}]}}"#
        ),
        &[
            ("pass", Status::Passed),
            ("fail", Status::Error),
            ("broken", Status::Error),
        ],
    );
    let message = results
        .get("fail")
        .and_then(|result| result.message.clone());
    assert!(
        message
            .as_deref()
            .is_some_and(|message| message.contains("no earlier step bound the instance `nope`")),
        "{message:?}"
    );
}

#[test]
fn expect_outcome_holds_the_outcome_taken() {
    expect(
        &format!(
            r#""pass":{{"steps":[{CREATE},{{"step":"expect_outcome","outcome":{{"command":"t.Create","outcome":"created"}}}}]}},
            "fail":{{"steps":[{CREATE},{{"step":"expect_outcome","outcome":{{"command":"t.Create","outcome":"boom"}}}}]}},
            "none":{{"steps":[{{"step":"expect_outcome","outcome":{{"command":"t.Create","outcome":"created"}}}}]}}"#
        ),
        &[
            ("pass", Status::Passed),
            ("fail", Status::Failed),
            ("none", Status::Error),
        ],
    );
}

#[test]
fn expect_error_and_expect_no_error_hold_the_declared_error() {
    let refuse = r#"{"step":"execute_command","command":"t.Refuse","actor":"t.Client"}"#;
    expect(
        &format!(
            r#""error/pass":{{"steps":[{refuse},{{"step":"expect_error","error":"t.Missing"}}]}},
            "error/fail":{{"steps":[{refuse},{{"step":"expect_error","error":"t.Other"}}]}},
            "error/none":{{"steps":[{CREATE},{{"step":"expect_error","error":"t.Missing"}}]}},
            "no-error/pass":{{"steps":[{CREATE},{{"step":"expect_no_error"}}]}},
            "no-error/fail":{{"steps":[{refuse},{{"step":"expect_no_error"}}]}}"#
        ),
        &[
            ("error/pass", Status::Passed),
            ("error/fail", Status::Failed),
            ("error/none", Status::Failed),
            ("no-error/pass", Status::Passed),
            ("no-error/fail", Status::Failed),
        ],
    );
}

#[test]
fn expect_event_holds_the_event_its_literal_payload_and_its_shape() {
    let shape = r#"{"task_id":{"holds":"primitive","kind":"string"},"size":{"holds":"primitive","kind":"integer"},"gone":{"holds":"primitive","kind":"string","optional":true}}"#;
    let wrong = r#"{"note":{"holds":"primitive","kind":"integer"}}"#;
    expect(
        &format!(
            r#""pass":{{"steps":[{CREATE},{{"step":"expect_event","event":"t.Created","payload":{{"note":"hi"}},"shape":{shape}}}]}},
            "fail/shape":{{"steps":[{CREATE},{{"step":"expect_event","event":"t.Created","shape":{wrong}}}]}},
            "fail/value":{{"steps":[{CREATE},{{"step":"expect_event","event":"t.Created","payload":{{"note":"bye"}}}}]}},
            "fail/missing":{{"steps":[{CREATE},{{"step":"expect_event","event":"t.Closed"}}]}}"#
        ),
        &[
            ("pass", Status::Passed),
            ("fail/shape", Status::Failed),
            ("fail/value", Status::Failed),
            ("fail/missing", Status::Failed),
        ],
    );
}

#[test]
fn expect_event_values_resolve_what_earlier_steps_bound() {
    let twice = r#"{"step":"execute_command","command":"t.Twice","actor":"t.Client"}"#;
    let claim = |id: &str| {
        format!(
            r#"{{"step":"expect_event_values","event":"t.Created","payload":{{"task_id":{{"kind":"literal","value":"{id}"}}}}}}"#
        )
    };
    let (x2, x1, x3) = (claim("x-2"), claim("x-1"), claim("x-3"));
    expect(
        &format!(
            r#""pass":{{"steps":[{CREATE},{CAPTURE},{{"step":"expect_event_values","event":"t.Created","payload":{{"task_id":{{"kind":"instance","instance":"item"}},"note":{{"kind":"observed","event":"t.Created","field":"note"}}}}}}]}},
            "fail":{{"steps":[{CREATE},{{"step":"expect_event_values","event":"t.Created","payload":{{"note":{{"kind":"literal","value":"bye"}}}}}}]}},
            "unbound":{{"steps":[{CREATE},{{"step":"expect_event_values","event":"t.Created","payload":{{"note":{{"kind":"awaited","field":"note"}}}}}}]}},
            "claims/pass":{{"steps":[{twice},{x2},{x1}]}},
            "claims/fail":{{"steps":[{twice},{x2},{x1},{x3}]}}"#
        ),
        &[
            ("pass", Status::Passed),
            ("fail", Status::Failed),
            ("unbound", Status::Error),
            ("claims/pass", Status::Passed),
            ("claims/fail", Status::Failed),
        ],
    );
}

#[test]
fn expect_no_event_and_expect_no_events_hold_what_was_not_published() {
    let refuse = r#"{"step":"execute_command","command":"t.Refuse","actor":"t.Client"}"#;
    expect(
        &format!(
            r#""no-event/pass":{{"steps":[{CREATE},{{"step":"expect_no_event","event":"t.Closed"}}]}},
            "no-event/fail":{{"steps":[{CREATE},{{"step":"expect_no_event","event":"t.Created"}}]}},
            "no-events/pass":{{"steps":[{refuse},{{"step":"expect_no_events"}}]}},
            "no-events/fail":{{"steps":[{CREATE},{{"step":"expect_no_events"}}]}}"#
        ),
        &[
            ("no-event/pass", Status::Passed),
            ("no-event/fail", Status::Failed),
            ("no-events/pass", Status::Passed),
            ("no-events/fail", Status::Failed),
        ],
    );
}

#[test]
fn capture_instance_binds_the_published_identity_or_errs() {
    expect(
        &format!(
            r#""pass":{{"steps":[{CREATE},{CAPTURE},{ITEMS},{{"step":"expect_view","view":"t.Items","expectation":{{"expect":"contains","fields":{{"id":{{"kind":"instance","instance":"item"}}}}}}}}]}},
            "fail":{{"steps":[{CREATE},{{"step":"capture_instance","instance":"item","entity":"t.Item","event":"t.Closed","field":"task_id"}}]}}"#
        ),
        &[("pass", Status::Passed), ("fail", Status::Error)],
    );
}

#[test]
fn establish_entity_hands_the_target_the_instance_and_binds_its_identity_once() {
    let setup = |instance: &str, entity: &str, identity: &str| {
        format!(
            r#"{{"step":"establish_entity","instance":"{instance}","entity":"{entity}","identity":"{identity}","fields":{{"size":7}},"state":"Closed"}}"#
        )
    };
    let held = r#"{"step":"expect_view","view":"t.Items","expectation":{"expect":"contains","fields":{"id":{"kind":"instance","instance":"item"},"state":{"kind":"literal","value":"Closed"},"size":{"kind":"literal","value":7}}}}"#;
    let item = setup("item", "t.Item", "x-5");
    let results = expect(
        &format!(
            r#""held":{{"steps":[{item},{ITEMS},{held}]}},
            "unread":{{"steps":[{ITEMS},{item},{held}]}},
            "same-instance":{{"steps":[{item},{}]}},
            "same-identity":{{"steps":[{item},{}]}},
            "other-entity":{{"steps":[{}]}}"#,
            setup("item", "t.Item", "x-6"),
            setup("again", "t.Item", "x-5"),
            setup("item", "t.Other", "x-5"),
        ),
        &[
            ("held", Status::Passed),
            ("unread", Status::Error),
            ("same-instance", Status::Error),
            ("same-identity", Status::Error),
            ("other-entity", Status::Unsupported),
        ],
    );
    for id in ["same-instance", "same-identity"] {
        let message = results
            .get(id)
            .and_then(|result| result.message.clone())
            .unwrap_or_default();
        assert!(
            message.contains("duplicate instance or qualified entity identity"),
            "{id}: {message}"
        );
    }
    assert_eq!(
        results
            .get("other-entity")
            .and_then(|result| result.message.as_deref()),
        Some("the stub holds no t.Other")
    );
}

#[test]
fn expect_not_granted_holds_the_refusal_and_that_nothing_was_published() {
    let nobody = r#"{"step":"execute_command","command":"t.Create","actor":"t.Nobody"}"#;
    let leak = r#"{"step":"execute_command","command":"t.Leak","actor":"t.Nobody"}"#;
    let refused = r#"{"step":"expect_not_granted","actor":"t.Nobody","unpublished":["t.Created"]}"#;
    expect(
        &format!(
            r#""pass":{{"steps":[{nobody},{refused}]}},
            "fail/ran":{{"steps":[{CREATE},{{"step":"expect_not_granted","actor":"t.Client"}}]}},
            "fail/published":{{"steps":[{leak},{refused}]}},
            "anonymous":{{"steps":[{{"step":"execute_command","command":"t.Create","actor":null}},{{"step":"expect_not_granted","actor":null}}]}}"#
        ),
        &[
            ("pass", Status::Passed),
            ("fail/ran", Status::Failed),
            ("fail/published", Status::Failed),
            ("anonymous", Status::Passed),
        ],
    );
}

#[test]
fn read_as_names_the_reader_of_the_views_after_it() {
    let refused = r#"{"step":"expect_not_granted","actor":"t.Nobody"}"#;
    expect(
        &format!(
            r#""pass":{{"steps":[{{"step":"read_as","actor":"t.Nobody"}},{ITEMS},{refused}]}},
            "anonymous":{{"steps":[{{"step":"read_as","actor":null}},{ITEMS},{{"step":"expect_not_granted","actor":null}}]}},
            "fail/refused":{{"steps":[{{"step":"read_as","actor":"t.Nobody"}},{ITEMS},{{"step":"expect_view","view":"t.Items","expectation":{{"expect":"counts","at_least":1}}}}]}},
            "fail/served":{{"steps":[{{"step":"read_as","actor":"t.Client"}},{ITEMS},{refused}]}}"#
        ),
        &[
            ("pass", Status::Passed),
            ("anonymous", Status::Passed),
            ("fail/refused", Status::Failed),
            ("fail/served", Status::Failed),
        ],
    );
}

#[test]
fn query_view_reads_the_rows_with_resolved_params() {
    expect(
        r#""pass":{"steps":[{"step":"query_view","view":"t.Items","params":{"state":{"kind":"literal","value":"Open"}}},{"step":"expect_view","view":"t.Items","expectation":{"expect":"counts","at_least":1,"at_most":1}}]},
            "fail":{"steps":[{"step":"query_view","view":"t.Broken"}]},
            "unbound":{"steps":[{"step":"query_view","view":"t.Items","params":{"state":{"kind":"instance","instance":"nope"}}}]}"#,
        &[
            ("pass", Status::Passed),
            ("fail", Status::Error),
            ("unbound", Status::Error),
        ],
    );
}

#[test]
fn expect_view_holds_contains_excludes_and_counts() {
    let view = |expectation: &str| {
        format!(
            r#"{{"steps":[{ITEMS},{{"step":"expect_view","view":"t.Items","expectation":{expectation}}}]}}"#
        )
    };
    let open =
        r#"{"state":{"kind":"literal","value":"Open"},"size":{"kind":"literal","value":2.0}}"#;
    let closed = r#"{"state":{"kind":"literal","value":"Closed"}}"#;
    expect(
        &format!(
            r#""contains/pass":{},"contains/fail":{},"excludes/pass":{},"excludes/fail":{},
            "counts/pass":{},"counts/fail":{},"counts/empty":{},
            "unread":{{"steps":[{{"step":"expect_view","view":"t.Items","expectation":{{"expect":"counts","at_least":0}}}}]}}"#,
            view(&format!(r#"{{"expect":"contains","fields":{open}}}"#)),
            view(&format!(r#"{{"expect":"contains","fields":{closed}}}"#)),
            view(&format!(r#"{{"expect":"excludes","fields":{closed}}}"#)),
            view(&format!(r#"{{"expect":"excludes","fields":{open}}}"#)),
            view(r#"{"expect":"counts","at_least":1}"#),
            view(r#"{"expect":"counts","at_most":0}"#),
            view(r#"{"expect":"counts"}"#),
        ),
        &[
            ("contains/pass", Status::Passed),
            ("contains/fail", Status::Failed),
            ("excludes/pass", Status::Passed),
            ("excludes/fail", Status::Failed),
            ("counts/pass", Status::Passed),
            ("counts/fail", Status::Failed),
            ("counts/empty", Status::Error),
            ("unread", Status::Error),
        ],
    );
}

#[test]
fn subject_snapshots_hold_one_row_unchanged() {
    let subject = |id: &str| format!(r#"{{"id":{{"kind":"literal","value":"{id}"}}}}"#);
    let snapshot = |id: &str| {
        format!(
            r#"{{"step":"snapshot_subject","view":"t.Items","subject":{}}}"#,
            subject(id)
        )
    };
    let bump = r#"{"step":"execute_command","command":"t.Bump","actor":"t.Client"}"#;
    let unchanged = r#"{"step":"expect_subject_unchanged","view":"t.Items"}"#;
    let absent = |id: &str| {
        format!(
            r#"{{"step":"expect_subject_absent","view":"t.Items","subject":{}}}"#,
            subject(id)
        )
    };
    expect(
        &format!(
            r#""snapshot/pass":{{"steps":[{ITEMS},{},{ITEMS},{unchanged}]}},
            "snapshot/fail":{{"steps":[{ITEMS},{}]}},
            "unchanged/fail":{{"steps":[{ITEMS},{},{bump},{ITEMS},{unchanged}]}},
            "unchanged/none":{{"steps":[{ITEMS},{unchanged}]}},
            "absent/pass":{{"steps":[{ITEMS},{}]}},
            "absent/fail":{{"steps":[{ITEMS},{}]}}"#,
            snapshot("x-1"),
            snapshot("x-9"),
            snapshot("x-1"),
            absent("x-9"),
            absent("x-1"),
        ),
        &[
            ("snapshot/pass", Status::Passed),
            ("snapshot/fail", Status::Failed),
            ("unchanged/fail", Status::Failed),
            ("unchanged/none", Status::Error),
            ("absent/pass", Status::Passed),
            ("absent/fail", Status::Failed),
        ],
    );
}

#[test]
fn complete_subject_snapshots_hold_the_row_to_its_declared_shape() {
    let shape = |size: &str| {
        format!(
            r#"{{"identity_field":"id","fields":[{{"name":"id","type":"t.ItemId"}},{{"name":"state","type":"t.State"}},{{"name":"size","type":"{size}"}},{{"name":"note","type":"Optional<String>"}}],"declarations":{{"t.ItemId":{{"kind":"newtype","of":"String"}},"t.State":{{"kind":"enum","variants":["Open","Closed"]}}}}}}"#
        )
    };
    let snapshot = |size: &str| {
        format!(
            r#"{{"step":"snapshot_complete_subject","view":"t.Items","subject":{{"id":{{"kind":"literal","value":"x-1"}}}},"shape":{}}}"#,
            shape(size)
        )
    };
    let bump = r#"{"step":"execute_command","command":"t.Bump","actor":"t.Client"}"#;
    let unchanged = r#"{"step":"expect_complete_subject_unchanged","view":"t.Items"}"#;
    expect(
        &format!(
            r#""snapshot/pass":{{"steps":[{ITEMS},{},{ITEMS},{unchanged}]}},
            "snapshot/fail":{{"steps":[{ITEMS},{}]}},
            "unchanged/fail":{{"steps":[{ITEMS},{},{bump},{ITEMS},{unchanged}]}}"#,
            snapshot("Integer"),
            snapshot("String"),
            snapshot("Integer"),
        ),
        &[
            ("snapshot/pass", Status::Passed),
            ("snapshot/fail", Status::Failed),
            ("unchanged/fail", Status::Failed),
        ],
    );
}

#[test]
fn expect_direct_response_holds_the_response_to_its_declared_fields() {
    let response = |fields: &str| {
        format!(
            r#"{{"step":"expect_direct_response","response":{{"command":"t.Create","outcome":{{"command":"t.Create","outcome":"created"}},"fields":{fields},"declarations":{{}},"expected":{{}}}}}}"#
        )
    };
    let declared = r#"[{"name":"id","type":"String"},{"name":"size","type":"Integer"},{"name":"note","type":"Optional<String>","presence":"omitted_when_absent"}]"#;
    let closed = r#"[{"name":"id","type":"String"}]"#;
    let present = r#"[{"name":"id","type":"String"},{"name":"size","type":"Integer"},{"name":"note","type":"Optional<String>","presence":"null_when_absent"}]"#;
    expect(
        &format!(
            r#""pass":{{"steps":[{CREATE},{}]}},
            "fail/undeclared":{{"steps":[{CREATE},{}]}},
            "fail/presence":{{"steps":[{CREATE},{}]}}"#,
            response(declared),
            response(closed),
            response(present),
        ),
        &[
            ("pass", Status::Passed),
            ("fail/undeclared", Status::Failed),
            ("fail/presence", Status::Failed),
        ],
    );
}

#[test]
fn expect_response_payload_holds_the_event_to_the_response_it_maps() {
    let response = |target: &str| {
        format!(
            r#"{{"step":"expect_response_payload","response":{{"command":"t.Create","outcome":{{"command":"t.Create","outcome":"created"}},"event":"t.Created","fields":[{{"name":"id","type":"String"}},{{"name":"size","type":"Integer"}}],"declarations":{{}},"mappings":{{"{target}":"id"}},"targets":[{{"name":"{target}","type":"String"}}]}}}}"#
        )
    };
    expect(
        &format!(
            r#""pass":{{"steps":[{CREATE},{}]}},
            "fail":{{"steps":[{CREATE},{}]}}"#,
            response("task_id"),
            response("note"),
        ),
        &[("pass", Status::Passed), ("fail", Status::Failed)],
    );
}

#[test]
fn configure_external_outcome_forces_the_next_run_or_comes_back_unsupported() {
    let force = |command: &str, outcome: &str| {
        format!(
            r#"{{"step":"configure_external_outcome","force":{{"command":"{command}","outcome":"{outcome}"}}}}"#
        )
    };
    let results = expect(
        &format!(
            r#""pass":{{"steps":[{},{CREATE},{{"step":"expect_outcome","outcome":{{"command":"t.Create","outcome":"boom"}}}}]}},
            "fail":{{"steps":[{}]}},
            "unsupported":{{"steps":[{},{CREATE}]}}"#,
            force("t.Create", "boom"),
            force("t.Refuse", "other"),
            force("t.Create", "elsewhere"),
        ),
        &[
            ("pass", Status::Passed),
            ("fail", Status::Error),
            ("unsupported", Status::Unsupported),
        ],
    );
    assert_eq!(
        results
            .get("unsupported")
            .and_then(|result| result.message.as_deref()),
        Some("the stub cannot force t.Create elsewhere")
    );
}

#[test]
fn await_outcome_holds_what_the_system_issued() {
    let shape = r#"{"id":{"holds":"primitive","kind":"string"},"note":{"holds":"primitive","kind":"string","optional":true}}"#;
    let wrong = r#"{"note":{"holds":"primitive","kind":"string"}}"#;
    let close = |instance: &str, shape: &str| {
        format!(
            r#"{{"step":"await_outcome","command":"t.Close","identity":"id","instance":{instance},"shape":{shape}}}"#
        )
    };
    let item = r#"{"kind":"instance","instance":"item"}"#;
    let relayed = r#"{"step":"expect_event_values","event":"t.Closed","payload":{"task_id":{"kind":"awaited","field":"id"},"note":{"kind":"awaited","field":"note"}}}"#;
    let results = expect(
        &format!(
            r#""pass":{{"steps":[{CREATE},{CAPTURE},{},{{"step":"expect_outcome","outcome":{{"command":"t.Close","outcome":"closed"}}}},{relayed}]}},
            "fail/none":{{"steps":[{}]}},
            "fail/shape":{{"steps":[{CREATE},{CAPTURE},{}]}},
            "unsupported":{{"steps":[{{"step":"await_outcome","command":"t.Elsewhere","identity":"id","instance":{{"kind":"literal","value":"x-1"}}}}]}}"#,
            close(item, shape),
            close(r#"{"kind":"literal","value":"x-2"}"#, shape),
            close(item, wrong),
        ),
        &[
            ("pass", Status::Passed),
            ("fail/none", Status::Failed),
            ("fail/shape", Status::Failed),
            ("unsupported", Status::Unsupported),
        ],
    );
    assert_eq!(
        results
            .get("unsupported")
            .and_then(|result| result.message.as_deref()),
        Some("the stub observes no t.Elsewhere")
    );
}

#[test]
fn expect_not_issued_holds_that_the_system_did_not_issue_it() {
    let not_issued = |instance: &str| {
        format!(
            r#"{{"step":"expect_not_issued","command":"t.Close","instance":{{"kind":"literal","value":"{instance}"}},"unpublished":["t.Closed"]}}"#
        )
    };
    expect(
        &format!(
            r#""pass":{{"steps":[{}]}},"fail":{{"steps":[{}]}}"#,
            not_issued("x-2"),
            not_issued("x-1"),
        ),
        &[("pass", Status::Passed), ("fail", Status::Failed)],
    );
}

#[test]
fn compact_spellings_expand_against_the_event_catalog() {
    let nobody = r#"{"step":"execute_command","command":"t.Create","actor":"t.Nobody"}"#;
    let leak = r#"{"step":"execute_command","command":"t.Leak","actor":"t.Nobody"}"#;
    let refused = r#"{"step":"expect_not_granted","actor":"t.Nobody"}"#;
    let outcome =
        r#"{"step":"expect_outcome","outcome":{"command":"t.Create","outcome":"created"}}"#;
    expect(
        &format!(
            r#""absent/pass":{{"steps":[{CREATE},{outcome}],"absent_events":[{{"before":1,"except":["t.Created"]}}]}},
            "absent/fail":{{"steps":[{CREATE},{outcome}],"absent_events":[{{"before":1,"except":[]}}]}},
            "unpublished/pass":{{"steps":[{nobody},{refused}],"unpublished_events":[{{"step":1,"except":["t.Bumped"]}}]}},
            "unpublished/fail":{{"steps":[{leak},{refused}],"unpublished_events":[{{"step":1,"except":["t.Bumped"]}}]}},
            "unpublished/misplaced":{{"steps":[{nobody},{refused}],"unpublished_events":[{{"step":0,"except":[]}}]}}"#
        ),
        &[
            ("absent/pass", Status::Passed),
            ("absent/fail", Status::Failed),
            ("unpublished/pass", Status::Passed),
            ("unpublished/fail", Status::Failed),
            ("unpublished/misplaced", Status::Error),
        ],
    );
}

/// ESS 0.75.0: an authored scenario can claim a struct-valued payload field only as a whole
/// literal; authoring refuses the dotted leaf key synthesis writes for the same claim, and the
/// runner compares the literal whole, so a leaf it leaves out reads as claimed absent. The leaves
/// it writes are held; one it leaves out that the target carries makes the scenario
/// `unsupported`, naming the leaf, and never `passed`.
#[test]
fn an_authored_struct_literal_holds_the_leaves_it_writes_and_names_those_it_leaves_out() {
    let stamp = r#"{"step":"execute_command","command":"t.Stamp","actor":"t.Client"}"#;
    let claim = |status: &str| {
        format!(r#"{{"step":"expect_event","event":"t.Stamped","payload":{{"status":{status}}}}}"#)
    };
    let (open, closed, whole) = (
        claim(r#"{"state":"Open"}"#),
        claim(r#"{"state":"Closed"}"#),
        claim(r#"{"state":"Open","at":"2026-01-01T00:00:00Z"}"#),
    );
    let results = expect(
        &format!(
            r#""t/authored/left-out":{{"steps":[{stamp},{open}]}},
            "t/authored/written-wrong":{{"steps":[{stamp},{closed}]}},
            "t/authored/whole":{{"steps":[{stamp},{whole}]}},
            "t/authored/left-out-then-wrong":{{"steps":[{stamp},{open},{CREATE},{{"step":"expect_outcome","outcome":{{"command":"t.Create","outcome":"missing"}}}}]}},
            "t.Stamp/outcome/stamped":{{"steps":[{stamp},{open}]}}"#
        ),
        &[
            ("t/authored/left-out", Status::Unsupported),
            ("t/authored/written-wrong", Status::Failed),
            ("t/authored/whole", Status::Passed),
            ("t/authored/left-out-then-wrong", Status::Failed),
            ("t.Stamp/outcome/stamped", Status::Failed),
        ],
    );
    let reason = results
        .get("t/authored/left-out")
        .and_then(|result| result.message.clone())
        .unwrap_or_default();
    assert!(
        reason.starts_with("ESS 0.75.0") && reason.contains("`status.at`"),
        "{reason}"
    );
    assert!(!reason.contains("2026-01-01"), "{reason}");
}

#[test]
fn a_step_or_field_the_interpreter_does_not_know_is_an_error_not_a_skip() {
    let results = expect(
        &format!(
            r#""kind":{{"steps":[{CREATE},{{"step":"teleport"}}]}},
            "field":{{"steps":[{CREATE},{{"step":"expect_no_error","severity":"high"}}]}},
            "within":{{"steps":[{{"step":"await_outcome","command":"t.Close","identity":"id","instance":{{"kind":"literal","value":"x-1"}},"within":"PT1S"}}]}}"#
        ),
        &[
            ("kind", Status::Error),
            ("field", Status::Error),
            ("within", Status::Error),
        ],
    );
    let message = results
        .get("kind")
        .and_then(|result| result.message.clone());
    assert!(
        message
            .as_deref()
            .is_some_and(|message| message.contains("teleport")),
        "{message:?}"
    );
}

#[test]
fn the_results_document_holds_one_status_per_scenario() {
    let results = expect(
        &format!(r#""a":{{"steps":[{CREATE}]}},"b":{{"steps":[{{"step":"teleport"}}]}}"#),
        &[("a", Status::Passed), ("b", Status::Error)],
    );
    let document = results.to_json();
    let read = Results::parse(&document).expect("the document reads back");
    assert_eq!(read, results);
    let value = json(&document);
    assert_eq!(
        value.member("format"),
        Some(&Value::Text("ess-conformance-results/1".to_owned()))
    );
    assert!(value.member("results").is_some());
}

#[test]
fn a_suite_of_another_version_is_refused() {
    let refused =
        Suite::parse(r#"{"provenance":{"suite_version":"ess-conformance/51"},"scenarios":{}}"#);
    assert!(refused.is_err());
}
