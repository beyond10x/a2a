//! Adversary: the `a2a-testkit run` command line against a correct endpoint in a documented
//! configuration, and with a `--results` path it cannot write.

mod adv_support;
mod common;

use std::net::TcpListener;

use a2a_conformance::Status;
use a2a_server::{Access, BearerToken, EchoAgent, Limits, Settings};
use adv_support::{counts, line, run_against, suite};
use common::{Endpoint, echo, printed, run, scratch};

const EXTENDED: &str = "a2a.discovery.GetExtendedAgentCard/outcome/returned";

/// `a2a-server --extended-card <json> --extended-card-token-file <path>` (AGENTS.md, Commands):
/// the card declares the extended card and the server serves it for a bearer token, as A2A v1.0.1
/// section 3.1.11 has it.
fn with_extended_card() -> Endpoint {
    let mut card = a2a_server::default_card();
    card.capabilities.extended_agent_card = Some(true);
    let mut extended = card.clone();
    extended.name = "the extended card".to_owned();
    let settings = Settings {
        card,
        extended_card: Some(extended),
    };
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = listener.local_addr().expect("bound address");
    let accepted = settings.card.default_input_modes.clone();
    std::thread::spawn(move || {
        let _ = a2a_server::serve_with(
            listener,
            settings,
            Box::new(EchoAgent::new(accepted)),
            Limits::DEFAULT,
            Access {
                extended_card_token: BearerToken::new("t0ken"),
            },
        );
    });
    Endpoint { address }
}

/// The repository's own server, configured as AGENTS.md documents, is a correct endpoint; the run
/// sends no credential and files its `401 UNAUTHENTICATED` as a failure of the endpoint.
#[test]
fn a_server_serving_its_extended_card_for_a_token_is_not_reported_failed() {
    let server = with_extended_card();
    let (output, results) = run_against(&server.url(), &suite("adv_cli"), "adv-cli-extended");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !matches!(
            results.status(EXTENDED),
            Some(Status::Failed | Status::Error)
        ),
        "a correct endpoint is reported wrong: {:?}",
        line(&stdout, EXTENDED)
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}\n{}",
        counts(&output),
        line(&stdout, EXTENDED).unwrap_or_default()
    );
}

/// Acceptance 2: the run prints one line per scenario and the counts. With a `--results` path it
/// cannot write, the suite has run against the endpoint and nothing of it is printed.
#[test]
fn an_unwritable_results_path_still_prints_the_run() {
    let server = echo();
    let directory = scratch("adv-cli-unwritable");
    let unwritable = directory.join("no-such-directory").join("results.json");
    let output = run(&server.url(), &suite("adv_cli"), &unwritable);
    assert_ne!(output.status.code(), Some(0), "{}", printed(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout
            .lines()
            .any(|line| line == "passed a2a.tasks.GetTask/outcome/not-found"),
        "the run's lines are not printed when --results cannot be written\n{}",
        printed(&output)
    );
}
