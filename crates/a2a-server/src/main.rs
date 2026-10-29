//! `a2a-server`: the echo reference agent behind A2A's HTTP+JSON binding.

use std::io::Write as _;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use a2a_server::{Access, BearerToken, EchoAgent, Limits, Settings};
use clap::Parser;

/// Serves the echo reference agent over A2A's HTTP+JSON binding.
#[derive(Debug, Parser)]
#[command(name = "a2a-server", version)]
struct Cli {
    /// The address to listen on; port 0 takes a free port.
    #[arg(long, default_value = "127.0.0.1:8080")]
    listen: String,
    /// A JSON agent card to serve. Without it, the built-in card, its interface URL set to the
    /// bound address.
    #[arg(long)]
    card: Option<PathBuf>,
    /// A JSON extended agent card, served by `GET /extendedAgentCard` when the card declares
    /// `capabilities.extendedAgentCard`, to a request with the token of
    /// `--extended-card-token-file`.
    #[arg(long, requires = "extended_card_token_file")]
    extended_card: Option<PathBuf>,
    /// A file holding the bearer token `GET /extendedAgentCard` asks for, as `Authorization:
    /// Bearer <token>`; a request without it is answered 401. The token is never printed.
    #[arg(long, value_name = "PATH")]
    extended_card_token_file: Option<PathBuf>,
    /// The most connections held at once, fewer where the file descriptor limit would run out
    /// first. A connection over the cap is closed: the one waiting longest for a request, or the
    /// new one where every connection held is being answered.
    #[arg(long, default_value_t = Limits::DEFAULT.max_connections, value_parser = positive::<usize>)]
    max_connections: usize,
    /// Seconds a client may take to send its whole request, from when it connects; a request
    /// not in by then is answered 408 and its connection closed. After its answer, a connection
    /// is closed once as long again has passed.
    #[arg(long, value_name = "SECONDS", default_value_t = Limits::DEFAULT.request_timeout.as_secs(), value_parser = positive::<u64>)]
    request_timeout: u64,
    /// The most streams following one task at once; one more is answered 429.
    #[arg(long, default_value_t = Limits::DEFAULT.max_subscribers_per_task, value_parser = positive::<usize>)]
    max_subscribers_per_task: usize,
    /// The most streams following tasks at once, across all tasks; one more is answered 429.
    /// [default: half of --max-connections]
    #[arg(long, value_parser = positive::<usize>)]
    max_subscribers: Option<usize>,
    /// The most tasks one tenant holds, requests without a tenant counting as one tenant; a
    /// message that would create one more is answered 429.
    #[arg(long, default_value_t = Limits::DEFAULT.max_tasks, value_parser = positive::<usize>)]
    max_tasks: usize,
    /// The most messages one task's history holds; a message to a task at it is answered 429.
    #[arg(long, default_value_t = Limits::DEFAULT.max_history, value_parser = positive::<usize>)]
    max_history: usize,
    /// The most push notification configurations one task holds; one more is answered 429.
    #[arg(long, default_value_t = Limits::DEFAULT.max_push_configs, value_parser = positive::<usize>)]
    max_push_configs: usize,
}

/// A whole number of 1 or more.
fn positive<T: std::str::FromStr + PartialOrd + From<u8>>(text: &str) -> Result<T, String> {
    match text.parse::<T>() {
        Ok(value) if value >= T::from(1) => Ok(value),
        _ => Err(format!(
            "expected a whole number of 1 or more, not `{text}`"
        )),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("a2a-server: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(), String> {
    let access = Access {
        extended_card_token: cli
            .extended_card_token_file
            .as_deref()
            .map(token_at)
            .transpose()?,
    };
    let limits = Limits {
        max_connections: cli.max_connections,
        request_timeout: Duration::from_secs(cli.request_timeout),
        max_subscribers_per_task: cli.max_subscribers_per_task,
        max_subscribers: cli
            .max_subscribers
            .unwrap_or_else(|| (cli.max_connections / 2).max(1)),
        max_tasks: cli.max_tasks,
        max_history: cli.max_history,
        max_push_configs: cli.max_push_configs,
    };
    let listener =
        TcpListener::bind(&cli.listen).map_err(|error| format!("{}: {error}", cli.listen))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("{}: {error}", cli.listen))?;
    let card = match &cli.card {
        Some(path) => card_at(path)?,
        None => {
            let mut card = a2a_server::default_card();
            for interface in &mut card.supported_interfaces {
                interface.url = format!("http://{address}");
            }
            card
        }
    };
    let extended_card = cli.extended_card.as_deref().map(card_at).transpose()?;
    println!("a2a-server listening on http://{address}");
    let _ = std::io::stdout().flush();
    let agent = EchoAgent::new(card.default_input_modes.clone());
    a2a_server::serve_with(
        listener,
        Settings {
            card,
            extended_card,
        },
        Box::new(agent),
        limits,
        access,
    )
    .map_err(|error| format!("{address}: {error}"))
}

fn card_at(path: &Path) -> Result<a2a::discovery::AgentCard, String> {
    let text =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    a2a_server::read_card(&text).map_err(|error| format!("{}: {error}", path.display()))
}

/// The bearer token the file at `path` holds. An error names the path and never the file's
/// content.
fn token_at(path: &Path) -> Result<BearerToken, String> {
    let text =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    BearerToken::new(&text).ok_or_else(|| {
        format!(
            "{}: expected one bearer token (RFC 6750 `b64token`)",
            path.display()
        )
    })
}
