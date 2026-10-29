//! The conformance target over the wire: an A2A endpoint at a base URL, reached through
//! `a2a-client`, carrying out the steps of a synthesized scenario the HTTP+JSON binding can
//! observe.
//!
//! A command runs as the call [`crate::OPERATIONS`] binds it to, with the scenario's input renamed
//! as the generated contract names the wire and decoded as the generated server decodes it; the
//! input's `tenant` becomes the request's `/{tenant}` prefix. The answer reads as the outcome the
//! generated contract (`a2a-agent.openapi.json`) declares for it: a success as the command's
//! accepting outcome, an error by the declared error its `google.rpc.ErrorInfo` reason carries.
//! Every step the wire cannot observe is `TargetError::Unsupported` with the reason, never
//! `passed`.

use a2a::json::{self, Value};
use a2a::server::wire;
use a2a::{discovery, tasks};
use a2a_client::{Client, Error, Reason};
use a2a_conformance::naming::Naming;
use a2a_conformance::{Answer, Awaited, Event, Reader, Target, TargetError};

use crate::binding::carried;
use crate::contract::{Contract, Outcome};

/// The one view A2A has an operation for: `GET /tasks`, ListTasks (A2A v1.0.1 section 9.4).
const TASKS: &str = "a2a.tasks.Tasks";

/// An A2A endpoint as a [`Target`]: the client it calls as, and what the scenario open on it has
/// done.
pub struct WireTarget {
    client: Client,
    /// Why the events of the last command are not observable, where it took an outcome that
    /// publishes.
    unobserved: Option<String>,
    /// Whether a command of the open scenario took an accepting outcome, so a record it named
    /// may exist on the endpoint.
    recorded: bool,
}

impl WireTarget {
    /// The target over `client`, the endpoint's base URL and the headers every call sends.
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            unobserved: None,
            recorded: false,
        }
    }

    /// What `command` answered, read as the outcome its contract declares for that answer.
    fn read(&mut self, command: &str, called: Called) -> Result<Answer, TargetError> {
        let outcomes = Contract::get().outcomes(command);
        let (outcome, response) = match called {
            Ok(Accepted { response, outcome }) => {
                let accepting: Vec<&Outcome> = outcomes
                    .iter()
                    .filter(|outcome| outcome.error.is_none())
                    .collect();
                let taken = match (outcome, accepting.as_slice()) {
                    (_, [only]) => *only,
                    (Some(named), _) => accepting
                        .iter()
                        .find(|outcome| outcome.name == named)
                        .copied()
                        .ok_or_else(|| {
                            TargetError::Failed(format!(
                                "{command} declares no accepting outcome `{named}`"
                            ))
                        })?,
                    (None, _) => {
                        return Err(TargetError::Failed(format!(
                            "{command} declares {} accepting outcomes and the answer names none",
                            accepting.len()
                        )));
                    }
                };
                (taken, response)
            }
            Err(error) => (self.refused(command, outcomes, &error)?, None),
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
        self.unobserved = outcome.publishes.then(|| {
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

    /// The declared outcome an error answer of `command` is.
    fn refused<'a>(
        &self,
        command: &str,
        outcomes: &'a [Outcome],
        error: &Error,
    ) -> Result<&'a Outcome, TargetError> {
        if let Error::InvalidRequest(detail) = error {
            return Err(TargetError::Unsupported(format!(
                "a2a-client cannot send {command} with this input: {detail}"
            )));
        }
        let Some(short) = declared(error) else {
            return Err(TargetError::Failed(format!(
                "{command} answered {error}, which is no declared error of A2A v1.0.1"
            )));
        };
        let domain = command
            .rsplit_once('.')
            .map_or(command, |(domain, _)| domain);
        let named = format!("{domain}.{short}");
        let candidates: Vec<&Outcome> = outcomes
            .iter()
            .filter(|outcome| outcome.error.as_deref() == Some(named.as_str()))
            .collect();
        match candidates.as_slice() {
            [] => Err(TargetError::Failed(format!(
                "{command} answered {error}: {named}, which no outcome of {command} carries"
            ))),
            [only] => Ok(only),
            several => {
                let unknown: Vec<&&Outcome> = several
                    .iter()
                    .filter(|outcome| outcome.unknown_identity)
                    .collect();
                match unknown.as_slice() {
                    [only] if !self.recorded => Ok(only),
                    _ => Err(TargetError::Unsupported(format!(
                        "{command} answered {named}, which its outcomes {} all carry; the wire \
                         does not tell them apart",
                        several
                            .iter()
                            .map(|outcome| format!("`{}`", outcome.name))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))),
                }
            }
        }
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

/// The short name of the declared error an error answer is: its `google.rpc.ErrorInfo` reason
/// as A2A v1.0.1 section 5.4 maps it, or `RESOURCE_EXHAUSTED`, which carries no reason
/// (`crates/a2a-server/src/router.rs`, `declared_error`).
fn declared(error: &Error) -> Option<&'static str> {
    let Error::Status { status, reason, .. } = error else {
        return None;
    };
    Some(match reason {
        Some(Reason::TaskNotFound) => "TaskNotFound",
        Some(Reason::TaskNotCancelable) => "TaskNotCancelable",
        Some(Reason::PushNotificationNotSupported) => "PushNotificationNotSupported",
        Some(Reason::UnsupportedOperation) => "UnsupportedOperation",
        Some(Reason::ContentTypeNotSupported) => "ContentTypeNotSupported",
        Some(Reason::ExtendedAgentCardNotConfigured) => "ExtendedAgentCardNotConfigured",
        Some(Reason::ExtensionSupportRequired) => "ExtensionSupportRequired",
        Some(Reason::VersionNotSupported) => "VersionNotSupported",
        Some(Reason::ContextMismatch) => "ContextMismatch",
        Some(Reason::ContextNotAccepted) => "ContextNotAccepted",
        None if status == "RESOURCE_EXHAUSTED" => "ResourceExhausted",
        Some(Reason::InvalidAgentResponse | Reason::Other(_)) | None => return None,
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
    fn begin(&mut self, _scenario: &str) -> Result<(), TargetError> {
        self.unobserved = None;
        self.recorded = false;
        Ok(())
    }

    fn execute(
        &mut self,
        command: &str,
        actor: Option<&str>,
        input: &Value,
    ) -> Result<Answer, TargetError> {
        self.unobserved = None;
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
        self.read(command, called)
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
