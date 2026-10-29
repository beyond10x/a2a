//! Assertion helpers over a connected endpoint. Each one makes its calls through
//! [`a2a_client::Client`] and answers what it read, or a [`Mismatch`] that names what it expected
//! and what the endpoint answered; none of them panics.

use std::fmt;

use a2a::discovery::AgentCard;
use a2a::server::wire;
use a2a::tasks::{
    CreateTaskPushNotificationConfig, DeleteTaskPushNotificationConfig, GetTask,
    GetTaskPushNotificationConfig, ListTaskPushNotificationConfigs, MessageStreamResponse,
    PushNotificationConfigId, SendStreamingMessage, TaskId, TaskPushNotificationConfig,
    TaskSnapshot, WireTaskState,
};
use a2a_client::{Client, Error, Reason};

use crate::contract::{Answered, Contract, error_of};

/// The protocol binding `a2a-client` speaks (A2A v1.0.1 section 5.3).
const HTTP_JSON: &str = "HTTP+JSON";

/// The most list pages [`expect_push_config_roundtrip`] reads looking for the config it created.
const PAGES: usize = 64;

/// What an endpoint answered where a helper expected something else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mismatch {
    /// What was checked.
    pub check: String,
    /// The value the check expected.
    pub expected: String,
    /// The value the endpoint answered.
    pub observed: String,
}

impl fmt::Display for Mismatch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: expected {}, observed {}",
            self.check, self.expected, self.observed
        )
    }
}

impl std::error::Error for Mismatch {}

fn mismatch(
    check: impl Into<String>,
    expected: impl Into<String>,
    observed: impl Into<String>,
) -> Mismatch {
    Mismatch {
        check: check.into(),
        expected: expected.into(),
        observed: observed.into(),
    }
}

/// The state as the binding spells it, `TASK_STATE_COMPLETED`.
fn state_name(state: WireTaskState) -> String {
    let mut name = String::new();
    wire::encode_a2a_tasks_task_state_2(&state, &mut name);
    name.trim_matches('"').to_owned()
}

fn state_names(states: &[WireTaskState]) -> String {
    states
        .iter()
        .map(|state| state_name(*state))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The public agent card (A2A v1.0.1 section 8.2), named `name` and offering an HTTP+JSON
/// interface.
///
/// # Errors
///
/// A [`Mismatch`] naming the card it expected and the error the fetch answered, the name the
/// card carries, or the bindings it offers.
pub fn expect_card(client: &Client, name: &str) -> Result<AgentCard, Mismatch> {
    let card = client.agent_card().map_err(|error| {
        mismatch(
            "reading /.well-known/agent-card.json",
            format!("the public agent card of `{name}`"),
            error.to_string(),
        )
    })?;
    if card.name != name {
        return Err(mismatch("the agent card's name", name, card.name));
    }
    if !card
        .supported_interfaces
        .iter()
        .any(|interface| interface.protocol_binding == HTTP_JSON)
    {
        let offered: Vec<&str> = card
            .supported_interfaces
            .iter()
            .map(|interface| interface.protocol_binding.as_str())
            .collect();
        return Err(mismatch(
            format!("the supported interfaces of the agent card of `{name}`"),
            format!("an interface of protocol binding {HTTP_JSON}"),
            if offered.is_empty() {
                "no interface".to_owned()
            } else {
                offered.join(", ")
            },
        ));
    }
    Ok(card)
}

/// The task `request` reads, in `state`.
///
/// # Errors
///
/// A [`Mismatch`] naming the state it expected and the state the task is in, or the error
/// `GetTask` answered with its reason.
pub fn expect_task_state(
    client: &Client,
    request: &GetTask,
    state: WireTaskState,
) -> Result<TaskSnapshot, Mismatch> {
    let check = format!("the state of task `{}`", request.id.0);
    let task = client
        .get_task(request)
        .map_err(|error| mismatch(&check, state_name(state), error.to_string()))?;
    if task.id != request.id {
        return Err(mismatch(
            format!("the task GetTask answered for `{}`", request.id.0),
            format!("task `{}`", request.id.0),
            format!("task `{}`", task.id.0),
        ));
    }
    if task.status.state != state {
        return Err(mismatch(
            check,
            state_name(state),
            state_name(task.status.state),
        ));
    }
    Ok(task)
}

/// That `call` fails with the A2A error of `reason` (A2A v1.0.1 section 11.6), at the HTTP status
/// and `google.rpc.Status` status section 5.4 maps it to.
///
/// # Errors
///
/// A [`Mismatch`] naming the error it expected and what the call answered: the type of the value
/// it returned, or the error with its status and reason.
pub fn expect_error_reason<T>(
    client: &Client,
    reason: &Reason,
    call: impl FnOnce(&Client) -> Result<T, Error>,
) -> Result<(), Mismatch> {
    let check = "the error the call answers";
    match call(client) {
        Err(error) if declares(&error, reason) => Ok(()),
        Err(error) => Err(mismatch(check, the_error(reason), error.to_string())),
        Ok(_) => Err(mismatch(
            check,
            the_error(reason),
            format!("a {} and no error", std::any::type_name::<T>()),
        )),
    }
}

/// Whether `error` is the A2A error of `reason`, at the status A2A v1.0.1 section 5.4 maps it to.
fn declares(error: &Error, reason: &Reason) -> bool {
    let Error::Status {
        http,
        status,
        reason: Some(answered),
        ..
    } = error
    else {
        return false;
    };
    answered == reason
        && mapped(reason).is_none_or(|mapped| mapped.http == *http && mapped.status == *status)
}

/// The HTTP status and `google.rpc.Status` status the error of `reason` is answered with, where
/// the contract declares it.
fn mapped(reason: &Reason) -> Option<&'static Answered> {
    error_of(reason).and_then(|short| Contract::get().answered_as(short))
}

