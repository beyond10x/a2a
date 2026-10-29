//! The HTTP+JSON binding (A2A v1.0.1 section 11) over the generated HTTP layer.
//!
//! The nine routes `ess/http.yaml` binds go through `a2a::server::a2a_agent::dispatch`; the
//! fourteen it cannot bind (`ess/http-unmapped.tsv`: the `{id}:verb` routes and the `/{tenant}/`
//! twins) and the public card are thin routes here that rewrite the request onto a route the
//! generated layer serves. Every answer is read back with `crate::reread` and translated from
//! the generated envelope into the binding's: the response message itself, or a
//! `google.rpc.Status` error. Streams are server-sent events (section 11.7), fed from the events
//! the system publishes.

use std::cmp::Reverse;
use std::sync::Arc;
use std::time::Duration;

use a2a::actor::{Actor, Caller};
use a2a::behaviour::Generated;
use a2a::json::{self, Value};
use a2a::ports::a2a_agent::A2aAgent;
use a2a::server::{a2a_agent, http, wire};
use a2a::system::System;
use a2a::tasks::TaskId;

use crate::agent::Agent;
use crate::connections::{Outgoing, Peer};
use crate::paging::{self, List};
use crate::ports::{self, Mismatch, ServerPorts, Work};
use crate::reread;
use crate::steps::{self, Frame};
use crate::{Access, Limits, Settings};

/// The media type of every A2A answer but the public card's (section 11.2).
const A2A_JSON: &str = "application/a2a+json";

/// What a route does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Card,
    Send,
    Stream,
    List,
    Get,
    Cancel,
    Subscribe,
    ListConfigs,
    CreateConfig,
    GetConfig,
    DeleteConfig,
    WellKnown,
}

/// Every route without a tenant: method, path template, what it does, and whether the generated
/// layer serves it (`true`) or a hand route here. Each but the public card has a `/{tenant}/`
/// twin, served by hand.
const ROUTES: &[(&str, &str, Op, bool)] = &[
    ("GET", "/extendedAgentCard", Op::Card, true),
    ("POST", "/message:send", Op::Send, true),
    ("POST", "/message:stream", Op::Stream, true),
    ("GET", "/tasks", Op::List, true),
    ("GET", "/tasks/{id}", Op::Get, true),
    (
        "GET",
        "/tasks/{taskId}/pushNotificationConfigs",
        Op::ListConfigs,
        true,
    ),
    (
        "POST",
        "/tasks/{taskId}/pushNotificationConfigs",
        Op::CreateConfig,
        true,
    ),
    (
        "GET",
        "/tasks/{taskId}/pushNotificationConfigs/{id}",
        Op::GetConfig,
        true,
    ),
    (
        "DELETE",
        "/tasks/{taskId}/pushNotificationConfigs/{id}",
        Op::DeleteConfig,
        true,
    ),
    ("POST", "/tasks/{id}:cancel", Op::Cancel, false),
    ("GET", "/tasks/{id}:subscribe", Op::Subscribe, false),
    ("GET", "/.well-known/agent-card.json", Op::WellKnown, false),
];

/// The `A2A-Version` service parameter (section 3.6.2), as a header or a query parameter.
const VERSION: &str = "A2A-Version";
/// The `A2A-Extensions` service parameter (section 3.6.1).
const EXTENSIONS: &str = "A2A-Extensions";

/// The lifecycle name of each wire state, as the `Tasks` view filters by it.
// ESS gap (ess 0.67.0): a view parameter typed by a lifecycle state reads the state's name, and
// no construct maps the wire enum `TaskState` (`TASK_STATE_*`) onto those states
// (`ess/domains/tasks.yaml`, `Tasks` param `status`), so the map is kept here.
const STATES: &[(&str, &str)] = &[
    ("TASK_STATE_SUBMITTED", "Submitted"),
    ("TASK_STATE_WORKING", "Working"),
    ("TASK_STATE_COMPLETED", "Completed"),
    ("TASK_STATE_FAILED", "Failed"),
    ("TASK_STATE_CANCELED", "Canceled"),
    ("TASK_STATE_INPUT_REQUIRED", "InputRequired"),
    ("TASK_STATE_REJECTED", "Rejected"),
    ("TASK_STATE_AUTH_REQUIRED", "AuthRequired"),
];

/// Every served route: the table, then the tenant twins.
fn table() -> Vec<(&'static str, String, Op, bool)> {
    let direct = ROUTES
        .iter()
        .map(|(method, path, op, bound)| (*method, (*path).to_owned(), *op, *bound));
    let tenant = ROUTES
        .iter()
        .filter(|(_, _, op, _)| *op != Op::WellKnown)
        .map(|(method, path, op, _)| (*method, format!("/{{tenant}}{path}"), *op, false));
    direct.chain(tenant).collect()
}

/// Every (method, path) served, and whether it is `bound` or `hand`.
pub(crate) fn served() -> Vec<(&'static str, String, &'static str)> {
    table()
        .into_iter()
        .map(|(method, path, _, bound)| (method, path, if bound { "bound" } else { "hand" }))
        .collect()
}

