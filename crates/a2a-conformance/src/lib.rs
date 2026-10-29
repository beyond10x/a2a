//! Runs the conformance suite ESS synthesizes from `ess/` against a [`Target`], and writes an
//! `ess-conformance-results/1` document with one status per scenario.
//!
//! The interpreter carries out every step kind an `ess-conformance/52` suite of this
//! specification holds, as the pinned `ess` runner carries them out. A step kind or field it does
//! not know makes that scenario `error`; nothing is skipped. A target answers
//! [`TargetError::Unsupported`] for what it cannot carry out, and the scenario is `unsupported`
//! with that reason.

pub mod gate;
mod interpret;
pub mod naming;
mod results;
mod server;
mod shape;
mod suite;
mod target;
mod value;

pub use a2a::json::Value;
pub use results::{RESULTS_FORMAT, Results, ScenarioResult, Status};
pub use server::ServerTarget;
pub use suite::{SUITE_VERSION, Suite};
pub use target::{Answer, Awaited, Event, Reader, Target, TargetError};

/// Runs every scenario of `suite` against `target`, each in its own context, in suite order.
pub fn run<T: Target + ?Sized>(suite: &Suite, target: &mut T) -> Results {
    let results = suite
        .scenarios
        .iter()
        .map(|(id, scenario)| interpret::scenario(id, scenario, target))
        .collect();
    Results {
        completed_at: now_ms(),
        results,
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}
