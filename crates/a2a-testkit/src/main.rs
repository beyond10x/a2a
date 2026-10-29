//! `a2a-testkit`: runs the synthesized conformance suite against an A2A endpoint at a base URL,
//! over its HTTP+JSON binding.
//!
//! Prints one line per scenario, its status, its id and the reason where it did not pass, then
//! the counts. Exits 0 when at least one scenario passed and none is `failed` or `error`; 1 when
//! one is, or when none passed, since such a run checked nothing; 2 when the run cannot start (a
//! suite that cannot be read, a base URL `a2a-client` refuses, a bearer file that cannot be read)
//! or its results cannot be written, which is reported after the lines are printed.

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
    ///
    /// Exits 0 when at least one scenario passed and none is failed or error; 1 when one is
    /// failed or error, or when no scenario passed, since that run checked nothing; 2 when the
    /// run cannot start or its results cannot be written.
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
        /// A file holding the bearer token to send with every operation, as the extended agent
        /// card requires (A2A v1.0.1 section 3.1.11). The token is never printed. Without it, an
        /// extended card served only to authenticated clients is `unsupported`.
        #[arg(long, value_name = "PATH")]
        bearer_file: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let Command::Run {
        url,
        suite,
        results,
        bearer_file,
    } = Cli::parse().command;
    let ran = match run(&url, &suite, bearer_file.as_deref()) {
        Ok(ran) => ran,
        Err(refusal) => {
            eprintln!("a2a-testkit: {refusal}");
            return ExitCode::from(2);
        }
    };
    for result in &ran.results {
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
    let passed = ran.count(Status::Passed);
    let wrong = ran.count(Status::Failed) + ran.count(Status::Error);
    println!(
        "{} scenario(s): {passed} passed, {} failed, {} error, {} unsupported{}",
        ran.results.len(),
        ran.count(Status::Failed),
        ran.count(Status::Error),
        ran.count(Status::Unsupported),
        if passed == 0 {
            "; no scenario passed, so the run checked nothing"
        } else {
            ""
        }
    );
    if let Some(out) = results
        && let Err(error) = std::fs::write(&out, ran.to_json())
    {
        eprintln!("a2a-testkit: {}: {error}", out.display());
        return ExitCode::from(2);
    }
    if wrong == 0 && passed > 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run(url: &str, suite: &Path, bearer_file: Option<&Path>) -> Result<Results, String> {
    let client = Client::new(url).map_err(|error| format!("--url: {error}"))?;
    let text =
        std::fs::read_to_string(suite).map_err(|error| format!("{}: {error}", suite.display()))?;
    let parsed = Suite::parse(&text).map_err(|error| format!("{}: {error}", suite.display()))?;
    let mut target = WireTarget::new(client)
        .with_suite(&text)
        .map_err(|error| format!("{}: {error}", suite.display()))?;
    if let Some(path) = bearer_file {
        let token = std::fs::read_to_string(path)
            .map_err(|error| format!("--bearer-file {}: {error}", path.display()))?;
        let token = token.trim();
        if token.is_empty() {
            return Err(format!("--bearer-file {}: holds no token", path.display()));
        }
        target = target.with_bearer(token);
    }
    Ok(a2a_conformance::run(&parsed, &mut target))
}