/// The variables of `path` where it matches `template`, by name and still percent-encoded.
///
/// A template segment is a literal, or a `{name}` followed by an optional literal suffix such as
/// `:cancel`. A variable holds a non-empty segment; a whole-segment variable does not hold one
/// ending in a verb suffix, so `/tasks/{id}` leaves `/tasks/t-1:cancel` to the cancel route.
fn fit(template: &str, path: &str) -> Option<Vec<(String, String)>> {
    let template: Vec<&str> = template.strip_prefix('/')?.split('/').collect();
    let given: Vec<&str> = path.strip_prefix('/')?.split('/').collect();
    if template.len() != given.len() {
        return None;
    }
    let mut found = Vec::new();
    for (expected, text) in template.iter().zip(given) {
        match expected
            .strip_prefix('{')
            .and_then(|rest| rest.split_once('}'))
        {
            Some((name, suffix)) => {
                let value = text.strip_suffix(suffix)?;
                if value.is_empty()
                    || (suffix.is_empty()
                        && (value.ends_with(":cancel") || value.ends_with(":subscribe")))
                {
                    return None;
                }
                found.push((name.to_owned(), value.to_owned()));
            }
            None if *expected == text => {}
            None => return None,
        }
    }
    Some(found)
}

/// `text` with every `%XX` decoded, and `+` as a space where `form`; `None` where it does not
/// decode to UTF-8.
fn decoded(text: &str, form: bool) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' if form => out.push(b' '),
            b'%' => {
                let high = char::from(*bytes.get(index + 1)?).to_digit(16)?;
                let low = char::from(*bytes.get(index + 2)?).to_digit(16)?;
                out.push(u8::try_from(high * 16 + low).ok()?);
                index += 2;
            }
            byte => out.push(byte),
        }
        index += 1;
    }
    String::from_utf8(out).ok()
}

/// The pairs of a query string, still encoded.
fn pairs(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            (key.to_owned(), value.to_owned())
        })
        .collect()
}