/// The error of `reason` as an expectation names it.
fn the_error(reason: &Reason) -> String {
    match mapped(reason) {
        Some(mapped) => format!("an error of reason {} on {mapped}", reason.as_str()),
        None => format!("an error of reason {}", reason.as_str()),
    }
}

/// One stream response, for a message about it.
fn described(event: &MessageStreamResponse) -> String {
    if let Some(task) = &event.task {
        format!("task `{}`", task.id.0)
    } else if let Some(update) = &event.status_update {
        format!("a status update of task `{}`", update.task_id.0)
    } else if let Some(update) = &event.artifact_update {
        format!("an artifact update of task `{}`", update.task_id.0)
    } else if event.message.is_some() {
        "a message".to_owned()
    } else {
        "an empty stream response".to_owned()
    }
}

/// The task a stream response is about, where it names one.
fn about(event: &MessageStreamResponse) -> Option<&TaskId> {
    event
        .task
        .as_ref()
        .map(|task| &task.id)
        .or_else(|| event.status_update.as_ref().map(|update| &update.task_id))
        .or_else(|| event.artifact_update.as_ref().map(|update| &update.task_id))
}

/// The stream responses of `request`: the task first (A2A v1.0.1 section 3.1.2), then updates of
/// that task only, its states moving through `states` in order.
///
/// # Errors
///
/// A [`Mismatch`] naming the states it expected and the ones the stream carried, an event that is
/// not the task first or that is about another task, or the error the stream answered with its
/// reason.
pub fn expect_stream_states(
    client: &Client,
    request: &SendStreamingMessage,
    states: &[WireTaskState],
) -> Result<Vec<MessageStreamResponse>, Mismatch> {
    let check = format!(
        "the states streamed for message `{}`",
        request.message.message_id
    );
    let expected = state_names(states);
    let events = client
        .send_streaming_message(request)
        .map_err(|error| mismatch(&check, &expected, error.to_string()))?;
    let mut read: Vec<MessageStreamResponse> = Vec::new();
    let mut streamed = Vec::new();
    for event in events {
        let event = event.map_err(|error| {
            let observed = if streamed.is_empty() {
                error.to_string()
            } else {
                format!("{}, then {error}", state_names(&streamed))
            };
            mismatch(&check, &expected, observed)
        })?;
        match read.first().and_then(|first| first.task.as_ref()) {
            None if event.task.is_none() => {
                return Err(mismatch(
                    format!(
                        "the first event streamed for message `{}`",
                        request.message.message_id
                    ),
                    "the task the message created",
                    described(&event),
                ));
            }
            Some(task) if about(&event) != Some(&task.id) => {
                return Err(mismatch(
                    format!("the events streamed for task `{}`", task.id.0),
                    format!("events of task `{}`", task.id.0),
                    described(&event),
                ));
            }
            _ => {}
        }
        if let Some(task) = &event.task {
            streamed.push(task.status.state);
        }
        if let Some(update) = &event.status_update {
            streamed.push(update.status.state);
        }
        read.push(event);
    }
    if streamed != states {
        let observed = if streamed.is_empty() {
            "no state".to_owned()
        } else {
            state_names(&streamed)
        };
        return Err(mismatch(check, expected, observed));
    }
    Ok(read)
}

