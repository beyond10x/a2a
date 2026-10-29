//! The outcome each scenario of a suite expects of each command it runs.
//!
//! The interpreter of `a2a-conformance` asks the target to run a command and to hand over what it
//! published before it checks the command's outcome. Over the wire the events are not observable,
//! so a target that took an outcome which publishes would answer the scenario `unsupported` before
//! the interpreter sees that the outcome is not the one expected. Knowing the expectation, the
//! target reports such an outcome as taken, and the scenario fails on it.

use std::collections::BTreeMap;

use a2a::json::{self, Value};

/// Per scenario, for each `execute_command` step in order, the outcome the first
/// `expect_outcome` of that command before the next `execute_command` names.
#[derive(Debug, Default)]
pub(crate) struct Expectations(BTreeMap<String, Vec<Option<String>>>);

fn text(value: Option<&Value>) -> Option<&str> {
    match value {
        Some(Value::Text(text)) => Some(text),
        _ => None,
    }
}

impl Expectations {
    /// The expectations of the `ess-conformance/52` suite `suite`.
    ///
    /// # Errors
    ///
    /// A message where `suite` is not JSON or holds no `scenarios` object.
    pub(crate) fn read(suite: &str) -> Result<Self, String> {
        let document = json::parse_written(suite)
            .map_err(|error| format!("the suite is not JSON: {error}"))?;
        let Some(Value::Object(scenarios)) = document.member("scenarios") else {
            return Err("the suite holds no `scenarios` object".to_owned());
        };
        let mut read = BTreeMap::new();
        for (id, scenario) in scenarios {
            let Some(Value::Array(steps)) = scenario.member("steps") else {
                continue;
            };
            let mut expected: Vec<Option<String>> = Vec::new();
            let mut last: Option<&str> = None;
            for step in steps {
                match text(step.member("step")) {
                    Some("execute_command") => {
                        expected.push(None);
                        last = text(step.member("command"));
                    }
                    Some("expect_outcome") => {
                        let outcome = step.member("outcome");
                        let named = text(outcome.and_then(|outcome| outcome.member("command")));
                        if let (Some(slot), Some(command), Some(name)) = (
                            expected.last_mut(),
                            last,
                            text(outcome.and_then(|outcome| outcome.member("outcome"))),
                        ) && named == Some(command)
                            && slot.is_none()
                        {
                            *slot = Some(name.to_owned());
                        }
                    }
                    _ => {}
                }
            }
            read.insert(id.clone(), expected);
        }
        Ok(Self(read))
    }

    /// The expectations of the scenario `id`, none where the suite does not hold it.
    pub(crate) fn of(&self, id: &str) -> Vec<Option<String>> {
        self.0.get(id).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::Expectations;

    #[test]
    fn each_command_run_expects_the_outcome_checked_before_the_next_one() {
        let read = Expectations::read(
            r#"{"scenarios":{"s":{"steps":[
                {"step":"execute_command","command":"a.B"},
                {"step":"expect_outcome","outcome":{"command":"a.B","outcome":"made"}},
                {"step":"capture_instance"},
                {"step":"execute_command","command":"a.C"},
                {"step":"expect_error","error":"a.E"},
                {"step":"execute_command","command":"a.D"},
                {"step":"expect_outcome","outcome":{"command":"a.C","outcome":"late"}},
                {"step":"expect_outcome","outcome":{"command":"a.D","outcome":"first"}},
                {"step":"expect_outcome","outcome":{"command":"a.D","outcome":"second"}}
            ]}}}"#,
        )
        .expect("a suite");
        assert_eq!(
            read.of("s"),
            [Some("made".to_owned()), None, Some("first".to_owned())]
        );
        assert!(read.of("another").is_empty());
    }
}