fn joined(pairs: &[(String, String)]) -> String {
    pairs
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

/// A `google.rpc.Status` error answer (section 11.6), with an `ErrorInfo` naming `reason` where
/// the error is an A2A one.
// ESS gap (ess 0.67.0): the generated layer answers a declared error as its own envelope
// (`{"outcome", "published", "error": "<qualified name>"}`) with status 404, 409 or 502, and a refusal
// as `{"refused": …}`; `ess/http.yaml` has no construct for an error's HTTP status nor for a
// `google.rpc.Status` body with `ErrorInfo`, so both are written here and in `declared_error`.
fn status_error(code: u16, status: &str, reason: Option<&str>, message: &str) -> http::Response {
    let mut body = String::from("{");
    json::member(&mut body, "error");
    body.push('{');
    json::member(&mut body, "code");
    json::push_integer(&mut body, i64::from(code));
    json::member(&mut body, "status");
    json::push_text(&mut body, status);
    json::member(&mut body, "message");
    json::push_text(&mut body, message);
    json::member(&mut body, "details");
    body.push('[');
    if let Some(reason) = reason {
        body.push('{');
        json::member(&mut body, "@type");
        json::push_text(&mut body, "type.googleapis.com/google.rpc.ErrorInfo");
        json::member(&mut body, "reason");
        json::push_text(&mut body, reason);
        json::member(&mut body, "domain");
        json::push_text(&mut body, "a2a-protocol.org");
        body.push('}');
    }
    body.push(']');
    body.push('}');
    body.push('}');
    http::Response::new(code, A2A_JSON, body)
}

fn invalid(message: &str) -> http::Response {
    status_error(400, "INVALID_ARGUMENT", None, message)
}

/// The error answer for a declared error of the model, by its qualified name.
// ESS gap (ess 0.67.0): no construct gives a declared error its HTTP status, `google.rpc.Code`
// and A2A reason (section 11.6), so this table holds them; see `status_error`.
fn declared_error(name: &str) -> http::Response {
    let short = name.rsplit('.').next().unwrap_or(name);
    let (code, status, reason, message) = match short {
        "TaskNotFound" => (
            404,
            "NOT_FOUND",
            Some("TASK_NOT_FOUND"),
            "the task was not found",
        ),
        "TaskNotCancelable" => (
            400,
            "FAILED_PRECONDITION",
            Some("TASK_NOT_CANCELABLE"),
            "the task cannot be canceled",
        ),
        "PushNotificationNotSupported" => (
            400,
            "FAILED_PRECONDITION",
            Some("PUSH_NOTIFICATION_NOT_SUPPORTED"),
            "the agent does not support push notifications",
        ),
        "UnsupportedOperation" => (
            400,
            "FAILED_PRECONDITION",
            Some("UNSUPPORTED_OPERATION"),
            "the operation is not supported",
        ),
        "ExtendedAgentCardNotConfigured" => (
            400,
            "FAILED_PRECONDITION",
            Some("EXTENDED_AGENT_CARD_NOT_CONFIGURED"),
            "the agent has no extended agent card configured",
        ),
        "ExtensionSupportRequired" => (
            400,
            "FAILED_PRECONDITION",
            Some("EXTENSION_SUPPORT_REQUIRED"),
            "the agent requires an extension the request does not declare",
        ),
        "VersionNotSupported" => (
            400,
            "FAILED_PRECONDITION",
            Some("VERSION_NOT_SUPPORTED"),
            "the agent does not support the requested protocol version",
        ),
        "ContentTypeNotSupported" => (
            400,
            "INVALID_ARGUMENT",
            Some("CONTENT_TYPE_NOT_SUPPORTED"),
            "a part's media type is not supported",
        ),
        "ContextMismatch" => (
            400,
            "INVALID_ARGUMENT",
            Some("CONTEXT_MISMATCH"),
            "the message's context is not its task's",
        ),
        "ContextNotAccepted" => (
            400,
            "INVALID_ARGUMENT",
            Some("CONTEXT_NOT_ACCEPTED"),
            "the agent cannot accept the message's context",
        ),
        // Not an A2A error, so no `ErrorInfo` reason: a system error (section 3.3.2) for a
        // server limit (section 13.4, "SHOULD implement rate limiting").
        "ResourceExhausted" => (
            429,
            "RESOURCE_EXHAUSTED",
            None,
            "the request would take more than the server keeps for one task or tenant",
        ),
        _ => {
            return status_error(500, "INTERNAL", None, &format!("undeclared error `{name}`"));
        }
    };
    status_error(code, status, reason, message)
}

/// The error answer for a refusal the generated layer makes about the request itself, at the
/// status it refused with: a refusal of the request's form or size keeps its 4xx status.
fn refusal_error(status: u16, detail: &str) -> http::Response {
    match status {
        400 => invalid(detail),
        401 => status_error(401, "UNAUTHENTICATED", None, detail),
        403 => status_error(403, "PERMISSION_DENIED", None, detail),
        404 => status_error(404, "NOT_FOUND", None, detail),
        405 | 501 => status_error(status, "UNIMPLEMENTED", None, detail),
        408 => status_error(408, "DEADLINE_EXCEEDED", None, detail),
        429 => status_error(429, "RESOURCE_EXHAUSTED", None, detail),
        402..=499 => status_error(status, "INVALID_ARGUMENT", None, detail),
        _ => status_error(500, "INTERNAL", None, detail),
    }
}

/// A refusal the generated layer wrote, as an error answer: its detail from its JSON body, or
/// the status's reason phrase where the body is none (the generated `431` for a head over its
/// limit is `text/plain`).
pub(crate) fn refused_answer(response: &http::Response) -> http::Response {
    let detail = json::parse(&response.body)
        .ok()
        .and_then(|value| match value.member("refused") {
            Some(Value::Text(detail)) => Some(detail.clone()),
            _ => None,
        })
        .unwrap_or_else(|| http::reason(response.status).to_owned());
    refusal_error(response.status, &detail)
}

/// The answer to a request not in within `timeout`.
// ESS gap (ess 0.67.0): the generated `http::read` bounds each read by `READ_TIMEOUT`, not the
// whole request, so a client sending a byte a second holds its connection without end; the
// generated layer has no request deadline and no 408 answer, so `connections` keeps one by hand.
pub(crate) fn late_answer(timeout: Duration) -> http::Response {
    status_error(
        408,
        "DEADLINE_EXCEEDED",
        None,
        &format!("the request did not arrive within {} s", timeout.as_secs()),
    )
}

fn text_of(value: &Value) -> String {
    let mut out = String::new();
    json::push_value(&mut out, value);
    out
}

/// What a request comes to.
enum Answer {
    /// One answer, then the connection closes.
    Plain(http::Response),
    /// A server-sent event stream: its first event, and the task it then follows, with whether
    /// it ends at an interrupted state too.
    Stream(String, Option<(TaskId, bool)>),
}

/// A stream following a task.
struct Follower {
    task: TaskId,
    peer: Peer,
    ends_on_interrupt: bool,
}

/// The head of a server-sent event stream (section 11.7).
const STREAM_HEAD: &str = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n";

/// The server: the generated system over [`ServerPorts`], the open streams, the steps left to
/// issue once the answer is out, and the frames waiting for the streams.
pub(crate) struct Router {
    system: System<Generated<ServerPorts>>,
    ports: ServerPorts,
    limits: Limits,
    access: Access,
    followers: Vec<Follower>,
    later: Vec<Work>,
    frames: Vec<Frame>,
}

impl Router {
    pub(crate) fn new(
        settings: Settings,
        agent: Box<dyn Agent>,
        limits: Limits,
        access: Access,
    ) -> Self {
        let ports = ServerPorts::new(settings, agent, limits);
        let system = System::new(A2aAgent::new(Generated::new(ports.clone())));
        Self {
            system,
            ports,
            limits,
            access,
            followers: Vec::new(),
            later: Vec::new(),
            frames: Vec::new(),
        }
    }

    /// Answers one request read off `peer`'s connection, then issues the steps the answer did not
    /// wait for and delivers what the system published to the open streams. Every answer and
    /// event is queued for the connection's writer, so no socket holds this thread.
    pub(crate) fn request(&mut self, request: http::Request, peer: Peer) {
        match self.answer(&request) {
            Answer::Plain(response) => {
                peer.offer(Outgoing::Answer(response));
            }
            Answer::Stream(first, follow) => {
                let opened = peer.offer(Outgoing::Text(Arc::from(format!(
                    "{STREAM_HEAD}{}",
                    event(&first)
                ))));
                if let (true, Some((task, ends_on_interrupt))) = (opened, follow) {
                    self.followers.push(Follower {
                        task,
                        peer,
                        ends_on_interrupt,
                    });
                }
            }
        }
        self.later.extend(self.ports.take_work());
        for work in std::mem::take(&mut self.later) {
            let frames = steps::frames(steps::run(&mut self.system, &self.ports, work));
            self.frames.extend(frames);
        }
        self.deliver();
    }

    /// Drops every stream whose client hung up: its socket read end of file or failed, after
    /// whatever the client sent once its request was in. A client that hangs up before an event
    /// for its task comes otherwise holds its socket for good.
    pub(crate) fn prune(&mut self) {
        self.followers.retain(|follower| {
            let gone = follower.peer.gone();
            if gone {
                follower.peer.cut();
            }
            !gone
        });
    }

    /// Issues the steps of every message whose answer waits for them, and keeps the rest for
    /// after the answer. Answers the task and requested history length of each awaited one.
    fn settle(&mut self) -> Vec<(TaskId, Option<i64>)> {
        let mut awaited = Vec::new();
        for work in self.ports.take_work() {
            if work.awaited {
                awaited.push((work.task.clone(), work.history_length));
                let frames = steps::frames(steps::run(&mut self.system, &self.ports, work));
                self.frames.extend(frames);
            } else {
                self.later.push(work);
            }
        }
        awaited
    }

    /// Queues every frame for the streams following its task, and closes each stream at its end.
    /// A stream whose writer's backlog is full, its client reading more slowly than the events
    /// come, is dropped and closed.
    fn deliver(&mut self) {
        for (task, update) in std::mem::take(&mut self.frames) {
            let mut frame = String::new();
            wire::encode_event_a2a_tasks_message_stream_response(&update, &mut frame);
            let frame: Arc<str> = Arc::from(event(&frame));
            let state = update
                .status_update
                .as_ref()
                .map(|status| ports::lifecycle_state(status.status.state));
            self.followers.retain(|follower| {
                if follower.task != task {
                    return true;
                }
                if !follower.peer.offer(Outgoing::Text(Arc::clone(&frame))) {
                    follower.peer.cut();
                    return false;
                }
                !state.is_some_and(|state| ends(state, follower.ends_on_interrupt))
            });
        }
        let ports = &self.ports;
        self.followers.retain(|follower| {
            ports
                .state_of(&follower.task)
                .is_some_and(|(state, _)| !ends(state, follower.ends_on_interrupt))
        });
    }

    /// The refusal of one more stream where it would pass [`Limits::max_subscribers`], or
    /// [`Limits::max_subscribers_per_task`] for `task`, a task a request under `tenant` reaches.
    /// Streams whose client hung up are dropped first and do not count. A task the request does
    /// not reach is left to the generated layer, which answers it as not found.
    // ESS gap (ess 0.67.0): a model declares no bound on how many streams follow a task, nor an
    // error for a request over one, so both bounds and their 429 are kept here.
    fn crowded(&mut self, task: Option<&TaskId>, tenant: Option<&str>) -> Option<http::Response> {
        self.prune();
        if self.followers.len() >= self.limits.max_subscribers {
            return Some(exhausted(&format!(
                "the server follows {} tasks' streams at most",
                self.limits.max_subscribers
            )));
        }
        let task = task.filter(|task| self.ports.reachable(task, tenant))?;
        let following = self
            .followers
            .iter()
            .filter(|follower| follower.task == *task)
            .count();
        (following >= self.limits.max_subscribers_per_task).then(|| {
            exhausted(&format!(
                "the task has {} streams, the most it may have",
                self.limits.max_subscribers_per_task
            ))
        })
    }

    /// The refusal of a `GetExtendedAgentCard` without the token [`Access`] holds, where the
    /// server serves an extended card (the card declares one and one is configured). A2A v1.0.1
    /// section 13.3: the extended card is for authenticated clients.
    fn unauthenticated(&self, request: &http::Request) -> Option<http::Response> {
        if !self.ports.serves_extended_card() {
            return None;
        }
        let presented = request
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("Authorization"))
            .and_then(|(_, value)| bearer(value));
        let admitted = match (&self.access.extended_card_token, presented) {
            (Some(token), Some(presented)) => token.admits(presented),
            _ => false,
        };
        if admitted {
            return None;
        }
        let mut answer = status_error(
            401,
            "UNAUTHENTICATED",
            None,
            "the extended agent card is served for a bearer token",
        );
        answer
            .headers
            .push(("WWW-Authenticate".to_owned(), "Bearer".to_owned()));
        Some(answer)
    }

    fn answer(&mut self, request: &http::Request) -> Answer {
        let mut allowed: Vec<&str> = Vec::new();
        let mut chosen = None;
        for (method, template, op, _) in table() {
            let Some(found) = fit(&template, &request.path) else {
                continue;
            };
            if method == request.method {
                chosen = Some((op, found));
                break;
            }
            allowed.push(method);
        }
        let Some((op, found)) = chosen else {
            if allowed.is_empty() {
                return Answer::Plain(status_error(
                    404,
                    "NOT_FOUND",
                    None,
                    &format!("no operation is served at `{}`", request.path),
                ));
            }
            allowed.sort_unstable();
            allowed.dedup();
            let allow = allowed.join(", ");
            let mut answer = status_error(
                405,
                "UNIMPLEMENTED",
                None,
                &format!("`{}` answers {allow}", request.path),
            );
            answer.headers.push(("Allow".to_owned(), allow));
            return Answer::Plain(answer);
        };
        if op == Op::WellKnown {
            let mut body = String::new();
            wire::encode_a2a_discovery_agent_card(self.ports.card(), &mut body);
            return Answer::Plain(http::Response::new(200, http::JSON, body));
        }
        // ESS gap (ess 0.67.0): an actor grant names who may run a command, and no construct
        // authenticates the caller as that actor (`ess/domains/discovery.yaml`,
        // `GetExtendedAgentCard`), so the bearer token is checked here.
        if op == Op::Card
            && let Some(refusal) = self.unauthenticated(request)
        {
            return Answer::Plain(refusal);
        }
        let mut variables = Vec::new();
        let mut tenant = None;
        for (name, raw) in found {
            if name == "tenant" {
                match decoded(&raw, false) {
                    Some(text) => tenant = Some(text),
                    None => {
                        return Answer::Plain(invalid("the tenant is not percent-encoded UTF-8"));
                    }
                }
            } else {
                variables.push((name, raw));
            }
        }
        let mut request = request.clone();
        self.negotiate(&mut request);
        // ESS gap (ess 0.67.0): no route of `ess/http.yaml` carries a `/{tenant}/` twin, since an
        // operation has one route (ESS-HTTP-005) and a path segment cannot carry the optional
        // `input.tenant` (ESS-HTTP-009). The twin is matched here and its tenant moved into the
        // query (get) or the body (post, delete) of the bound route.
        //
        // The tenant is the path's alone: a plain route has none, whatever its query or body
        // says, so it reaches only the tasks made on plain routes (`ess/domains/tasks.yaml`,
        // GetTask `other-tenant`).
        if request.method == "GET" {
            let mut kept: Vec<_> = pairs(&request.query)
                .into_iter()
                .filter(|(key, _)| decoded(key, true).as_deref() != Some("tenant"))
                .collect();
            if let Some(tenant) = &tenant {
                kept.push(("tenant".to_owned(), http::form_encoded(tenant)));
            }
            request.query = joined(&kept);
        } else {
            match &tenant {
                Some(tenant) => with_member(&mut request.body, "tenant", tenant),
                None => without_member(&mut request.body, "tenant"),
            }
        }
        let direct = ROUTES
            .iter()
            .find(|(method, _, route, bound)| *route == op && *bound && *method == request.method);
        if let Some((_, template, _, _)) = direct {
            let mut path = (*template).to_owned();
            for (name, raw) in &variables {
                path = path.replace(&format!("{{{name}}}"), raw);
            }
            request.path = path;
        }
        if let Some(refusal) = precheck(op, &request) {
            return Answer::Plain(refusal);
        }
        let follows = match op {
            Op::Subscribe => Some(
                variables
                    .iter()
                    .find(|(name, _)| name == "id")
                    .and_then(|(_, raw)| decoded(raw, false))
                    .map(TaskId),
            ),
            Op::Stream => Some(http::json_body(&request.body, false).ok().and_then(|body| {
                match body.member("message")?.member("taskId")? {
                    Value::Text(id) => Some(TaskId(id.clone())),
                    _ => None,
                }
            })),
            _ => None,
        };
        if let Some(task) = follows
            && let Some(refusal) = self.crowded(task.as_ref(), tenant.as_deref())
        {
            return Answer::Plain(refusal);
        }
        match op {
            Op::List => Answer::Plain(self.list(&request)),
            Op::Cancel | Op::Subscribe => self.by_hand(op, &request, &variables),
            Op::Stream => {
                let response = self.dispatch(op, &request);
                self.stream(response, true)
            }
            Op::Send => {
                let response = self.dispatch(op, &request);
                let awaited = self.settle();
                let answer = self.reply(op, response.status, response.body.as_str());
                match awaited.last() {
                    Some((task, history_length)) if answer.status == 200 => {
                        Answer::Plain(self.settled(task, *history_length).unwrap_or(answer))
                    }
                    _ => Answer::Plain(answer),
                }
            }
            _ => {
                let response = self.dispatch(op, &request);
                Answer::Plain(self.reply(op, response.status, response.body.as_str()))
            }
        }
    }

    /// Takes the service parameters off the request and hands them to the ports.
    fn negotiate(&self, request: &mut http::Request) {
        let mut version = request
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(VERSION))
            .map(|(_, value)| value.clone());
        let mut extensions: Vec<String> = request
            .headers
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case(EXTENSIONS))
            .map(|(_, value)| value.clone())
            .collect();
        let mut kept = Vec::new();
        for (key, value) in pairs(&request.query) {
            let name = decoded(&key, true).unwrap_or_default();
            if name.eq_ignore_ascii_case(VERSION) {
                if version.is_none() {
                    version = decoded(&value, true);
                }
            } else if name.eq_ignore_ascii_case(EXTENSIONS) {
                extensions.extend(decoded(&value, true));
            } else {
                kept.push((key, value));
            }
        }
        request.query = joined(&kept);
        let extensions = extensions
            .iter()
            .flat_map(|value| value.split(','))
            .map(str::trim)
            .filter(|uri| !uri.is_empty())
            .map(str::to_owned)
            .collect();
        self.ports.negotiate(version, extensions);
    }

    fn caller(op: Op) -> Caller {
        Caller {
            actor: if op == Op::Card {
                Actor::DiscoveryClient
            } else {
                Actor::TasksClient
            },
        }
    }

    /// Runs a bound route through the generated layer, and keeps the status events the command
    /// published for the streams.
    fn dispatch(&mut self, op: Op, request: &http::Request) -> http::Response {
        self.ports.next_command();
        let response = a2a_agent::dispatch(&mut self.system, Some(&Self::caller(op)), request);
        // ESS gap (ess 0.67.0): the generated `dispatch` pumps the system and takes its log
        // itself, handing the adapter no typed events, so a stream's status frames (a cancel's
        // CANCELED among them) are read back from the `published` list of the JSON envelope.
        if let Ok(envelope) = reread::parse(&response.body) {
            self.frames.extend(steps::published(&envelope));
        }
        response
    }

    /// A blocking `SendMessage`'s answer once the steps it waited for ran: the task as they left
    /// it. The generated answer was written when the command ran, before its steps.
    // ESS gap (ess 0.67.0): a command's response is written when the command runs; no construct
    // answers after the system-issued steps it starts (A2A section 3.2.2, a blocking send returns
    // the task "in a terminal or interrupted state"), so the answer is rebuilt here.
    fn settled(&self, task: &TaskId, history_length: Option<i64>) -> Option<http::Response> {
        let task = self.ports.snapshot(task, history_length, true)?;
        let mut body = String::from("{");
        json::member(&mut body, "task");
        wire::encode_a2a_tasks_task_snapshot(&task, &mut body);
        body.push('}');
        Some(http::Response::new(200, A2A_JSON, body))
    }

    /// A generated answer as the binding's: the response message, or an error.
    fn reply(&self, op: Op, status: u16, body: &str) -> http::Response {
        let Ok(envelope) = reread::parse(body) else {
            return status_error(
                500,
                "INTERNAL",
                None,
                "the generated layer answered no JSON",
            );
        };
        self.translate(op, status, &envelope)
    }

    fn translate(&self, op: Op, status: u16, envelope: &Value) -> http::Response {
        if let Some(Value::Text(name)) = envelope.member("error") {
            return declared_error(name);
        }
        if let Some(Value::Text(detail)) = envelope.member("refused") {
            return refusal_error(status, detail);
        }
        if op == Op::DeleteConfig && envelope.member("outcome").is_some() {
            return http::Response::new(200, A2A_JSON, "{}");
        }
        let Some(response) = envelope.member("response") else {
            return status_error(
                500,
                "INTERNAL",
                None,
                "the generated layer answered no response",
            );
        };
        let inner = match op {
            Op::Get | Op::Cancel | Op::Subscribe => response.member("task"),
            Op::CreateConfig | Op::GetConfig => response.member("config"),
            Op::Card => response.member("card"),
            _ => Some(response),
        };
        match inner {
            Some(inner) => http::Response::new(200, A2A_JSON, text_of(inner)),
            None => status_error(
                500,
                "INTERNAL",
                None,
                "the generated response is incomplete",
            ),
        }
    }

    /// `CancelTask` and `SubscribeToTask`, whose `{id}:verb` routes `ess/http.yaml` cannot bind:
    /// their input built from the path, the body or query, and run through the generated layer's
    /// command route for each, whose answer is read as a bound route's.
    // ESS gap (ess 0.67.0): an `ess/http.yaml` path template cannot carry a verb suffix after a
    // variable segment (`/tasks/{id}:cancel`), so these two routes are read here and sent on to
    // the generated command routes (`/tasks/commands/tasks:cancel`, `…:subscribe`), and their
    // status frames read back from the envelope as in `dispatch`. The generated `handle` is not
    // used: it reads its own answer back with the intake's depth limit
    // (`generated/rust/a2a/src/server/entry.rs:77`, `entry::read`) and panics on an answer deeper
    // than that (see `crate::reread`).
    fn by_hand(
        &mut self,
        op: Op,
        request: &http::Request,
        variables: &[(String, String)],
    ) -> Answer {
        let mut members = match op {
            Op::Cancel => match http::json_body(&request.body, false) {
                Ok(Value::Object(members)) => members,
                Ok(_) => return Answer::Plain(invalid("body: expected an object")),
                Err(detail) => return Answer::Plain(invalid(&detail)),
            },
            _ => {
                let mut members = Vec::new();
                for (key, value) in pairs(&request.query) {
                    let key = decoded(&key, true).unwrap_or_default();
                    let Some(value) = decoded(&value, true) else {
                        return Answer::Plain(invalid(&format!(
                            "query.{key}: not percent-encoded UTF-8"
                        )));
                    };
                    // specification.md:1275 "SHOULD ignore unrecognized fields".
                    if key == "tenant" {
                        members.push((key, Value::Text(value)));
                    }
                }
                members
            }
        };
        for (name, raw) in variables {
            if members.iter().any(|(key, _)| key == name) {
                return Answer::Plain(invalid(&format!(
                    "body.{name}: the path carries this field, so the request does not"
                )));
            }
            let Some(value) = decoded(raw, false) else {
                return Answer::Plain(invalid(&format!("path.{name}: not percent-encoded UTF-8")));
            };
            members.push((name.clone(), Value::Text(value)));
        }
        let path = if op == Op::Cancel {
            "/tasks/commands/tasks:cancel"
        } else {
            "/tasks/commands/tasks:subscribe"
        };
        let command = http::Request {
            method: "POST".to_owned(),
            path: path.to_owned(),
            query: String::new(),
            headers: request.headers.clone(),
            body: text_of(&Value::Object(members)).into_bytes(),
        };
        let response = self.dispatch(op, &command);
        if op == Op::Cancel {
            return Answer::Plain(self.reply(op, response.status, &response.body));
        }
        self.stream(response, false)
    }

    /// A stream's opening: an error answer before it opens, or its first event, which is the
    /// response itself (`{"task": …}` or `{"message": …}`).
    fn stream(&self, response: http::Response, ends_on_interrupt: bool) -> Answer {
        let Ok(envelope) = reread::parse(&response.body) else {
            return Answer::Plain(status_error(
                500,
                "INTERNAL",
                None,
                "the generated layer answered no JSON",
            ));
        };
        let first = match envelope.member("response") {
            Some(first) if envelope.member("error").is_none() => first,
            _ => return Answer::Plain(self.translate(Op::Stream, response.status, &envelope)),
        };
        let follow = match first.member("task").and_then(|task| task.member("id")) {
            Some(Value::Text(id)) => Some((TaskId(id.clone()), ends_on_interrupt)),
            _ => None,
        };
        Answer::Stream(text_of(first), follow)
    }

    /// `ListTasks`: its request read and checked here, the rows read from the bound `Tasks` view,
    /// then ordered, paged and trimmed as section 3.1.4 says.
    // ESS gap (ess 0.67.0): the `Tasks` view in `ess/domains/tasks.yaml` has no order synthesis
    // accepts, no cursor paging, no view field omitted by a parameter and no refusal before its
    // rows, so ordering, paging, `historyLength`, `includeArtifacts` and the negotiation
    // refusals are done here.
    fn list(&mut self, request: &http::Request) -> http::Response {
        match self.ports.mismatch() {
            Some(Mismatch::Version) => return declared_error("VersionNotSupported"),
            Some(Mismatch::Extension) => return declared_error("ExtensionSupportRequired"),
            None => {}
        }
        let mut context = None;
        let mut status = None;
        let mut page_size = 50_usize;
        let mut page_token = None;
        let mut history_length = None;
        let mut after = None;
        let mut include_artifacts = false;
        let mut tenant = None;
        for (key, value) in pairs(&request.query) {
            let key = decoded(&key, true).unwrap_or_default();
            let Some(value) = decoded(&value, true) else {
                return invalid(&format!("query.{key}: not percent-encoded UTF-8"));
            };
            match key.as_str() {
                "tenant" => tenant = Some(value),
                "contextId" => context = Some(value),
                "status" => match STATES.iter().find(|(wire, _)| *wire == value) {
                    Some((_, name)) => status = Some(*name),
                    None => return invalid(&format!("query.status: not a task state: `{value}`")),
                },
                "pageSize" => match value.parse::<usize>() {
                    Ok(size) if (1..=100).contains(&size) => page_size = size,
                    _ => return invalid("query.pageSize: expected a whole number from 1 to 100"),
                },
                "pageToken" if value.is_empty() => page_token = None,
                "pageToken" => match cursor(&value) {
                    Some(after) => page_token = Some(after),
                    None => return invalid("query.pageToken: not a token this server issued"),
                },
                "historyLength" => match value.parse::<i64>() {
                    Ok(length) if length >= 0 => history_length = Some(length),
                    _ => {
                        return invalid(
                            "query.historyLength: expected a whole number of 0 or more",
                        );
                    }
                },
                "statusTimestampAfter" => {
                    if ports::instant(&value).is_none() {
                        return invalid(
                            "query.statusTimestampAfter: expected an RFC 3339 timestamp",
                        );
                    }
                    after = Some(value);
                }
                "includeArtifacts" => match value.as_str() {
                    "true" => include_artifacts = true,
                    "false" => include_artifacts = false,
                    _ => return invalid("query.includeArtifacts: expected true or false"),
                },
                // specification.md:1275 "SHOULD ignore unrecognized fields".
                _ => {}
            }
        }
        let mut query = Vec::new();
        if let Some(context) = &context {
            query.push(("contextId".to_owned(), http::form_encoded(context)));
        }
        if let Some(status) = status {
            query.push(("status".to_owned(), status.to_owned()));
        }
        if let Some(after) = &after {
            query.push(("statusTimestampAfter".to_owned(), http::form_encoded(after)));
        }
        if let Some(tenant) = &tenant {
            query.push(("tenant".to_owned(), http::form_encoded(tenant)));
        }
        let view = http::Request {
            method: "GET".to_owned(),
            path: "/tasks".to_owned(),
            query: joined(&query),
            headers: request.headers.clone(),
            body: Vec::new(),
        };
        let response = self.dispatch(Op::List, &view);
        if response.status != 200 {
            return refused_answer(&response);
        }
        let rows = match reread::parse(&response.body) {
            Ok(Value::Object(mut members)) => {
                match members
                    .iter()
                    .position(|(name, _)| name == "rows")
                    .map(|at| members.swap_remove(at).1)
                {
                    Some(Value::Array(rows)) => rows,
                    _ => {
                        return status_error(
                            500,
                            "INTERNAL",
                            None,
                            "the Tasks view answered no rows",
                        );
                    }
                }
            }
            _ => {
                return status_error(500, "INTERNAL", None, "the Tasks view answered no rows");
            }
        };
        let mut keys = Vec::with_capacity(rows.len());
        for row in &rows {
            match key_of(row) {
                Ok(key) => keys.push(key),
                Err(detail) => return status_error(500, "INTERNAL", None, detail),
            }
        }
        keys.sort_unstable();
        let total = keys.len();
        let size = i64::try_from(page_size).unwrap_or(i64::MAX);
        let start = page_token.map_or(0, |after| keys.partition_point(|key| key.order <= after));
        let end = start.saturating_add(page_size).min(total);
        let next = match keys.get(..end).and_then(<[_]>::last) {
            Some(last) if end < total => token(last),
            _ => String::new(),
        };
        let mut body = String::from("{");
        json::member(&mut body, "tasks");
        body.push('[');
        let mut written = 0_usize;
        for key in keys.get(start..end).unwrap_or_default() {
            let Some(task) = self.ports.snapshot(
                &TaskId(key.order.1.clone()),
                history_length,
                include_artifacts,
            ) else {
                continue;
            };
            if written > 0 {
                body.push(',');
            }
            written += 1;
            wire::encode_a2a_tasks_task_snapshot(&task, &mut body);
        }
        body.push(']');
        json::member(&mut body, "nextPageToken");
        json::push_text(&mut body, &next);
        json::member(&mut body, "pageSize");
        json::push_integer(&mut body, size);
        json::member(&mut body, "totalSize");
        json::push_integer(&mut body, i64::try_from(total).unwrap_or(i64::MAX));
        body.push('}');
        http::Response::new(200, A2A_JSON, body)
    }
}

