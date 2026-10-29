//! The `ess-conformance-results/1` document a run writes: one status per scenario, and for every
//! scenario that did not pass, why.

use std::fmt;

use a2a::json::{self, Value};

/// The results format `ess verify conform report` reads.
pub const RESULTS_FORMAT: &str = "ess-conformance-results/1";

/// How one scenario came out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Status {
    /// Every check held.
    Passed,
    /// The target could not carry a step out.
    Unsupported,
    /// The scenario could not be run: the suite, the interpreter or the target failed.
    Error,
    /// A check did not hold.
    Failed,
}

impl Status {
    /// The status as the results document spells it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Unsupported => "unsupported",
            Self::Error => "error",
            Self::Failed => "failed",
        }
    }

    /// The status a spelling names.
    #[must_use]
    pub fn named(text: &str) -> Option<Self> {
        Some(match text {
            "passed" => Self::Passed,
            "unsupported" => Self::Unsupported,
            "error" => Self::Error,
            "failed" => Self::Failed,
            _ => return None,
        })
    }
}

impl fmt::Display for Status {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// How one scenario came out, and why where it did not pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioResult {
    /// The scenario's id in the suite.
    pub scenario_id: String,
    /// Its status.
    pub status: Status,
    /// Why it did not pass: the first check that decided its status. For `unsupported`, the
    /// reason the target gave.
    pub message: Option<String>,
}

/// Every scenario's result, in suite order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Results {
    /// When the run finished, in milliseconds since the Unix epoch.
    pub completed_at: u64,
    /// One result per scenario.
    pub results: Vec<ScenarioResult>,
}

impl Results {
    /// The result of the scenario `id`.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ScenarioResult> {
        self.results.iter().find(|result| result.scenario_id == id)
    }

    /// The status of the scenario `id`.
    #[must_use]
    pub fn status(&self, id: &str) -> Option<Status> {
        self.get(id).map(|result| result.status)
    }

    /// How many scenarios came out `status`.
    #[must_use]
    pub fn count(&self, status: Status) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == status)
            .count()
    }

    /// The `ess-conformance-results/1` document.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut out = String::from("{");
        json::member(&mut out, "format");
        json::push_text(&mut out, RESULTS_FORMAT);
        json::member(&mut out, "completed_at");
        out.push_str(&self.completed_at.to_string());
        json::member(&mut out, "results");
        out.push('[');
        for (position, result) in self.results.iter().enumerate() {
            if position > 0 {
                out.push(',');
            }
            out.push('{');
            json::member(&mut out, "scenario_id");
            json::push_text(&mut out, &result.scenario_id);
            json::member(&mut out, "status");
            json::push_text(&mut out, result.status.as_str());
            if let Some(message) = &result.message {
                json::member(&mut out, "message");
                json::push_text(&mut out, message);
            }
            out.push('}');
        }
        out.push_str("]}\n");
        out
    }

    /// Reads an `ess-conformance-results/1` document.
    ///
    /// # Errors
    ///
    /// The document is not JSON or not of that format.
    pub fn parse(text: &str) -> Result<Self, String> {
        let document = json::parse_written(text)
            .map_err(|error| format!("the results are not JSON: {error}"))?;
        match document.member("format") {
            Some(Value::Text(format)) if format == RESULTS_FORMAT => {}
            _ => return Err(format!("the results are not {RESULTS_FORMAT}")),
        }
        let completed_at = match document.member("completed_at") {
            Some(Value::Number(spelling)) => spelling
                .parse()
                .map_err(|_| format!("completed_at is not a count: {spelling}"))?,
            _ => return Err("the results carry no completed_at".to_owned()),
        };
        let Some(Value::Array(items)) = document.member("results") else {
            return Err("the results carry no results list".to_owned());
        };
        let results = items
            .iter()
            .map(|item| {
                let text = |name: &str| match item.member(name) {
                    Some(Value::Text(text)) => Some(text.clone()),
                    _ => None,
                };
                let scenario_id = text("scenario_id").ok_or("a result names no scenario_id")?;
                let status = text("status")
                    .as_deref()
                    .and_then(Status::named)
                    .ok_or_else(|| format!("{scenario_id}: no status"))?;
                Ok(ScenarioResult {
                    scenario_id,
                    status,
                    message: text("message"),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self {
            completed_at,
            results,
        })
    }
}
