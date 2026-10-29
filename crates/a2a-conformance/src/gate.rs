//! The gate `task conform` holds a run to.
//!
//! The run passes where the `ess-conformance-report/2` `ess verify conform report` made from its
//! results counts 0 failed and 0 error, and the scenarios that came out `unsupported` are exactly
//! those the allowance file names, each with the reason the run gave. A scenario that becomes
//! unsupported, changes its reason, or stops being unsupported fails the gate until the file says
//! so.

use std::collections::BTreeMap;

use a2a::json::{self, Value};

use crate::{Results, Status};

/// The report format the gate reads.
pub const REPORT_FORMAT: &str = "ess-conformance-report/2";

/// Holds a run to the gate: `report` is the report made from `results`, `unsupported` the
/// allowance file, one scenario per line as its id, a tab, and its reason.
///
/// # Errors
///
/// The run does not pass the gate, or an input cannot be read; the refusal names every scenario
/// that keeps it from passing.
pub fn hold(report: &str, results: &Results, unsupported: &str) -> Result<String, String> {
    let report = Report::read(report)?;
    let allowed = allowances(unsupported)?;
    report.is_of(results)?;
    let mut refusals = Vec::new();
    for status in [Status::Failed, Status::Error] {
        let held: Vec<String> = results
            .results
            .iter()
            .filter(|result| result.status == status)
            .map(|result| {
                format!(
                    "{} ({})",
                    result.scenario_id,
                    result.message.as_deref().unwrap_or("no message")
                )
            })
            .collect();
        if report.count(status) != 0 {
            refusals.push(format!(
                "{} {status}: {}",
                report.count(status),
                held.join("; ")
            ));
        }
    }
    for result in &results.results {
        if result.status != Status::Unsupported {
            continue;
        }
        let reason = result.message.as_deref().unwrap_or_default();
        match allowed.get(result.scenario_id.as_str()) {
            None => refusals.push(format!(
                "{} is unsupported and the allowance file does not name it; its reason: {reason}",
                result.scenario_id
            )),
            Some((_, written)) if *written != reason => refusals.push(format!(
                "{} is unsupported for another reason than the allowance file gives: the run \
                 gave `{reason}`, the file `{written}`",
                result.scenario_id
            )),
            Some(_) => {}
        }
    }
    for (id, (line, _)) in &allowed {
        match results.get(id) {
            None => refusals.push(format!(
                "the allowance file names {id} (line {line}), which the run does not hold"
            )),
            Some(result) if result.status != Status::Unsupported => refusals.push(format!(
                "the allowance file names {id} (line {line}), which came out {}",
                result.status
            )),
            Some(_) => {}
        }
    }
    if refusals.is_empty() {
        Ok(format!(
            "conformance gate passed: {} scenario(s), {} passed, 0 failed, 0 error, {} unsupported, \
             each named in the allowance file",
            results.results.len(),
            report.count(Status::Passed),
            report.count(Status::Unsupported)
        ))
    } else {
        Err(format!(
            "conformance gate failed:\n  {}",
            refusals.join("\n  ")
        ))
    }
}

/// What the gate reads of a report.
struct Report {
    counts: BTreeMap<String, u64>,
    unsupported: Vec<String>,
}

impl Report {
    fn read(text: &str) -> Result<Self, String> {
        let document = json::parse_written(text)
            .map_err(|error| format!("the report is not JSON: {error}"))?;
        match document.member("format") {
            Some(Value::Text(format)) if format == REPORT_FORMAT => {}
            other => {
                return Err(format!(
                    "the report is not an {REPORT_FORMAT}: its format is {}",
                    other.map_or_else(|| "absent".to_owned(), crate::value::quote)
                ));
            }
        }
        let Some(Value::Object(counts)) = document.member("counts") else {
            return Err("the report holds no counts".to_owned());
        };
        let counts = counts
            .iter()
            .filter_map(|(name, count)| match count {
                Value::Number(number) => number.parse().ok().map(|count| (name.clone(), count)),
                _ => None,
            })
            .collect();
        let unsupported = match document
            .member("outcomes")
            .and_then(|outcomes| outcomes.member("unsupported"))
        {
            Some(Value::Array(ids)) => ids
                .iter()
                .map(|id| match id {
                    Value::Text(id) => Ok(id.clone()),
                    _ => Err("the report names an unsupported scenario by no id".to_owned()),
                })
                .collect::<Result<_, _>>()?,
            _ => return Err("the report lists no unsupported outcomes".to_owned()),
        };
        Ok(Self {
            counts,
            unsupported,
        })
    }

    fn count(&self, status: Status) -> u64 {
        self.counts
            .get(status.as_str())
            .copied()
            .unwrap_or_default()
    }

    /// Whether the report was made from `results`: the same number of scenarios, the same count of
    /// each status, and the same scenarios unsupported.
    fn is_of(&self, results: &Results) -> Result<(), String> {
        let mut differences = Vec::new();
        let total = self.counts.get("total").copied().unwrap_or_default();
        if usize::try_from(total).ok() != Some(results.results.len()) {
            differences.push(format!(
                "the report counts {total} scenario(s), the results hold {}",
                results.results.len()
            ));
        }
        for status in [
            Status::Passed,
            Status::Failed,
            Status::Error,
            Status::Unsupported,
        ] {
            let held = results.count(status);
            if usize::try_from(self.count(status)).ok() != Some(held) {
                differences.push(format!(
                    "the report counts {} {status}, the results hold {held}",
                    self.count(status)
                ));
            }
        }
        let mut reported = self.unsupported.clone();
        reported.sort();
        let mut held: Vec<String> = results
            .results
            .iter()
            .filter(|result| result.status == Status::Unsupported)
            .map(|result| result.scenario_id.clone())
            .collect();
        held.sort();
        for id in held.iter().filter(|id| !reported.contains(id)) {
            differences.push(format!(
                "{id} is unsupported in the results, not in the report"
            ));
        }
        for id in reported.iter().filter(|id| !held.contains(id)) {
            differences.push(format!(
                "{id} is unsupported in the report, not in the results"
            ));
        }
        if differences.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "the report was not made from these results: {}",
                differences.join("; ")
            ))
        }
    }
}

/// The allowance file: each scenario it names, with the line that names it and the reason.
fn allowances(text: &str) -> Result<BTreeMap<&str, (usize, &str)>, String> {
    let mut allowed = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        let Some((id, reason)) = line.split_once('\t') else {
            return Err(format!(
                "allowance file line {number}: expected a scenario id, a tab and a reason"
            ));
        };
        if id.is_empty() || reason.is_empty() {
            return Err(format!(
                "allowance file line {number}: expected a scenario id, a tab and a reason"
            ));
        }
        if let Some((first, _)) = allowed.insert(id, (number, reason)) {
            return Err(format!(
                "allowance file line {number}: {id} is named on line {first} already"
            ));
        }
    }
    Ok(allowed)
}