/// Where a task stands in `ListTasks`: newest status first (section 3.1.4), then by id.
type Order = (Reverse<Option<time::OffsetDateTime>>, String);

/// A task in `ListTasks` as its row of the `Tasks` view gives it: where it stands, and its status
/// timestamp as stored, which its page token carries. Tasks are ordered and paged by these
/// alone, and only the page's tasks are read whole.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    order: Order,
    stamp: String,
}

/// The key of a row of the `Tasks` view.
fn key_of(row: &Value) -> Result<Key, &'static str> {
    let Some(Value::Text(id)) = row.member("id") else {
        return Err("a row of the Tasks view without an id");
    };
    let stamp = match row.member("statusTimestamp") {
        Some(Value::Text(stamp)) => stamp.clone(),
        _ => String::new(),
    };
    let at = ports::instant(&stamp);
    Ok(Key {
        order: (Reverse(at), id.clone()),
        stamp,
    })
}

/// The page token naming the task of `key` as the last of its page: its status timestamp and its
/// id.
fn token(key: &Key) -> String {
    paging::token(List::Tasks, &format!("{}\n{}", key.stamp, key.order.1))
}

/// Where the page a token names starts after; `None` where this server issued no such token: not
/// a `ListTasks` token, or a key no task has (no id, or a timestamp that is not RFC 3339).
fn cursor(token: &str) -> Option<Order> {
    let key = paging::key(List::Tasks, token)?;
    let (stamp, id) = key.split_once('\n')?;
    if id.is_empty() {
        return None;
    }
    let at = if stamp.is_empty() {
        None
    } else {
        Some(ports::instant(stamp)?)
    };
    Some((Reverse(at), id.to_owned()))
}

