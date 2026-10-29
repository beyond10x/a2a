//! The conformance target over the wire: an A2A endpoint at a base URL, reached through
//! `a2a-client`, carrying out the steps of a synthesized scenario the HTTP+JSON binding can
//! observe.
//!
//! A command runs as the call [`crate::OPERATIONS`] binds it to, with the scenario's input renamed
//! as the generated contract names the wire and decoded as the generated server decodes it; the
//! input's `tenant` becomes the request's `/{tenant}` prefix. The answer reads as the outcome the
//! generated contract (`a2a-agent.openapi.json`) declares for it: a success as the command's
//! accepting outcome, an error by the declared error its `google.rpc.ErrorInfo` reason carries,
//! at the HTTP status and `google.rpc.Status` status A2A v1.0.1 section 5.4 maps that error to.
//! Of the outcomes that carry the error, those the circumstances rule out are dropped: an outcome
//! decided outside the request whose circumstance the request or the agent card excludes. Every
//! step the wire cannot observe is `TargetError::Unsupported` with the reason, never `passed`.
//!
//! Given the suite ([`WireTarget::with_suite`]), the target knows the outcome each step expects:
//! an answer that is not that outcome is reported as the outcome it is, or as the answer no
//! outcome declares, so the scenario fails on it rather than coming out `unsupported` on the
//! events the outcome would publish.

use a2a::discovery::AgentCard;
use a2a::json::{self, Value};
use a2a::server::wire;
use a2a::{discovery, tasks};
use a2a_client::{Client, Error};
use a2a_conformance::naming::Naming;
use a2a_conformance::{Answer, Awaited, Event, Reader, Target, TargetError};

use crate::binding::carried;
use crate::contract::{Contract, Outcome, error_of};
use crate::expected::Expectations;

/// The one view A2A has an operation for: `GET /tasks`, ListTasks (A2A v1.0.1 section 9.4).
const TASKS: &str = "a2a.tasks.Tasks";

/// The command whose answer A2A v1.0.1 section 3.1.11 gives to authenticated clients only.
const EXTENDED_CARD: &str = "a2a.discovery.GetExtendedAgentCard";

/// An A2A endpoint as a [`Target`]: the client it calls as, and what the scenario open on it has
/// done.
pub struct WireTarget {
    client: Client,
    /// Whether the client sends a bearer token the run was given.
    bearer: bool,
    /// The endpoint's public agent card, read the first time a circumstance it declares decides
    /// between outcomes.
    card: Option<Result<AgentCard, String>>,
    /// What the scenarios of the suite expect, where the target was given the suite.
    expectations: Expectations,
    /// The outcome the open scenario expects of each command it runs, in order.
    expected: Vec<Option<String>>,
    /// How many commands the open scenario has run.
    executed: usize,
    /// Why the events of the last command are not observable, where it took an outcome that
    /// publishes.
    unobserved: Option<String>,
    /// Whether a command of the open scenario took an accepting outcome, so a record it named
    /// may exist on the endpoint.
    recorded: bool,
}

