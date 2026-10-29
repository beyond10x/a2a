//! `a2a-conformance`: runs the synthesized suite against `a2a-server` in process, and holds a
//! run's report to the scenarios `unsupported.tsv` names.

use std::path::PathBuf;
use std::process::ExitCode;

use a2a_conformance::{Results, ServerTarget, Status, Suite, gate};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "a2a-conformance", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run every scenario of an `ess-conformance/52` suite against a2a-server, each on a fresh
    /// in-process server, and write an `ess-conformance-results/1` document.
    Run {
        /// The suite `ess verify conform synthesize` wrote.
        #[arg(long)]
        suite: PathBuf,
        /// Where to write the results.
        #[arg(long)]
        results: PathBuf,
    },
    /// Hold a run to the gate: the report `ess verify conform report` wrote has 0 failed and 0
    /// error, and the scenarios that came out `unsupported` are exactly those the file names,
    /// each with the reason the run gave.
    Gate {
        /// The `ess-conformance-report/2` of the run.
        #[arg(long)]
        report: PathBuf,
        /// The `ess-conformance-results/1` the report was made from.
        #[arg(long)]
        results: PathBuf,
        /// The scenarios allowed to be `unsupported`: one per line, its id, a tab, its reason.
        #[arg(long)]
        unsupported: PathBuf,
    },
}

fn read(path: &PathBuf) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn main() -> ExitCode {
    let outcome = match Cli::parse().command {
        Command::Run { suite, results } => run(&suite, &results),
        Command::Gate {
            report,
            results,
            unsupported,
        } => (|| {
            let results = Results::parse(&read(&results)?)?;
            gate::hold(&read(&report)?, &results, &read(&unsupported)?)
        })(),
    };
    match outcome {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!("a2a-conformance: {refusal}");
            ExitCode::FAILURE
        }
    }
}

fn run(suite: &PathBuf, out: &PathBuf) -> Result<String, String> {
    let suite = Suite::parse(&read(suite)?)?;
    let results = a2a_conformance::run(&suite, &mut ServerTarget::new());
    std::fs::write(out, results.to_json())
        .map_err(|error| format!("{}: {error}", out.display()))?;
    let counts: Vec<String> = [
        Status::Passed,
        Status::Failed,
        Status::Error,
        Status::Unsupported,
    ]
    .iter()
    .map(|status| format!("{} {status}", results.count(*status)))
    .collect();
    Ok(format!(
        "{} scenario(s): {}; written to {}",
        results.results.len(),
        counts.join(", "),
        out.display()
    ))
}