/// The refusal of a request the generated layer would take though the binding forbids it.
///
/// `historyLength` is "0 or more" on every operation (specification.md:467, "consistent semantics
/// across all operations"), and a page token is one this server issued (section 3.1.4).
fn precheck(op: Op, request: &http::Request) -> Option<http::Response> {
    let negative = |value: Option<&Value>| match value {
        Some(Value::Number(number)) => number.trim_start().starts_with('-'),
        _ => false,
    };
    match op {
        Op::Get => pairs(&request.query).into_iter().find_map(|(key, value)| {
            let negative = decoded(&key, true).as_deref() == Some("historyLength")
                && decoded(&value, true).is_some_and(|value| value.trim().starts_with('-'));
            negative.then(|| invalid("query.historyLength: expected a whole number of 0 or more"))
        }),
        Op::Send | Op::Stream => {
            let body = http::json_body(&request.body, false).ok()?;
            let length = body
                .member("configuration")
                .and_then(|configuration| configuration.member("historyLength"));
            negative(length).then(|| {
                invalid("body.configuration.historyLength: expected a whole number of 0 or more")
            })
        }
        Op::ListConfigs => pairs(&request.query).into_iter().find_map(|(key, value)| {
            let foreign = decoded(&key, true).as_deref() == Some("pageToken")
                && decoded(&value, true).is_some_and(|value| {
                    !value.is_empty() && paging::key(List::Configs, &value).is_none()
                });
            foreign.then(|| invalid("query.pageToken: not a token this server issued"))
        }),
        _ => None,
    }
}