impl WireTarget {
    /// The target over `client`, the endpoint's base URL and the headers every call sends.
    ///
    /// The client is taken to send `A2A-Version: 1.0`, its default: an endpoint refusing that
    /// version with `VERSION_NOT_SUPPORTED` is not an A2A v1.0.1 endpoint for this run.
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            bearer: false,
            card: None,
            expectations: Expectations::default(),
            expected: Vec::new(),
            executed: 0,
            unobserved: None,
            recorded: false,
        }
    }

    /// The target sending `Authorization: Bearer <token>` with every operation, as `a2a-client`
    /// sends it. Without a token, an extended card the endpoint serves only to authenticated
    /// clients is `unsupported`.
    #[must_use]
    pub fn with_bearer(self, token: &str) -> Self {
        Self {
            client: self.client.with_bearer(token),
            bearer: true,
            ..self
        }
    }

    /// The target knowing the outcome each scenario of the `ess-conformance/52` suite `suite`
    /// expects of each command it runs.
    ///
    /// # Errors
    ///
    /// A message where `suite` is not JSON or holds no `scenarios` object.
    pub fn with_suite(self, suite: &str) -> Result<Self, String> {
        Ok(Self {
            expectations: Expectations::read(suite)?,
            ..self
        })
    }

    /// The public agent card, read once; `None` where the endpoint serves none.
    fn card(&mut self) -> Option<&AgentCard> {
        let client = &self.client;
        self.card
            .get_or_insert_with(|| client.agent_card().map_err(|error| error.to_string()))
            .as_ref()
            .ok()
    }

    /// Why the circumstances of this request rule `outcome` out, where they do: an outcome
    /// decided outside the request whose circumstance the request itself, the agent card or the
    /// scenario so far excludes.
    fn excluded(&mut self, command: &str, outcome: &Outcome, input: &Value) -> Option<String> {
        if !outcome.external {
            return None;
        }
        let message = input.member("message");
        let names = |field: &str| {
            message
                .and_then(|message| message.member(field))
                .is_some_and(|value| *value != Value::Null)
        };
        let why = match outcome.name.as_str() {
            "version-not-supported" => {
                "the request names A2A-Version 1.0, the version A2A v1.0.1 defines"
            }
            "not-cancelable-now" if !self.recorded => "the scenario holds no task",
            "task-not-found" | "task-terminal" | "context-mismatch" | "history-limit"
                if message.is_some() && !names("taskId") =>
            {
                "the message names no task"
            }
            "task-limit" if names("taskId") => "the message names a task",
            "context-not-accepted" if message.is_some() && !names("contextId") => {
                "the message names no context"
            }
            "extension-required" => {
                let required = self
                    .card()?
                    .capabilities
                    .extensions
                    .iter()
                    .flatten()
                    .any(|extension| extension.required == Some(true));
                if required {
                    return None;
                }
                "the agent card marks no extension required"
            }
            "not-supported" if command == EXTENDED_CARD => {
                if self.card()?.capabilities.extended_agent_card != Some(true) {
                    return None;
                }
                "the agent card declares capabilities.extendedAgentCard"
            }
            "not-configured" => {
                if self.card()?.capabilities.extended_agent_card == Some(true) {
                    return None;
                }
                "the agent card does not declare capabilities.extendedAgentCard"
            }
            "push-not-supported" => {
                if self.card()?.capabilities.push_notifications != Some(true) {
                    return None;
                }
                "the agent card declares capabilities.pushNotifications"
            }
            "streaming-not-supported" => {
                if self.card()?.capabilities.streaming != Some(true) {
                    return None;
                }
                "the agent card declares capabilities.streaming"
            }
            _ => return None,
        };
        Some(why.to_owned())
    }

    /// The outcomes an error answer of `command` may be, and what it answered, for a message.
    fn refusals(
        &self,
        command: &str,
        outcomes: &'static [Outcome],
        error: &Error,
    ) -> Result<(Vec<&'static Outcome>, String), TargetError> {
        let Error::Status {
            http,
            status,
            reason,
            ..
        } = error
        else {
            return Err(match error {
                Error::InvalidRequest(detail) => TargetError::Unsupported(format!(
                    "a2a-client cannot send {command} with this input: {detail}"
                )),
                other => TargetError::Failed(format!("{command}: {other}")),
            });
        };
        if *http == 401 && command == EXTENDED_CARD && !self.bearer {
            return Err(TargetError::Unsupported(format!(
                "{command} answered {error}: the endpoint serves its extended card to \
                 authenticated clients (A2A v1.0.1 section 3.1.11), and the run was given no \
                 --bearer-file"
            )));
        }
        let short = match reason {
            Some(reason) => error_of(reason),
            None if status == "RESOURCE_EXHAUSTED" => Some("ResourceExhausted"),
            None => None,
        };
        let Some(short) = short else {
            return Ok((Vec::new(), format!("{error}, no error A2A v1.0.1 declares")));
        };
        let domain = command
            .rsplit_once('.')
            .map_or(command, |(domain, _)| domain);
        let named = format!("{domain}.{short}");
        if let Some(mapped) = Contract::get().answered_as(&named)
            && (mapped.http != *http || mapped.status != *status)
        {
            return Ok((
                Vec::new(),
                format!("{error}, where A2A v1.0.1 answers {short} with {mapped}"),
            ));
        }
        let carrying = outcomes
            .iter()
            .filter(|outcome| outcome.error.as_deref() == Some(named.as_str()))
            .collect();
        Ok((carrying, error.to_string()))
    }

    /// What `command` answered, read as the outcome its contract declares for that answer.
    fn read(
        &mut self,
        command: &str,
        input: &Value,
        expected: Option<&str>,
        called: Called,
    ) -> Result<Answer, TargetError> {
        let outcomes = Contract::get().outcomes(command);
        let (candidates, response, answered) = match called {
            Ok(Accepted { response, outcome }) => (
                outcomes
                    .iter()
                    .filter(|declared| {
                        declared.error.is_none()
                            && outcome.is_none_or(|named| declared.name == named)
                    })
                    .collect(),
                response,
                "a success answer".to_owned(),
            ),
            Err(error) => {
                let (candidates, answered) = self.refusals(command, outcomes, &error)?;
                (candidates, None, answered)
            }
        };
        let mut ruled_out = Vec::new();
        let mut left: Vec<&'static Outcome> = Vec::new();
        for candidate in candidates {
            match self.excluded(command, candidate, input) {
                Some(why) => ruled_out.push(format!("`{}`, as {why}", candidate.name)),
                None => left.push(candidate),
            }
        }
        let taken = match left.as_slice() {
            [] if ruled_out.is_empty() => Err(format!("no declared outcome: {answered}")),
            [] => Err(format!(
                "no declared outcome: {answered}, whose outcome {} the circumstances rule out",
                ruled_out.join(" and ")
            )),
            [only] => Ok(*only),
            several => {
                let unknown: Vec<&&Outcome> = several
                    .iter()
                    .filter(|outcome| outcome.unknown_identity)
                    .collect();
                let names = several
                    .iter()
                    .map(|outcome| format!("`{}`", outcome.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                match unknown.as_slice() {
                    [only] if !self.recorded => Ok(**only),
                    _ if expected.is_none_or(|expected| {
                        several.iter().any(|outcome| outcome.name == expected)
                    }) || several.iter().any(|outcome| outcome.external) =>
                    {
                        return Err(TargetError::Unsupported(format!(
                            "{command} answered {answered}, which its outcomes {names} all carry; \
                             the wire does not tell them apart"
                        )));
                    }
                    _ => Err(format!("one of {names}: {answered}")),
                }
            }
        };
        let outcome = match taken {
            Ok(outcome) => outcome,
            Err(observed) => {
                self.unobserved = None;
                return match expected {
                    Some(_) => Ok(Answer {
                        outcome: observed,
                        error: None,
                        response: None,
                    }),
                    None => Err(TargetError::Failed(format!(
                        "{command} answered {observed}"
                    ))),
                };
            }
        };
        if outcome.external {
            return Err(TargetError::Unsupported(format!(
                "{command} took `{}`, an outcome the specification decides outside the request; \
                 the wire can neither arrange it nor tell it from the circumstances the scenario \
                 set up",
                outcome.name
            )));
        }
        if outcome.error.is_none() {
            self.recorded = true;
        }
        let differs = expected.is_some_and(|expected| expected != outcome.name);
        self.unobserved = (outcome.publishes && !differs).then(|| {
            format!(
                "{command} took `{}`, which publishes events the A2A HTTP+JSON binding does not \
                 carry",
                outcome.name
            )
        });
        let response = match response {
            Some(text) => {
                let wire = json::parse_written(&text).map_err(|error| {
                    TargetError::Failed(format!(
                        "{command}: the response re-encoded is not JSON: {error}"
                    ))
                })?;
                Some(Naming::contract().response_to_ess(command, &wire))
            }
            None => None,
        };
        Ok(Answer {
            outcome: outcome.name.clone(),
            error: outcome.error.clone(),
            response,
        })
    }

    /// Runs `command` through the client, its `input` wire-named.
    fn call(&self, command: &str, input: &Value) -> Result<Called, TargetError> {
        let client = &self.client;
        Ok(match command {
            "a2a.discovery.GetExtendedAgentCard" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_discovery_get_extended_agent_card(input, "input"),
                )?;
                client.extended_agent_card(&request).map(|card| {
                    Accepted::of(
                        &discovery::GetExtendedAgentCardResponse { card },
                        wire::encode_response_a2a_discovery_get_extended_agent_card,
                    )
                })
            }
            "a2a.tasks.SendMessage" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_tasks_send_message(input, "input"),
                )?;
                client.send_message(&request).map(|answered| {
                    let outcome = sent(&request.message, answered.reply.is_some());
                    Accepted::of(&answered, wire::encode_response_a2a_tasks_send_message)
                        .named(outcome)
                })
            }
            "a2a.tasks.SendStreamingMessage" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_tasks_send_streaming_message(input, "input"),
                )?;
                client
                    .send_streaming_message(&request)
                    .and_then(first)
                    .map(|event| {
                        let outcome = sent(&request.message, event.message.is_some());
                        Accepted::of(
                            &tasks::SendStreamingMessageResponse {
                                task: event.task,
                                reply: event.message,
                            },
                            wire::encode_response_a2a_tasks_send_streaming_message,
                        )
                        .named(outcome)
                    })
            }
            "a2a.tasks.GetTask" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_tasks_get_task(input, "input"),
                )?;
                client.get_task(&request).map(|task| {
                    Accepted::of(
                        &tasks::GetTaskResponse { task },
                        wire::encode_response_a2a_tasks_get_task,
                    )
                })
            }
            "a2a.tasks.CancelTask" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_tasks_cancel_task(input, "input"),
                )?;
                client.cancel_task(&request).map(|task| {
                    Accepted::of(
                        &tasks::CancelTaskResponse { task },
                        wire::encode_response_a2a_tasks_cancel_task,
                    )
                })
            }
            "a2a.tasks.SubscribeToTask" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_tasks_subscribe_to_task(input, "input"),
                )?;
                client
                    .subscribe_to_task(&request)
                    .and_then(first)
                    .and_then(|event| {
                        event.task.ok_or_else(|| {
                            Error::Protocol(
                                "the first event of a SubscribeToTask stream is not the task \
                                 (A2A v1.0.1 section 3.1.6)"
                                    .to_owned(),
                            )
                        })
                    })
                    .map(|task| {
                        Accepted::of(
                            &tasks::SubscribeToTaskResponse { task },
                            wire::encode_response_a2a_tasks_subscribe_to_task,
                        )
                    })
            }
            "a2a.tasks.CreateTaskPushNotificationConfig" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_tasks_create_task_push_notification_config(
                        input, "input",
                    ),
                )?;
                client
                    .create_task_push_notification_config(&request)
                    .map(|config| {
                        Accepted::of(
                            &tasks::CreateTaskPushNotificationConfigResponse { config },
                            wire::encode_response_a2a_tasks_create_task_push_notification_config,
                        )
                        .named(if request.id.is_some() {
                            "created"
                        } else {
                            "assigned"
                        })
                    })
            }
            "a2a.tasks.GetTaskPushNotificationConfig" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_tasks_get_task_push_notification_config(
                        input, "input",
                    ),
                )?;
                client
                    .get_task_push_notification_config(&request)
                    .map(|config| {
                        Accepted::of(
                            &tasks::GetTaskPushNotificationConfigResponse { config },
                            wire::encode_response_a2a_tasks_get_task_push_notification_config,
                        )
                    })
            }
            "a2a.tasks.ListTaskPushNotificationConfigs" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_tasks_list_task_push_notification_configs(
                        input, "input",
                    ),
                )?;
                client
                    .list_task_push_notification_configs(&request)
                    .map(|listed| {
                        Accepted::of(
                            &listed,
                            wire::encode_response_a2a_tasks_list_task_push_notification_configs,
                        )
                    })
            }
            "a2a.tasks.DeleteTaskPushNotificationConfig" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_tasks_delete_task_push_notification_config(
                        input, "input",
                    ),
                )?;
                client
                    .delete_task_push_notification_config(&request)
                    .map(|()| Accepted {
                        response: None,
                        outcome: None,
                    })
            }
            other => {
                return Err(TargetError::Failed(format!(
                    "{other} is bound to an A2A operation a2a-testkit does not call"
                )));
            }
        })
    }
}

