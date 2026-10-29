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
pub use ports::ServerPorts;

use std::net::TcpListener;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use a2a::actor::Caller;
use a2a::behaviour::{Context, Generated, PushNotificationConfigStorage, TaskStorage};
use a2a::discovery::AgentCard;
use a2a::discovery::obligations::GetExtendedAgentCardBehavior;
use a2a::json::Value;
use a2a::ports::a2a_agent::A2aAgent;
use a2a::server::a2a_agent;
use a2a::server::entry::Refused;
use a2a::system::{System, SystemEvent};
use a2a::tasks::obligations::{
    CancelTaskBehavior, CreateTaskPushNotificationConfigBehavior,
    DeleteTaskPushNotificationConfigBehavior, GetTaskBehavior,
    GetTaskPushNotificationConfigBehavior, ListTaskPushNotificationConfigsBehavior,
    SendMessageBehavior, SendStreamingMessageBehavior, SubscribeToTaskBehavior,
    TaskPushNotificationConfigsQuery, TasksQuery,
};
use a2a::tasks::{TaskEntitySnapshot, TaskId};

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

/// Every obligation the generated system leaves to the ports it runs over: the clock, the two
/// stores, and the behaviours and view queries the generated crate does not implement itself
/// (`generated/rust/a2a/PLAN.md`). [`ServerPorts`] keeps them all; a wrapper of it that keeps them
/// too can stand in for it in [`InProcess`].
pub trait Obligations:
    Context
    + TaskStorage
    + PushNotificationConfigStorage
    + GetExtendedAgentCardBehavior
    + CancelTaskBehavior
    + CreateTaskPushNotificationConfigBehavior
    + DeleteTaskPushNotificationConfigBehavior
    + GetTaskBehavior
    + GetTaskPushNotificationConfigBehavior
    + ListTaskPushNotificationConfigsBehavior
    + SendMessageBehavior
    + SendStreamingMessageBehavior
    + SubscribeToTaskBehavior
    + TasksQuery
    + TaskPushNotificationConfigsQuery
{
}

impl<P> Obligations for P where
    P: Context
        + TaskStorage
        + PushNotificationConfigStorage
        + GetExtendedAgentCardBehavior
        + CancelTaskBehavior
        + CreateTaskPushNotificationConfigBehavior
        + DeleteTaskPushNotificationConfigBehavior
        + GetTaskBehavior
        + GetTaskPushNotificationConfigBehavior
        + ListTaskPushNotificationConfigsBehavior
        + SendMessageBehavior
        + SendStreamingMessageBehavior
        + SubscribeToTaskBehavior
        + TasksQuery
        + TaskPushNotificationConfigsQuery
{
}

/// The server without its HTTP binding: the generated system over [`ServerPorts`], driven call by
/// call in this process. It decides every command as [`serve_with`] does, checks the bearer token
/// for `GetExtendedAgentCard` as the binary checks the `Authorization` header (with the [`Access`]
/// of [`InProcess::with_access`]), and issues the steps an agent asks for after the call that
/// asked, through the same code as the server loop; what it publishes is read off the system's
/// log.
///
/// A trusted harness entry, not a serving surface: it runs any command or view of the generated
/// layer for any actor a grant names, the binary's caller being fixed per route
/// (`a2a.discovery.Client` for the extended card, `a2a.tasks.Client` otherwise). So it answers what
/// the binary serves on no route: the `TaskPushNotificationConfigs` view, with each
/// configuration's token and authentication. That view is scoped by its `tenant` parameter as the
/// binary scopes a route by its `/{tenant}/` prefix: a configuration is a row only under the
/// tenant it was created under, and one created without a tenant only without one. The tenant is
/// whatever the harness passes, so it is for conformance runs and tests; never put it behind a
/// network or hand it input from an untrusted caller.
///
/// The generated system runs over `P`: [`ServerPorts`] itself, or a wrapper of them
/// ([`InProcess::wrapped`]).
pub struct InProcess<P = ServerPorts> {
    system: System<Generated<P>>,
    ports: ServerPorts,
    access: Access,
}

/// What one call or issued step came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Called {
    /// The generated layer's answer (`a2a::server::a2a_agent::handle`): for a command the
    /// outcome it took, under `outcome`, with what it published under `published`, its declared
    /// error under `error` or its response under `response`, wire-named; for a view its rows,
    /// under `rows`. Or why it answered no declared outcome.
    ///
    /// One answer is [`InProcess`]'s own: `{"outcome": "unauthenticated", "published": []}`, for
    /// a `GetExtendedAgentCard` that presented no token [`Access`] admits where the server serves
    /// an extended card, which the binary answers `401 UNAUTHENTICATED`. The specification
    /// declares no such outcome: an actor grant names who may run a command, and no construct
    /// authenticates the caller as that actor (`ess/domains/discovery.yaml`).
    pub answer: Result<Value, Refused>,
    /// Every event the system published while the call ran, in log order: the command's own,
    /// then those of the steps it left the agent to ask for.
    pub events: Vec<SystemEvent>,
}

/// The outcome of a `GetExtendedAgentCard` without the token, where an extended card is served.
const UNAUTHENTICATED: &str = "unauthenticated";