/// Whether a stream following a task ends at this state.
fn ends(state: a2a::tasks::TaskState, ends_on_interrupt: bool) -> bool {
    ports::terminal(state) || (ends_on_interrupt && ports::interrupted(state))
}

/// One server-sent event: one `data:` line.
fn event(data: &str) -> String {
    format!("data: {data}\n\n")
}

/// The body with `name` set to `text`: an empty body becomes an object of that one member. A
/// body that is no JSON object is left for the generated layer to refuse.
fn with_member(body: &mut Vec<u8>, name: &str, text: &str) {
    let Ok(Value::Object(mut members)) = http::json_body(body, false) else {
        return;
    };
    members.retain(|(key, _)| key != name);
    members.push((name.to_owned(), Value::Text(text.to_owned())));
    *body = text_of(&Value::Object(members)).into_bytes();
}

/// The body without its member `name`; a body that is no JSON object, or has no such member, is
/// left as it is.
fn without_member(body: &mut Vec<u8>, name: &str) {
    let Ok(Value::Object(mut members)) = http::json_body(body, false) else {
        return;
    };
    let before = members.len();
    members.retain(|(key, _)| key != name);
    if members.len() != before {
        *body = text_of(&Value::Object(members)).into_bytes();
    }
}

/// The answer to a request over a bound the server keeps (A2A v1.0.1 section 13.4).
fn exhausted(message: &str) -> http::Response {
    status_error(429, "RESOURCE_EXHAUSTED", None, message)
}