/// That the push notification config `request` creates is served back as it was created, read
/// by its id, listed for its task, and deleted (A2A v1.0.1 sections 9.6 to 9.9), after which
/// reading it answers `TASK_NOT_FOUND`. Answers the config as it was created.
///
/// A config that was created is deleted whatever a step before the delete found, so the helper
/// leaves no config behind; the first difference is the one reported.
///
/// # Errors
///
/// A [`Mismatch`] naming the step, the config's url, and the value or error the endpoint
/// answered at that step. Neither a token nor credentials are shown.
pub fn expect_push_config_roundtrip(
    client: &Client,
    request: &CreateTaskPushNotificationConfig,
) -> Result<TaskPushNotificationConfig, Mismatch> {
    let url = &request.url;
    let task = &request.task_id;
    let check = |step: &str| format!("{step} the push notification config of task `{}`", task.0);
    let created = client
        .create_task_push_notification_config(request)
        .map_err(|error| {
            mismatch(
                check("creating"),
                format!("a config for {url}"),
                error.to_string(),
            )
        })?;
    let mut first = served(&created, request, &check("creating")).err();
    let Some(id) = created.id.clone() else {
        return Err(first.unwrap_or_else(|| {
            mismatch(
                check("creating"),
                format!("the config for {url} with an id"),
                "a config with no id",
            )
        }));
    };
    let read = |client: &Client| {
        client.get_task_push_notification_config(&GetTaskPushNotificationConfig {
            tenant: request.tenant.clone(),
            task_id: task.clone(),
            id: id.clone(),
        })
    };
    let reading = check(&format!("reading `{}` of", id.0));
    let got = match read(client) {
        Ok(got) => served(&got, request, &reading),
        Err(error) => Err(mismatch(
            &reading,
            format!("the config for {url}"),
            error.to_string(),
        )),
    };
    first = first.or(got.err());
    first = first.or(listed(client, request, &id, &check("listing")).err());
    if let Err(error) =
        client.delete_task_push_notification_config(&DeleteTaskPushNotificationConfig {
            tenant: request.tenant.clone(),
            task_id: task.clone(),
            id: id.clone(),
        })
    {
        return Err(first.unwrap_or_else(|| {
            mismatch(
                check(&format!("deleting `{}` of", id.0)),
                format!("the config for {url} deleted"),
                error.to_string(),
            )
        }));
    }
    if let Some(first) = first {
        return Err(first);
    }
    let gone = check(&format!("reading the deleted `{}` of", id.0));
    let expected = format!(
        "{} for the deleted config for {url}",
        the_error(&Reason::TaskNotFound)
    );
    match read(client) {
        Err(error) if declares(&error, &Reason::TaskNotFound) => Ok(created),
        Err(error) => Err(mismatch(gone, expected, error.to_string())),
        Ok(still) => Err(mismatch(
            gone,
            expected,
            format!("the config for {} still served", still.url),
        )),
    }
}

/// That `config` is the one `request` created: its url, its token, its authentication, its task
/// and the id asked for. Neither a token nor credentials are shown.
fn served(
    config: &TaskPushNotificationConfig,
    request: &CreateTaskPushNotificationConfig,
    check: &str,
) -> Result<(), Mismatch> {
    let url = &request.url;
    if config.url != *url {
        return Err(mismatch(check, url, &config.url));
    }
    if config.token != request.token {
        let observed = match (&config.token, &request.token) {
            (None, _) => "no token",
            (Some(_), None) => "a token where none was sent",
            (Some(_), Some(_)) => "another token",
        };
        return Err(mismatch(
            check,
            format!("the config for {url} with the token it was created with"),
            observed,
        ));
    }
    if let Some(sent) = &request.authentication
        && config.authentication.as_ref() != Some(sent)
    {
        let observed = match &config.authentication {
            None => "no authentication".to_owned(),
            Some(held) if held.scheme != sent.scheme => {
                format!("authentication of scheme `{}`", held.scheme)
            }
            Some(_) => format!("scheme `{}` with other credentials", sent.scheme),
        };
        return Err(mismatch(
            check,
            format!(
                "the config for {url} with the `{}` authentication it was created with",
                sent.scheme
            ),
            observed,
        ));
    }
    if let Some(task) = &config.task_id
        && *task != request.task_id
    {
        return Err(mismatch(
            check,
            format!("the config for {url} on task `{}`", request.task_id.0),
            format!("task `{}`", task.0),
        ));
    }
    if let Some(asked) = &request.id
        && config.id.as_ref() != Some(asked)
    {
        return Err(mismatch(
            check,
            format!("the config for {url} with id `{}`", asked.0),
            config
                .id
                .as_ref()
                .map_or_else(|| "no id".to_owned(), |id| format!("id `{}`", id.0)),
        ));
    }
    Ok(())
}

/// That the configs listed for the task of `request` hold the one of `id`, as created.
fn listed(
    client: &Client,
    request: &CreateTaskPushNotificationConfig,
    id: &PushNotificationConfigId,
    check: &str,
) -> Result<(), Mismatch> {
    let url = &request.url;
    let expected = format!("the config `{}` for {url} among those listed", id.0);
    let mut token = None;
    let mut seen = Vec::new();
    for _ in 0..PAGES {
        let page = client
            .list_task_push_notification_configs(&ListTaskPushNotificationConfigs {
                tenant: request.tenant.clone(),
                task_id: request.task_id.clone(),
                page_size: None,
                page_token: token.take(),
            })
            .map_err(|error| mismatch(check, &expected, error.to_string()))?;
        for config in page.configs.unwrap_or_default() {
            if config.id.as_ref() == Some(id) {
                return served(&config, request, check);
            }
            seen.push(config.id.map_or_else(
                || "a config with no id".to_owned(),
                |id| format!("`{}`", id.0),
            ));
        }
        match page.next_page_token {
            Some(next) if !next.is_empty() => token = Some(next),
            _ => break,
        }
    }
    let observed = if seen.is_empty() {
        "no config listed".to_owned()
    } else {
        format!("only {}", seen.join(", "))
    };
    Err(mismatch(check, expected, observed))
}
