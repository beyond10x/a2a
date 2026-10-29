//! A typed client of A2A's HTTP+JSON binding (A2A v1.0.1 section 11).
//!
//! One method per operation, taking and answering the types of the generated `a2a` crate; this
//! crate defines no model type of its own but the `ListTasks` request and answer, which the
//! generated crate lacks. A request's `tenant` selects the `/{tenant}/` route; an empty, `.` or
//! `..` tenant or id is refused as [`Error::InvalidRequest`] before anything is sent. Every error answer
//! is decoded into [`Error::Status`]; an answer that is not the binding's is
//! [`Error::Protocol`]. Streams are [`Events`] iterators over server-sent events.
//!
//! Transport is HTTP/1.1 over `std::net::TcpStream`, one connection per call, to `http://` base
//! URLs only; TLS and the JSON-RPC binding are not here.

mod error;
mod events;
mod transport;

use std::time::Duration;

use a2a::discovery::{AgentCard, GetExtendedAgentCard};
use a2a::json::{self, Value};
use a2a::primitives::Timestamp;
use a2a::server::wire;
use a2a::tasks::{
    CancelTask, ContextId, CreateTaskPushNotificationConfig, DeleteTaskPushNotificationConfig,
    GetTask, GetTaskPushNotificationConfig, ListTaskPushNotificationConfigs,
    ListTaskPushNotificationConfigsResponse, MessageStreamResponse, SendMessage,
    SendMessageResponse, SendStreamingMessage, SubscribeToTask, TaskPushNotificationConfig,
    TaskSnapshot, TaskSubscriptionResponse, WireTaskState,
};

pub use error::{Error, Reason};
pub use events::Events;

use transport::{Answer, Call, Origin, segment};

/// The media type of every A2A request and answer but the public card's (section 11.2).
const A2A_JSON: &str = "application/a2a+json";
/// The media type of a stream (section 11.7).
const EVENT_STREAM: &str = "text/event-stream";

/// The `ListTasks` request (A2A v1.0.1 section 3.1.4, proto `ListTasksRequest`).
// ESS gap (ess 0.75.0): `ListTasks` is the view `a2a.tasks.Tasks` in `ess/domains/tasks.yaml`, and
// synthesis emits no request type for a view's parameters, no page token or size, and no type
// for the paged answer, so both are declared here.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListTasksRequest {
    /// Selects the `/{tenant}/` route.
    pub tenant: Option<String>,
    /// Only tasks of this context.
    pub context_id: Option<ContextId>,
    /// Only tasks in this state.
    pub status: Option<WireTaskState>,
    /// At most this many tasks; the agent's default is 50.
    pub page_size: Option<i64>,
    /// The `next_page_token` of the page before.
    pub page_token: Option<String>,
    /// At most this many history messages per task.
    pub history_length: Option<i64>,
    /// Only tasks whose status changed at or after this instant.
    pub status_timestamp_after: Option<Timestamp>,
    /// Whether the tasks carry their artifacts.
    pub include_artifacts: Option<bool>,
}

/// The `ListTasks` answer (proto `ListTasksResponse`). A member the agent omits reads as its
/// proto3 default.
// ESS gap (ess 0.75.0): see `ListTasksRequest`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListTasksResponse {
    /// The page's tasks.
    pub tasks: Vec<TaskSnapshot>,
    /// The token of the next page; empty where this is the last.
    pub next_page_token: String,
    /// The page size the agent applied.
    pub page_size: i64,
    /// How many tasks match, on every page.
    pub total_size: i64,
}