/// The token of an `Authorization` value in the `Bearer` scheme (RFC 6750 section 2.1; the
/// scheme's name is case-insensitive, RFC 9110 section 11.1); `None` in any other scheme.
fn bearer(value: &str) -> Option<&str> {
    let (scheme, token) = value.trim().split_once(' ')?;
    scheme
        .eq_ignore_ascii_case("Bearer")
        .then_some(token.trim_start())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_bearer_token_in_any_case_of_its_scheme() {
        assert_eq!(bearer("Bearer abc"), Some("abc"));
        assert_eq!(bearer("bearer  abc "), Some("abc"));
        assert_eq!(bearer("Basic abc"), None);
        assert_eq!(bearer("Bearer"), None);
    }

    #[test]
    fn keeps_the_status_of_a_refusal_of_the_request_s_form() {
        for (status, code) in [
            (401, "UNAUTHENTICATED"),
            (408, "DEADLINE_EXCEEDED"),
            (411, "INVALID_ARGUMENT"),
            (413, "INVALID_ARGUMENT"),
            (429, "RESOURCE_EXHAUSTED"),
            (431, "INVALID_ARGUMENT"),
        ] {
            let answer = refusal_error(status, "detail");
            assert_eq!(answer.status, status);
            assert!(answer.body.contains(code), "{status}: {}", answer.body);
        }
        assert_eq!(refusal_error(502, "detail").status, 500);
    }
}