/// What the endpoint answered a command: its success, or the error.
type Called = Result<Accepted, Error>;

/// A success answer.
struct Accepted {
    /// The command's result, encoded as the generated server encodes it.
    response: Option<String>,
    /// The accepting outcome the answer is, where the command declares several.
    outcome: Option<&'static str>,
}

impl Accepted {
    fn of<T>(response: &T, encode: fn(&T, &mut String)) -> Self {
        let mut text = String::new();
        encode(response, &mut text);
        Self {
            response: Some(text),
            outcome: None,
        }
    }

    fn named(self, outcome: &'static str) -> Self {
        Self {
            outcome: Some(outcome),
            ..self
        }
    }
}

/// The accepting outcome of a message sent: a direct reply, a task continued, or a task created
/// (`ess/domains/tasks.yaml`, SendMessage and SendStreamingMessage).
fn sent(message: &tasks::Message, replied: bool) -> &'static str {
    if replied {
        "direct-message"
    } else if message.task_id.is_some() {
        "continued"
    } else {
        "task-created"
    }
}

/// The first event of a stream; the stream is closed when it is dropped.
fn first<T>(mut events: a2a_client::Events<T>) -> Result<T, Error> {
    events.next().unwrap_or_else(|| {
        Err(Error::Protocol(
            "the stream ended before its first event".to_owned(),
        ))
    })
}