/// A client of one A2A agent's HTTP+JSON interface.
///
/// Each call opens one connection, sends `A2A-Version` (default `1.0`), `A2A-Extensions` where
/// any are set and `Authorization: Bearer` where a token is set, and closes the connection when
/// the answer is read. The token goes with every operation under the base path, never with the
/// public card, and is never shown by `Debug`.
#[derive(Clone)]
pub struct Client {
    origin: Origin,
    version: String,
    extensions: String,
    bearer: Option<String>,
    timeout: Option<Duration>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Client")
            .field("origin", &self.origin)
            .field("version", &self.version)
            .field("extensions", &self.extensions)
            .field("bearer", &self.bearer.as_ref().map(|_| ".."))
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl Client {
    /// A client of the agent at `base_url`, `http://host[:port][/path]`. Operations go under the
    /// path; the public card is read at the host's `/.well-known/agent-card.json` (section 8.2).
    ///
    /// # Errors
    ///
    /// [`Error::UnsupportedScheme`] for any scheme but `http`, `https` included;
    /// [`Error::InvalidRequest`] where `base_url` is not such a URL. Neither error's text quotes
    /// the URL, so user information or control bytes in it go nowhere.
    pub fn new(base_url: &str) -> Result<Self, Error> {
        Ok(Self {
            origin: transport::origin(base_url)?,
            version: "1.0".to_owned(),
            extensions: String::new(),
            bearer: None,
            timeout: Some(Duration::from_secs(30)),
        })
    }

    /// Sends `A2A-Version: version` (section 3.6.2) instead of `1.0`.
    #[must_use]
    pub fn with_version(mut self, version: &str) -> Self {
        version.clone_into(&mut self.version);
        self
    }

    /// Declares these extensions in `A2A-Extensions` (section 3.6.1).
    #[must_use]
    pub fn with_extensions(mut self, uris: &[&str]) -> Self {
        self.extensions = uris.join(", ");
        self
    }

    /// Sends `Authorization: Bearer token` with every operation under the base path; the public
    /// card is read without it. A token that is not an RFC 6750 `b64token` makes every call fail
    /// with [`Error::InvalidRequest`] before anything is sent, and the error does not quote it.
    #[must_use]
    pub fn with_bearer(mut self, token: &str) -> Self {
        self.bearer = Some(token.to_owned());
        self
    }

    /// Bounds a call by `timeout`; `None` waits without end. A one-shot call ends within it as a
    /// whole, connecting included (name resolution aside); a stream's call is bounded as a whole up to its head, and then each
    /// wait between two events. The default is 30 seconds.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// The public agent card: `GET /.well-known/agent-card.json` (section 8.2).
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub fn agent_card(&self) -> Result<AgentCard, Error> {
        let card = self.json(
            "GET",
            "/.well-known/agent-card.json".to_owned(),
            None,
            "application/json",
            false,
        )?;
        decoded(wire::decode_a2a_discovery_agent_card(&card, "card"))
    }

    /// `GetExtendedAgentCard`: `GET [/{tenant}]/extendedAgentCard` (section 3.1.11).
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub fn extended_agent_card(&self, request: &GetExtendedAgentCard) -> Result<AgentCard, Error> {
        let path = self.path(request.tenant.as_deref(), "/extendedAgentCard")?;
        let card = self.json("GET", path, None, A2A_JSON, true)?;
        decoded(wire::decode_a2a_discovery_agent_card(&card, "card"))
    }

    /// `SendMessage`: `POST [/{tenant}]/message:send` (section 3.1.1). Answers the task, or the
    /// agent's direct message as `reply`.
    ///
    /// # Errors
    ///
    /// See [`Error`]; an answer holding neither a task nor a message is [`Error::Protocol`].
    pub fn send_message(&self, request: &SendMessage) -> Result<SendMessageResponse, Error> {
        let mut body = String::new();
        wire::encode_command_a2a_tasks_send_message(
            &SendMessage {
                tenant: None,
                ..request.clone()
            },
            &mut body,
        );
        let path = self.path(request.tenant.as_deref(), "/message:send")?;
        let answer = self.json("POST", path, Some(body), A2A_JSON, true)?;
        let task = match answer.member("task") {
            None | Some(Value::Null) => None,
            Some(task) => Some(decoded(wire::decode_a2a_tasks_task_snapshot(task, "task"))?),
        };
        let reply = match answer.member("message") {
            None | Some(Value::Null) => None,
            Some(message) => Some(decoded(wire::decode_a2a_tasks_message(message, "message"))?),
        };
        if task.is_some() == reply.is_some() {
            return Err(Error::Protocol(
                "a SendMessage answer holds exactly one of `task` and `message`".to_owned(),
            ));
        }
        Ok(SendMessageResponse { task, reply })
    }

    /// `SendStreamingMessage`: `POST [/{tenant}]/message:stream` (section 3.1.2). The events
    /// are read as they arrive.
    ///
    /// # Errors
    ///
    /// See [`Error`]: an error answer before the stream opens is [`Error::Status`].
    pub fn send_streaming_message(
        &self,
        request: &SendStreamingMessage,
    ) -> Result<Events<MessageStreamResponse>, Error> {
        let mut body = String::new();
        wire::encode_command_a2a_tasks_send_streaming_message(
            &SendStreamingMessage {
                tenant: None,
                ..request.clone()
            },
            &mut body,
        );
        let path = self.path(request.tenant.as_deref(), "/message:stream")?;
        self.stream(
            "POST",
            path,
            Some(body),
            wire::decode_event_a2a_tasks_message_stream_response,
        )
    }

    /// `GetTask`: `GET [/{tenant}]/tasks/{id}` (section 3.1.3).
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub fn get_task(&self, request: &GetTask) -> Result<TaskSnapshot, Error> {
        let mut path = self.path(
            request.tenant.as_deref(),
            &format!("/tasks/{}", segment(&request.id.0, "task id")?),
        )?;
        if let Some(length) = request.history_length {
            path.push_str(&format!("?historyLength={length}"));
        }
        let task = self.json("GET", path, None, A2A_JSON, true)?;
        decoded(wire::decode_a2a_tasks_task_snapshot(&task, "task"))
    }

    /// `ListTasks`: `GET [/{tenant}]/tasks` (section 3.1.4).
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub fn list_tasks(&self, request: &ListTasksRequest) -> Result<ListTasksResponse, Error> {
        let mut query = Vec::new();
        if let Some(context) = &request.context_id {
            query.push(("contextId", context.0.clone()));
        }
        if let Some(state) = request.status {
            let mut name = String::new();
            wire::encode_a2a_tasks_task_state_2(&state, &mut name);
            query.push(("status", name.trim_matches('"').to_owned()));
        }
        if let Some(size) = request.page_size {
            query.push(("pageSize", size.to_string()));
        }
        if let Some(token) = &request.page_token {
            query.push(("pageToken", token.clone()));
        }
        if let Some(length) = request.history_length {
            query.push(("historyLength", length.to_string()));
        }
        if let Some(after) = &request.status_timestamp_after {
            query.push(("statusTimestampAfter", after.0.clone()));
        }
        if let Some(include) = request.include_artifacts {
            query.push(("includeArtifacts", include.to_string()));
        }
        let mut path = self.path(request.tenant.as_deref(), "/tasks")?;
        for (at, (name, value)) in query.iter().enumerate() {
            path.push(if at == 0 { '?' } else { '&' });
            path.push_str(name);
            path.push('=');
            path.push_str(&a2a::server::http::form_encoded(value));
        }
        let answer = self.json("GET", path, None, A2A_JSON, true)?;
        decoded(list_tasks_response(&answer))
    }

    /// `CancelTask`: `POST [/{tenant}]/tasks/{id}:cancel` (section 3.1.5).
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub fn cancel_task(&self, request: &CancelTask) -> Result<TaskSnapshot, Error> {
        let mut body = String::from("{");
        if let Some(metadata) = &request.metadata {
            json::member(&mut body, "metadata");
            json::push_value(&mut body, metadata);
        }
        body.push('}');
        let path = self.path(
            request.tenant.as_deref(),
            &format!("/tasks/{}:cancel", segment(&request.id.0, "task id")?),
        )?;
        let task = self.json("POST", path, Some(body), A2A_JSON, true)?;
        decoded(wire::decode_a2a_tasks_task_snapshot(&task, "task"))
    }

    /// `SubscribeToTask`: `GET [/{tenant}]/tasks/{id}:subscribe` (section 3.1.6).
    ///
    /// # Errors
    ///
    /// See [`Error`]: an error answer before the stream opens is [`Error::Status`].
    pub fn subscribe_to_task(
        &self,
        request: &SubscribeToTask,
    ) -> Result<Events<TaskSubscriptionResponse>, Error> {
        let path = self.path(
            request.tenant.as_deref(),
            &format!("/tasks/{}:subscribe", segment(&request.id.0, "task id")?),
        )?;
        self.stream(
            "GET",
            path,
            None,
            wire::decode_event_a2a_tasks_task_subscription_response,
        )
    }

    /// `CreateTaskPushNotificationConfig`: `POST [/{tenant}]/tasks/{taskId}/pushNotificationConfigs`
    /// (section 3.1.7).
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub fn create_task_push_notification_config(
        &self,
        request: &CreateTaskPushNotificationConfig,
    ) -> Result<TaskPushNotificationConfig, Error> {
        let mut body = String::from("{");
        if let Some(id) = &request.id {
            json::member(&mut body, "id");
            json::push_text(&mut body, &id.0);
        }
        json::member(&mut body, "url");
        json::push_text(&mut body, &request.url);
        if let Some(token) = &request.token {
            json::member(&mut body, "token");
            json::push_text(&mut body, token);
        }
        if let Some(authentication) = &request.authentication {
            json::member(&mut body, "authentication");
            wire::encode_a2a_tasks_authentication_info(authentication, &mut body);
        }
        body.push('}');
        let path = self.path(
            request.tenant.as_deref(),
            &format!(
                "/tasks/{}/pushNotificationConfigs",
                segment(&request.task_id.0, "task id")?
            ),
        )?;
        let config = self.json("POST", path, Some(body), A2A_JSON, true)?;
        decoded(wire::decode_a2a_tasks_task_push_notification_config(
            &config, "config",
        ))
    }

    /// `GetTaskPushNotificationConfig`:
    /// `GET [/{tenant}]/tasks/{taskId}/pushNotificationConfigs/{id}` (section 3.1.8).
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub fn get_task_push_notification_config(
        &self,
        request: &GetTaskPushNotificationConfig,
    ) -> Result<TaskPushNotificationConfig, Error> {
        let path = self.path(
            request.tenant.as_deref(),
            &format!(
                "/tasks/{}/pushNotificationConfigs/{}",
                segment(&request.task_id.0, "task id")?,
                segment(&request.id.0, "config id")?
            ),
        )?;
        let config = self.json("GET", path, None, A2A_JSON, true)?;
        decoded(wire::decode_a2a_tasks_task_push_notification_config(
            &config, "config",
        ))
    }

    /// `ListTaskPushNotificationConfigs`:
    /// `GET [/{tenant}]/tasks/{taskId}/pushNotificationConfigs` (section 3.1.9).
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub fn list_task_push_notification_configs(
        &self,
        request: &ListTaskPushNotificationConfigs,
    ) -> Result<ListTaskPushNotificationConfigsResponse, Error> {
        let mut path = self.path(
            request.tenant.as_deref(),
            &format!(
                "/tasks/{}/pushNotificationConfigs",
                segment(&request.task_id.0, "task id")?
            ),
        )?;
        let mut separator = '?';
        if let Some(size) = request.page_size {
            path.push_str(&format!("{separator}pageSize={size}"));
            separator = '&';
        }
        if let Some(token) = &request.page_token {
            path.push_str(&format!(
                "{separator}pageToken={}",
                a2a::server::http::form_encoded(token)
            ));
        }
        let answer = self.json("GET", path, None, A2A_JSON, true)?;
        decoded(list_configs_response(&answer))
    }

    /// `DeleteTaskPushNotificationConfig`:
    /// `DELETE [/{tenant}]/tasks/{taskId}/pushNotificationConfigs/{id}` (section 3.1.10).
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub fn delete_task_push_notification_config(
        &self,
        request: &DeleteTaskPushNotificationConfig,
    ) -> Result<(), Error> {
        let path = self.path(
            request.tenant.as_deref(),
            &format!(
                "/tasks/{}/pushNotificationConfigs/{}",
                segment(&request.task_id.0, "task id")?,
                segment(&request.id.0, "config id")?
            ),
        )?;
        self.json("DELETE", path, None, A2A_JSON, true).map(|_| ())
    }

    /// The target of `route` under the base URL's path and, where given, the tenant's segment.
    fn path(&self, tenant: Option<&str>, route: &str) -> Result<String, Error> {
        match tenant {
            Some(tenant) => Ok(format!(
                "{}/{}{route}",
                self.origin.prefix,
                segment(tenant, "tenant")?
            )),
            None => Ok(format!("{}{route}", self.origin.prefix)),
        }
    }

    /// One exchange; `authorized` adds the bearer token where one is set.
    fn call(
        &self,
        method: &'static str,
        target: String,
        body: Option<String>,
        accept: &'static str,
        authorized: bool,
    ) -> Result<Answer, Error> {
        let mut headers = vec![("Accept", accept), ("A2A-Version", self.version.as_str())];
        if !self.extensions.is_empty() {
            headers.push(("A2A-Extensions", self.extensions.as_str()));
        }
        if self.bearer.as_deref().is_some_and(|token| !b64token(token)) {
            return Err(Error::InvalidRequest(
                "the bearer token is not an RFC 6750 b64token".to_owned(),
            ));
        }
        let authorization = self
            .bearer
            .as_ref()
            .filter(|_| authorized)
            .map(|token| format!("Bearer {token}"));
        if let Some(authorization) = &authorization {
            headers.push(("Authorization", authorization.as_str()));
        }
        transport::exchange(
            &self.origin,
            self.timeout,
            &Call {
                method,
                target,
                headers,
                body,
            },
        )
    }

    /// The JSON answer of a call; an error answer as [`Error::Status`].
    fn json(
        &self,
        method: &'static str,
        target: String,
        body: Option<String>,
        accept: &'static str,
        authorized: bool,
    ) -> Result<Value, Error> {
        let answer = self.call(method, target, body, accept, authorized)?;
        let status = answer.status;
        let text = answer.text()?;
        if !(200..300).contains(&status) {
            return Err(error::status_error(status, &text));
        }
        json::parse(&text).map_err(|error| {
            Error::Protocol(format!(
                "the {status} answer's body is not JSON ({error}): {}",
                error::excerpt(&text)
            ))
        })
    }

    /// The events of a stream; an error answer before it opens as [`Error::Status`].
    fn stream<T>(
        &self,
        method: &'static str,
        target: String,
        body: Option<String>,
        decode: fn(&Value, &str) -> Result<T, json::DecodeError>,
    ) -> Result<Events<T>, Error> {
        let mut answer = self.call(method, target, body, EVENT_STREAM, true)?;
        let status = answer.status;
        if !(200..300).contains(&status) {
            let text = answer.text()?;
            return Err(error::status_error(status, &text));
        }
        let media = answer.header("Content-Type").unwrap_or_default();
        let media = media.split(';').next().unwrap_or_default().trim();
        if !media.eq_ignore_ascii_case(EVENT_STREAM) {
            return Err(Error::Protocol(format!(
                "a stream was answered as `{media}`, not {EVENT_STREAM}"
            )));
        }
        answer.body.unbound();
        Ok(Events::new(answer.body, decode))
    }
}

/// Whether `token` is an RFC 6750 `b64token`: `1*( ALPHA / DIGIT / "-" / "." / "_" / "~" / "+"
/// / "/" ) *"="`.
fn b64token(token: &str) -> bool {
    let body = token.trim_end_matches('=');
    !body.is_empty()
        && body
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-._~+/".contains(&byte))
}

/// A decoded value, or the protocol error of a body that is not the operation's type.
fn decoded<T>(result: Result<T, json::DecodeError>) -> Result<T, Error> {
    result.map_err(|error| {
        Error::Protocol(format!("the answer is not the operation's type: {error}"))
    })
}

/// An object's member, `None` where absent or `null`.
fn present<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    value.member(name).filter(|member| **member != Value::Null)
}