impl InProcess<ServerPorts> {
    /// A server with no task and no configuration, deciding by `settings` and within `limits`,
    /// whose messages `agent` answers, and with no extended card token: as [`serve`], it serves
    /// the extended card to nobody.
    pub fn new(settings: Settings, agent: Box<dyn Agent>, limits: Limits) -> Self {
        Self::wrapped(settings, agent, limits, |ports| ports)
    }
}

impl<P: Obligations> InProcess<P> {
    /// As [`InProcess::new`], with the generated system run over `wrap` of the server's ports.
    pub fn wrapped(
        settings: Settings,
        agent: Box<dyn Agent>,
        limits: Limits,
        wrap: impl FnOnce(ServerPorts) -> P,
    ) -> Self {
        let ports = ServerPorts::new(settings, agent, limits);
        let system = System::new(A2aAgent::new(Generated::new(wrap(ports.clone()))));
        Self {
            system,
            ports,
            access: Access::default(),
        }
    }

    /// The server admitting the callers `access` admits, as [`serve_with`] does: a
    /// `GetExtendedAgentCard` is served the extended card only for the token it holds
    /// ([`InProcess::call_presenting`]).
    #[must_use]
    pub fn with_access(mut self, access: Access) -> Self {
        self.access = access;
        self
    }

    /// Decides every later call by `settings` and within `limits`; what is stored stays.
    pub fn reconfigure(&self, settings: Settings, limits: Limits) {
        self.ports.reconfigure(settings, limits);
    }

    /// Stores `task` as the server's task store holds a task, replacing one of the same id: no
    /// command runs, nothing is published and no limit is checked. For a harness that sets up a
    /// task it then reads (an authored scenario's `setup:`).
    pub fn establish_task(&mut self, task: TaskEntitySnapshot) {
        let mut ports = self.ports.clone();
        TaskStorage::put(&mut ports, task);
    }

    /// Runs the command or view `name` as `caller` with `input`, wire-named as its route reads
    /// it, under the `A2A-Version` `version` and the `A2A-Extensions` `extensions`, presenting no
    /// bearer token; then issues every step the agent asked for, in order. See
    /// [`InProcess::call_presenting`].
    ///
    /// The caller is whoever the harness names: this entry is trusted ([`InProcess`]).
    pub fn call(
        &mut self,
        caller: Option<&Caller>,
        version: Option<String>,
        extensions: Vec<String>,
        name: &str,
        input: Value,
    ) -> Called {
        self.call_presenting(caller, None, version, extensions, name, input)
    }

    /// As [`InProcess::call`], presenting the bearer token `presented`, as a request's
    /// `Authorization: Bearer <token>`. Where `name` is `GetExtendedAgentCard`, `caller` is
    /// granted it and the server serves an extended card (the card declares one and one is
    /// configured), the call is answered `unauthenticated` ([`Called::answer`]) before it runs
    /// unless the [`Access`] token admits `presented`, as the binary answers `401` before the
    /// route runs.
    ///
    /// The caller is whoever the harness names: this entry is trusted ([`InProcess`]).
    pub fn call_presenting(
        &mut self,
        caller: Option<&Caller>,
        presented: Option<&str>,
        version: Option<String>,
        extensions: Vec<String>,
        name: &str,
        input: Value,
    ) -> Called {
        if name == "a2a.discovery.GetExtendedAgentCard"
            && a2a_agent::admit(caller, name).is_ok()
            && !self.authenticated(presented)
        {
            return Called {
                answer: Ok(Value::Object(vec![
                    (
                        "outcome".to_owned(),
                        Value::Text(UNAUTHENTICATED.to_owned()),
                    ),
                    ("published".to_owned(), Value::Array(Vec::new())),
                ])),
                events: Vec::new(),
            };
        }
        self.ports.negotiate(version, extensions);
        self.ports.next_command();
        let (answer, mut events) =
            a2a_agent::handle_with_events(&mut self.system, caller, name, input);
        for work in self.ports.take_work() {
            events.extend(steps::run(&mut self.system, &self.ports, work));
        }
        Called { answer, events }
    }

    /// Whether a `GetExtendedAgentCard` presenting `presented` is served: the server serves no
    /// extended card, so the command decides, or the [`Access`] token admits `presented`.
    fn authenticated(&self, presented: Option<&str>) -> bool {
        if !self.ports.serves_extended_card() {
            return true;
        }
        match (&self.access.extended_card_token, presented) {
            (Some(token), Some(presented)) => token.admits(presented),
            _ => false,
        }
    }

    /// Issues `step` on `task` as the server loop issues an agent's step, through the same code:
    /// as its `a2a.tasks.Agent` command through the generated system, keeping what the generated
    /// behaviour leaves to the server (a status message it moved the task with, in the history;
    /// an artifact it updated, on the task). `None`, issuing nothing, where the task is gone or
    /// in a terminal state, or the step is `Work` and the task is working. Answers the input the
    /// command was issued with, wire-named, and what it came to.
    pub fn issue(&mut self, task: &TaskId, step: Step) -> Option<(Value, Called)> {
        let issued = steps::issue(&mut self.system, &self.ports, task, step)?;
        Some((
            issued.input,
            Called {
                answer: issued.answer,
                events: issued.events,
            },
        ))
    }
}