fn decoded<T>(command: &str, decoded: Result<T, json::DecodeError>) -> Result<T, TargetError> {
    decoded.map_err(|error| {
        TargetError::Failed(format!(
            "the scenario's input of {command} is not the command's wire input: {error}"
        ))
    })
}

/// Why `actor` cannot be the caller of `name` over the wire, where it cannot: the client calls as
/// the actor the contract grants `name` to its callers.
fn not_the_client(name: &str, doing: &str, actor: Option<&str>) -> Option<String> {
    let granted = Contract::get().granted(name);
    let clients = || {
        granted
            .iter()
            .map(|granted| format!("`{granted}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    match actor {
        None => Some(format!(
            "{doing} {name} as no actor: an A2A client always calls as {}, and an \
             unauthenticated call is no operation of the binding",
            clients()
        )),
        Some(actor) if !granted.iter().any(|granted| granted == actor) => Some(format!(
            "{doing} {name} as `{actor}`: the A2A client calls as {}, and the wire has no way to \
             act as another actor",
            clients()
        )),
        Some(_) => None,
    }
}

impl Target for WireTarget {
    fn begin(&mut self, scenario: &str) -> Result<(), TargetError> {
        self.unobserved = None;
        self.recorded = false;
        self.expected = self.expectations.of(scenario);
        self.executed = 0;
        Ok(())
    }

    fn execute(
        &mut self,
        command: &str,
        actor: Option<&str>,
        input: &Value,
    ) -> Result<Answer, TargetError> {
        self.unobserved = None;
        let expected = self.expected.get(self.executed).cloned().flatten();
        self.executed += 1;
        if !carried(command) {
            return Err(TargetError::Unsupported(format!(
                "{command} has no A2A operation: the agent issues it inside the server, so the \
                 wire cannot run it"
            )));
        }
        if let Some(reason) = not_the_client(command, "running", actor) {
            return Err(TargetError::Unsupported(reason));
        }
        let input = Naming::contract().input_to_wire(command, input);
        let called = self.call(command, &input)?;
        self.read(command, &input, expected.as_deref(), called)
    }

    fn query(
        &mut self,
        view: &str,
        reader: Reader<'_>,
        _params: &Value,
    ) -> Result<Vec<Value>, TargetError> {
        if view != TASKS || !carried(view) {
            return Err(TargetError::Unsupported(format!(
                "{view} has no A2A operation: A2A v1.0.1 offers no read of it, so the wire cannot \
                 observe its rows"
            )));
        }
        let actor = match reader {
            Reader::Harness => None,
            Reader::Actor(actor) => Some(actor),
            Reader::Anonymous => {
                return Err(TargetError::Unsupported(
                    not_the_client(view, "reading", None).unwrap_or_default(),
                ));
            }
        };
        if let Some(reason) = actor.and_then(|actor| not_the_client(view, "reading", Some(actor))) {
            return Err(TargetError::Unsupported(reason));
        }
        Err(TargetError::Unsupported(format!(
            "reading {view} (ListTasks, GET /tasks) answers every task the endpoint holds for the \
             tenant, and a live endpoint cannot be reset to the empty state each scenario starts \
             from"
        )))
    }

    fn force(&mut self, command: &str, outcome: &str) -> Result<(), TargetError> {
        Err(TargetError::Unsupported(format!(
            "forcing {command} to `{outcome}`: the specification decides it outside the request, \
             and the wire has no way to arrange those circumstances"
        )))
    }

    fn published(&mut self) -> Result<Vec<Event>, TargetError> {
        match &self.unobserved {
            Some(reason) => Err(TargetError::Unsupported(reason.clone())),
            None => Ok(Vec::new()),
        }
    }

    fn await_outcome(&mut self, command: &str, _instance: &Value) -> Result<Awaited, TargetError> {
        Err(TargetError::Unsupported(format!(
            "{command} has no A2A operation: the agent issues it inside the server, so the wire \
             cannot await its outcome"
        )))
    }

    fn end(&mut self) -> Result<(), TargetError> {
        self.unobserved = None;
        Ok(())
    }
}