fn object(value: &Value, at: &str) -> Result<(), json::DecodeError> {
    match value {
        Value::Object(_) => Ok(()),
        other => Err(json::DecodeError::of(at, "an object", other)),
    }
}

fn list_tasks_response(value: &Value) -> Result<ListTasksResponse, json::DecodeError> {
    object(value, "answer")?;
    let mut answer = ListTasksResponse::default();
    if let Some(tasks) = present(value, "tasks") {
        for (at, task) in json::items_at(tasks, "answer.tasks", "an array")?
            .iter()
            .enumerate()
        {
            answer.tasks.push(wire::decode_a2a_tasks_task_snapshot(
                task,
                &format!("answer.tasks.{at}"),
            )?);
        }
    }
    if let Some(token) = present(value, "nextPageToken") {
        json::text_at(token, "answer.nextPageToken", "a string")?
            .clone_into(&mut answer.next_page_token);
    }
    if let Some(size) = present(value, "pageSize") {
        answer.page_size = json::integer_at(size, "answer.pageSize", "an integer")?;
    }
    if let Some(total) = present(value, "totalSize") {
        answer.total_size = json::integer_at(total, "answer.totalSize", "an integer")?;
    }
    Ok(answer)
}

fn list_configs_response(
    value: &Value,
) -> Result<ListTaskPushNotificationConfigsResponse, json::DecodeError> {
    object(value, "answer")?;
    let configs = match present(value, "configs") {
        None => None,
        Some(configs) => Some(
            json::items_at(configs, "answer.configs", "an array")?
                .iter()
                .enumerate()
                .map(|(at, config)| {
                    wire::decode_a2a_tasks_task_push_notification_config(
                        config,
                        &format!("answer.configs.{at}"),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
    };
    let next_page_token = match present(value, "nextPageToken") {
        None => None,
        Some(token) => Some(json::text_at(token, "answer.nextPageToken", "a string")?.to_owned()),
    };
    Ok(ListTaskPushNotificationConfigsResponse {
        configs,
        next_page_token,
    })
}
