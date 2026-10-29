//! A2A's HTTP+JSON binding (A2A v1.0.1 section 11) over the generated `a2a` crate.
//!
//! Every model type comes from `generated/rust/a2a`; this crate defines none. It implements the
//! twelve obligations the generated crate leaves open (`generated/rust/a2a/PLAN.md`) over stores
//! of its own that keep [`Limits`], serves the upstream routes, and streams with server-sent
//! events.

pub mod agent;
mod connections;
mod paging;
mod ports;
mod reread;
mod router;
mod steps;

pub use agent::{Agent, EchoAgent, Refusal, Step, Turn};

use std::net::TcpListener;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use a2a::discovery::AgentCard;

/// How long the server waits for a request before it looks for streams whose client hung up.
const TICK: Duration = Duration::from_millis(250);

/// What the server is started with.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// The public agent card, served at `/.well-known/agent-card.json`; its capabilities decide
    /// streaming, push notifications and the extended card, and its interfaces the versions.
    pub card: AgentCard,
    /// The extended agent card, served by `GetExtendedAgentCard` when the card declares
    /// `capabilities.extendedAgentCard`.
    pub extended_card: Option<AgentCard>,
}

/// How much of the machine clients may hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// The most connections held at once. Fewer where the process's file descriptor limit would
    /// run out first. A connection over the cap is closed: the one waiting longest for a request
    /// head, before or after its answer, or the new one where every connection held is being
    /// answered.
    pub max_connections: usize,
    /// How long a client may take to send its whole request, head and body, counted from when
    /// the connection opens; bytes arriving do not extend it. A request not in by then is answered
    /// 408 and its connection closed. Once the answer is written, the client has as long again
    /// before its connection is closed.
    pub request_timeout: Duration,
    /// The most streams following one task at once. A `SubscribeToTask` or
    /// `SendStreamingMessage` over it is answered 429 before its stream opens.
    pub max_subscribers_per_task: usize,
    /// The most streams following tasks at once, across all tasks; over it, as for
    /// [`Limits::max_subscribers_per_task`].
    pub max_subscribers: usize,
    /// The most tasks one tenant holds, every request without a tenant counting as one tenant. A
    /// message that would create one more is answered 429 and stores nothing.
    pub max_tasks: usize,
    /// The most messages one task's history holds. A message to a task at it is answered 429 and
    /// stores nothing; a status message an agent's step moves the task with is not kept.
    pub max_history: usize,
    /// The most push notification configurations one task holds. Creating one more is answered
    /// 429 and stores nothing; replacing one of the same id is not refused.
    pub max_push_configs: usize,
}

impl Limits {
    /// 256 connections, 10 s per request, 16 streams per task and 128 in all, 10 000 tasks per
    /// tenant, 1000 history messages and 16 push notification configurations per task.
    pub const DEFAULT: Self = Self {
        max_connections: 256,
        request_timeout: Duration::from_secs(10),
        max_subscribers_per_task: 16,
        max_subscribers: 128,
        max_tasks: 10_000,
        max_history: 1000,
        max_push_configs: 16,
    };
}

impl Default for Limits {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A bearer token (RFC 6750 section 2.1, `b64token`). It is compared in constant time and never
/// printed: its `Debug` shows `BearerToken(..)`.
#[derive(Clone)]
pub struct BearerToken(String);

impl BearerToken {
    /// The token `text` holds, whitespace around it dropped; `None` where it is empty or not one
    /// `b64token`: `[A-Za-z0-9-._~+/]+=*`.
    pub fn new(text: &str) -> Option<Self> {
        let token = text.trim();
        let body = token.trim_end_matches('=');
        let valid = !body.is_empty()
            && body
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-._~+/".contains(&byte));
        valid.then(|| Self(token.to_owned()))
    }

    /// Whether `presented` is this token, in a time that depends on this token's length alone.
    pub fn admits(&self, presented: &str) -> bool {
        let expected = self.0.as_bytes();
        let presented = presented.as_bytes();
        let mut differ = u8::from(expected.len() != presented.len());
        for (at, byte) in expected.iter().enumerate() {
            let other = presented.get(at).copied().unwrap_or(0);
            differ |= std::hint::black_box(byte ^ other);
        }
        std::hint::black_box(differ) == 0
    }
}

impl std::fmt::Debug for BearerToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("BearerToken(..)")
    }
}

/// Who may read what the server keeps for authenticated callers.
#[derive(Debug, Clone, Default)]
pub struct Access {
    /// The token `GET /extendedAgentCard` asks for as `Authorization: Bearer <token>` (A2A v1.0.1
    /// section 13.3). Without one, the extended card is served to nobody.
    pub extended_card_token: Option<BearerToken>,
}

/// The built-in agent card of the echo agent.
pub fn default_card() -> AgentCard {
    read_card(include_str!("default-card.json")).expect("the built-in agent card decodes")
}

/// An agent card from its JSON.
///
/// # Errors
///
/// The text is not JSON, or not an `AgentCard`.
pub fn read_card(text: &str) -> Result<AgentCard, String> {
    let value = a2a::json::parse(text).map_err(|error| error.to_string())?;
    a2a::server::wire::decode_a2a_discovery_agent_card(&value, "card")
        .map_err(|error| error.to_string())
}

/// Every (method, path) the server answers, and whether the generated layer serves it
/// (`bound`) or a hand route of this crate (`hand`).
pub fn served() -> Vec<(&'static str, String, &'static str)> {
    router::served()
}

/// Serves A2A on `listener` within [`Limits::DEFAULT`] and with no extended card token; see
/// [`serve_with`].
///
/// # Errors
///
/// The accepting thread cannot start.
pub fn serve(
    listener: TcpListener,
    settings: Settings,
    agent: Box<dyn Agent>,
) -> std::io::Result<()> {
    serve_with(
        listener,
        settings,
        agent,
        Limits::DEFAULT,
        Access::default(),
    )
}

/// Serves A2A on `listener` within `limits`, to the callers `access` admits. One thread
/// accepts; each connection has one that reads its request and one that writes its answer; the
/// calling thread answers the requests in the order they were read and only queues what it
/// writes, so no client's socket holds back another client. A failed accept is reported on
/// standard error and accepting goes on. Returns only if the accepting thread stops.
///
/// # Errors
///
/// The accepting thread cannot start.
pub fn serve_with(
    listener: TcpListener,
    settings: Settings,
    agent: Box<dyn Agent>,
    limits: Limits,
    access: Access,
) -> std::io::Result<()> {
    let (sender, requests) = mpsc::channel();
    connections::start(listener, sender, limits)?;
    let mut router = router::Router::new(settings, agent, limits, access);
    loop {
        match requests.recv_timeout(TICK) {
            Ok((request, peer)) => router.request(request, peer),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Ok(()),
        }
        router.prune();
    }
}
