//! `a2a-testkit`: runs the synthesized conformance suite against an A2A endpoint at a base URL,
//! over its HTTP+JSON binding.
//!
//! Prints one line per scenario, its status, its id and the reason where it did not pass, then
//! the counts. Exits 0 when no scenario is `failed` or `error`, 1 when one is, and 2 when the run
//! cannot start: a suite that cannot be read, a base URL `a2a-client` refuses, or results that
//! cannot be written.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use a2a_client::Client;
use a2a_conformance::{Results, Status, Suite};
use a2a_testkit::WireTarget;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "a2a-testkit", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run every scenario of an `ess-conformance/52` suite against the endpoint, each step the
    /// wire can observe through a2a-client and every other one `unsupported` with its reason.
    Run {
        /// The endpoint's base URL, `http://host[:port][/path]`.
        #[arg(long)]
        url: String,
        /// The suite `ess verify conform synthesize` wrote.
        #[arg(long)]
        suite: PathBuf,
        /// Where to write the `ess-conformance-results/1` document of the run.
        #[arg(long)]
        results: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let Command::Run {
        url,
        suite,
        results,
    } = Cli::parse().command;
    match run(&url, &suite, results.as_deref()) {
        Ok(results) => {
            for result in &results.results {
                match &result.message {
                    Some(message) => println!(
                        "{} {}: {}",
                        result.status,
                        result.scenario_id,
                        message.replace(['\n', '\r'], " ")
                    ),
                    None => println!("{} {}", result.status, result.scenario_id),
                }
            }
            println!(
                "{} scenario(s): {} passed, {} failed, {} error, {} unsupported",
                results.results.len(),
                results.count(Status::Passed),
                results.count(Status::Failed),
                results.count(Status::Error),
                results.count(Status::Unsupported)
            );
            if results.count(Status::Failed) + results.count(Status::Error) == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(refusal) => {
            eprintln!("a2a-testkit: {refusal}");
            ExitCode::from(2)
        }
    }
}

fn run(url: &str, suite: &Path, out: Option<&Path>) -> Result<Results, String> {
    let client = Client::new(url).map_err(|error| format!("--url: {error}"))?;
    let text =
        std::fs::read_to_string(suite).map_err(|error| format!("{}: {error}", suite.display()))?;
    let suite = Suite::parse(&text).map_err(|error| format!("{}: {error}", suite.display()))?;
    let results = a2a_conformance::run(&suite, &mut WireTarget::new(client));
    if let Some(out) = out {
        std::fs::write(out, results.to_json())
            .map_err(|error| format!("{}: {error}", out.display()))?;
    }
    Ok(results)
}
