//! The target for `a2a-server`: every scenario on a fresh in-process server
//! ([`a2a_server::InProcess`]), the generated system over the ports the binary serves, its events
//! read off the system's log.
//!
//! What the specification leaves externally decided is decided by the server's circumstances:
//! the agent's answer to a message, the card, the limits, and the tasks already stored. A forced
//! outcome is brought about by arranging those circumstances for the one run it applies to, once
//! the caller is admitted, and setting them back after it:
//!
//! | outcome | arrangement |
//! |---|---|
//! | `version-not-supported` | the run names `A2A-Version` 0.1, which the card's interfaces do not declare |
//! | `extension-required` | the card requires an extension the run does not name |
//! | `streaming-not-supported`, `push-not-supported`, `not-supported` | the card does not declare the capability |
//! | `not-configured` | no extended card is configured |
//! | `content-type-not-supported`, `context-not-accepted`, `direct-message` | the agent refuses the message, or answers it directly |
//! | `not-cancelable-now` | the agent answers that the task cannot be canceled at its stage |
//! | `task-limit`, `config-limit` | the limit is 0 |
//! | `continued`, `history-limit`, `task-terminal`, `context-mismatch` | the message names a task an earlier message created: as it is, at a history limit of one message, canceled, or of another context |
//! | `task-not-found` | the message names a task id nothing created |
//!
//! The server is configured with an extended card that differs from the public one and a bearer
//! token for it, which every run as `a2a.discovery.Client` presents, as an authenticated client
//! of the binary does (A2A v1.0.1 section 13.3).
//!
//! What an arrangement publishes is not the run's and is dropped. Otherwise the agent takes every
//! message into a task and asks for no step: the system's own commands are issued when a
//! scenario awaits them, as the server loop issues an agent's step.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use a2a::actor::{Actor, Caller};
use a2a::discovery::AgentExtension;
use a2a::json::{self, DecodeError, Value};
use a2a::primitives::Timestamp;
use a2a::server::entry::Refused;
use a2a::server::{a2a_agent, wire};
use a2a::system::SystemEvent;
use a2a::tasks::{Artifact, Message, Part, TaskData, TaskEntitySnapshot, TaskId, TaskSnapshot};
use a2a_server::{
    Access, Agent, BearerToken, Called, InProcess, Limits, Obligations, Refusal, ServerPorts,
    Settings, Step, Turn,
};

use crate::naming::Naming;
use crate::target::{Answer, Awaited, Event, Reader, Target, TargetError};

/// The protocol version every run names, the one the card's interface declares.
const VERSION: &str = "1.0";

/// A version no interface of the card declares.
const UNSPOKEN_VERSION: &str = "0.1";

/// The extension the card requires where a run is to come out `extension-required`.
const REQUIRED_EXTENSION: &str = "urn:a2a-conformance:required";

/// The context of the task a `context-mismatch` message names.
const OTHER_CONTEXT: &str = "a2a-conformance-other-context";

/// The task id a `task-not-found` message names.
const UNKNOWN_TASK: &str = "a2a-conformance-unknown-task";

/// The bearer token the server is configured to serve the extended card for, which every run as
/// `a2a.discovery.Client` presents (A2A v1.0.1 section 13.3).
const CARD_TOKEN: &str = "a2a-conformance-card-token";

/// The actor the token is presented for: the one the binary runs `GET /extendedAgentCard` as.
const CARD_READER: Actor = Actor::DiscoveryClient;

/// The entity an authored `setup:` may hand the server.
const TASK: &str = "a2a.tasks.Task";

/// The view whose rows spell every field of a task, as a setup's fields are read.
const ALL_TASKS: &str = "a2a.tasks.AllTasks";

/// The commands whose tenant guard the model leaves unmapped, and why a run of one on a task of
/// another tenant is not held to the suite.
// ESS gap (ess 0.75.0): `ess/domains/tasks.yaml` leaves the tenant guard of these three commands
// UNMAPPED (CreateTaskPushNotificationConfig: "a related-row guard comparing a stored Optional
// with an input Optional that synthesis can witness" is missing), so synthesis arranges them on a
// task created under another tenant and expects `created`, `assigned`, `config-limit`, `found`,
// `config-not-found` or `deleted`. a2a-server answers such a request TaskNotFound by hand, as
// GetTask `other-tenant` declares for its own command (A2A v1.0.1 section 13.1). Such a run is
// declared unsupported with this reason rather than failed against a model that cannot say it,
// with one exception: a run that names no tenant and only arranges what a later step reads (a
// command other than the one the scenario is about, its id up to the first `/`, such as the
// configurations GetTaskPushNotificationConfig `found` reads) is sent under the tenant the task
// was created under, so the step the scenario is about runs and is held to the suite. A run that
// names a tenant keeps it: the configuration would store another tenant than the one it names,
// which a later claim reads (the PushNotificationConfigs rows of the TaskPushNotificationConfigs
// grant scenarios claim `tenant`). Except in the scenarios of [`REARRANGED`].
const ACROSS_TENANTS: [&str; 3] = [
    "a2a.tasks.CreateTaskPushNotificationConfig",
    "a2a.tasks.GetTaskPushNotificationConfig",
    "a2a.tasks.DeleteTaskPushNotificationConfig",
];

/// The scenarios, by the command they are about, whose arranging runs are sent under the task's
/// tenant even where they name another.
// ESS gap (ess 0.75.0): since the TaskPushNotificationConfigs view takes a `tenant` parameter
// (story:view-tenant-scope), synthesis arranges the configurations these scenarios read under
// tenants of its own (`tenant-1` …) on a task created under `tenant`, as the configuration's
// tenant is a free field to the model (the UNMAPPED tenant guard above). No claim of these
// scenarios reads a configuration's tenant (`found` claims the response's shape), so the
// arrangement runs under the task's tenant and the step the scenario is about stays held to the
// suite; without it, removing GetTaskPushNotificationConfig's tenant guard leaves the gate green
// (`tests/sec_tenant_guards.rs`).
const REARRANGED: [&str; 2] = [
    "a2a.tasks.GetTaskPushNotificationConfig",
    "a2a.tasks.DeleteTaskPushNotificationConfig",
];

/// The reason a run of an [`ACROSS_TENANTS`] command on another tenant's task is unsupported.
fn across_tenants(command: &str) -> String {
    format!(
        "{command} on a task of another tenant: a2a-server answers TaskNotFound (A2A v1.0.1 section 13.1), a guard ess/domains/tasks.yaml leaves UNMAPPED, so the synthesized scenario expects the command to act across tenants"
    )
}

/// What the agent answers, set for one message or one cancel.
#[derive(Default)]
struct Script {
    turn: RefCell<Option<Turn>>,
    not_cancelable: Cell<bool>,
}

/// The agent of the target: the scripted turn where one is set, else a task with no steps.
struct Scripted(Rc<Script>);

impl Agent for Scripted {
    fn turn(&mut self, _message: &Message, _task: Option<&TaskSnapshot>) -> Turn {
        self.0
            .turn
            .borrow_mut()
            .take()
            .unwrap_or(Turn::Steps(Vec::new()))
    }

    fn cancelable(&self, _task: &TaskSnapshot) -> bool {
        !self.0.not_cancelable.replace(false)
    }
}

/// The settings every run is decided by unless a forced outcome arranges others: the built-in
/// card, declaring an extended card, and an extended card that differs from it, so an answer of
/// the public card where the extended one is due is not the extended card.
fn settings() -> Settings {
    let mut card = a2a_server::default_card();
    card.capabilities.extended_agent_card = Some(true);
    let mut extended = card.clone();
    extended.name = format!("{} (extended)", card.name);
    extended.description = format!("{} Served to authenticated clients.", card.description);
    Settings {
        extended_card: Some(extended),
        card,
    }
}

/// What the server admits: the extended card for [`CARD_TOKEN`].
fn access() -> Access {
    Access {
        extended_card_token: BearerToken::new(CARD_TOKEN),
    }
}

/// The circumstances one run is decided in.
struct Run {
    version: String,
    settings: Settings,
    limits: Limits,
    input: Value,
}

/// `a2a-server` as a [`Target`]: a fresh [`InProcess`] server for each scenario, its generated
/// system over `P`, the server's own ports or a wrapper of them.
pub struct ServerTarget<P = ServerPorts> {
    wrap: fn(ServerPorts) -> P,
    server: Option<InProcess<P>>,
    script: Rc<Script>,
    forced: Option<(String, String)>,
    pending: Vec<Event>,
    /// The tenant each task a scenario's message created was created under.
    tenants: Vec<(String, Option<String>)>,
    /// The command or view the open scenario is about: its id up to the first `/`.
    subject: String,
}

impl ServerTarget<ServerPorts> {
    /// The target over the server's own ports.
    #[must_use]
    pub fn new() -> Self {
        Self::wrapped(|ports| ports)
    }
}

impl Default for ServerTarget<ServerPorts> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Obligations> ServerTarget<P> {
    /// The target over `wrap` of the server's ports: a wrapper that keeps an obligation
    /// otherwise.
    #[must_use]
    pub fn wrapped(wrap: fn(ServerPorts) -> P) -> Self {
        Self {
            wrap,
            server: None,
            script: Rc::new(Script::default()),
            forced: None,
            pending: Vec::new(),
            tenants: Vec::new(),
            subject: String::new(),
        }
    }

    /// The tenant a message of this scenario created the task `input` of `command` names under,
    /// where `command` is one of [`ACROSS_TENANTS`] and `input` names another tenant.
    fn owner_elsewhere(&self, command: &str, input: &Value) -> Option<Option<String>> {
        if !ACROSS_TENANTS.contains(&command) {
            return None;
        }
        let task = text(input.member("taskId"))?;
        let tenant = text(input.member("tenant"));
        self.tenants
            .iter()
            .find(|(created, under)| *created == task && *under != tenant)
            .map(|(_, under)| under.clone())
    }

    fn server(&mut self) -> Result<&mut InProcess<P>, TargetError> {
        self.server
            .as_mut()
            .ok_or_else(|| TargetError::Failed("no scenario is open".to_owned()))
    }

    /// Runs `name` as `caller` in the circumstances `run`, then sets the settings back. A run as
    /// [`CARD_READER`] presents [`CARD_TOKEN`], as an authenticated client of the binary sends it.
    fn call(
        &mut self,
        caller: Option<&Caller>,
        name: &str,
        run: Run,
    ) -> Result<Called, TargetError> {
        let presented = caller
            .is_some_and(|caller| caller.actor == CARD_READER)
            .then_some(CARD_TOKEN);
        let server = self.server()?;
        server.reconfigure(run.settings, run.limits);
        let called = server.call_presenting(
            caller,
            presented,
            Some(run.version),
            Vec::new(),
            name,
            run.input,
        );
        server.reconfigure(settings(), Limits::DEFAULT);
        Ok(called)
    }

    /// The circumstances that make `command`, run with `input`, come out `outcome`.
    fn arrange(&mut self, command: &str, outcome: &str, input: Value) -> Result<Run, TargetError> {
        let mut run = Run {
            version: VERSION.to_owned(),
            settings: settings(),
            limits: Limits::DEFAULT,
            input,
        };
        let capabilities = &mut run.settings.card.capabilities;
        match outcome {
            "version-not-supported" => run.version = UNSPOKEN_VERSION.to_owned(),
            "extension-required" => {
                capabilities.extensions = Some(vec![AgentExtension {
                    uri: Some(REQUIRED_EXTENSION.to_owned()),
                    description: None,
                    required: Some(true),
                    params: None,
                }]);
            }
            "streaming-not-supported" => capabilities.streaming = Some(false),
            "push-not-supported" => capabilities.push_notifications = Some(false),
            "not-supported" => capabilities.extended_agent_card = Some(false),
            "not-configured" => run.settings.extended_card = None,
            "content-type-not-supported" => self.script(Turn::Refuse(Refusal::ContentType)),
            "context-not-accepted" => self.script(Turn::Refuse(Refusal::Context)),
            "direct-message" => self.script(Turn::Reply(reply())),
            "not-cancelable-now" => self.script.not_cancelable.set(true),
            "task-limit" => run.limits.max_tasks = 0,
            "config-limit" => run.limits.max_push_configs = 0,
            "task-not-found" => run.input = naming_task(run.input, UNKNOWN_TASK),
            "continued" | "history-limit" | "task-terminal" | "context-mismatch" => {
                let context = match outcome {
                    "context-mismatch" => Some(OTHER_CONTEXT.to_owned()),
                    _ => {
                        text(message_of(&run.input).and_then(|message| message.member("contextId")))
                    }
                };
                let tenant = text(run.input.member("tenant"));
                let task = self.earlier_task(command, context, tenant.clone())?;
                if outcome == "task-terminal" {
                    self.cancel(&task, tenant)?;
                }
                if outcome == "history-limit" {
                    run.limits.max_history = 1;
                }
                if outcome == "context-mismatch"
                    && message_of(&run.input)
                        .and_then(|message| message.member("contextId"))
                        .is_none()
                {
                    run.input = with_message_member(
                        run.input,
                        "contextId",
                        Value::Text("a2a-conformance-context".to_owned()),
                    );
                }
                run.input = naming_task(run.input, &task);
            }
            other => {
                return Err(TargetError::Failed(format!(
                    "no arrangement brings {command} to `{other}`"
                )));
            }
        }
        Ok(run)
    }

    fn script(&self, turn: Turn) {
        *self.script.turn.borrow_mut() = Some(turn);
    }

    /// A task an earlier message created, in `context` and under `tenant`; what creating it
    /// published is dropped.
    fn earlier_task(
        &mut self,
        command: &str,
        context: Option<String>,
        tenant: Option<String>,
    ) -> Result<String, TargetError> {
        let mut input = String::from(
            r#"{"message":{"messageId":"a2a-conformance-earlier","role":"ROLE_USER","parts":[]"#,
        );
        if let Some(context) = &context {
            json::member(&mut input, "contextId");
            json::push_text(&mut input, context);
        }
        input.push('}');
        if let Some(tenant) = &tenant {
            json::member(&mut input, "tenant");
            json::push_text(&mut input, tenant);
        }
        input.push('}');
        let input = parse(&input)?;
        let client = Caller {
            actor: Actor::TasksClient,
        };
        let run = Run {
            version: VERSION.to_owned(),
            settings: settings(),
            limits: Limits::DEFAULT,
            input,
        };
        let called = self.call(Some(&client), command, run)?;
        called
            .events
            .iter()
            .find_map(|event| match event {
                SystemEvent::TaskCreated(created) => Some(created.task_id.0.clone()),
                _ => None,
            })
            .ok_or_else(|| {
                TargetError::Failed(format!(
                    "arranging an earlier task: {command} created none: {:?}",
                    called.answer
                ))
            })
    }

    /// Cancels `task`; what canceling it published is dropped.
    fn cancel(&mut self, task: &str, tenant: Option<String>) -> Result<(), TargetError> {
        let mut input = String::from("{");
        json::member(&mut input, "id");
        json::push_text(&mut input, task);
        if let Some(tenant) = &tenant {
            json::member(&mut input, "tenant");
            json::push_text(&mut input, tenant);
        }
        input.push('}');
        let client = Caller {
            actor: Actor::TasksClient,
        };
        let run = Run {
            version: VERSION.to_owned(),
            settings: settings(),
            limits: Limits::DEFAULT,
            input: parse(&input)?,
        };
        let called = self.call(Some(&client), "a2a.tasks.CancelTask", run)?;
        match &called.answer {
            Ok(envelope)
                if envelope.member("outcome") == Some(&Value::Text("canceled".to_owned())) =>
            {
                Ok(())
            }
            other => Err(TargetError::Failed(format!(
                "arranging a terminal task: CancelTask answered {other:?}"
            ))),
        }
    }

    /// Keeps what `events` published for [`Target::published`], ESS-named.
    fn keep(&mut self, events: &[SystemEvent]) -> Result<(), TargetError> {
        for event in events {
            let envelope = parse(&wire::encode_system_event(event))?;
            let payload = envelope.member("payload").cloned().unwrap_or(Value::Null);
            let name = event.name();
            self.pending.push(Event {
                name: name.to_owned(),
                payload: Naming::contract().event_to_ess(name, &payload),
            });
        }
        Ok(())
    }
}

impl<P: Obligations> Target for ServerTarget<P> {
    fn begin(&mut self, scenario: &str) -> Result<(), TargetError> {
        scenario
            .split('/')
            .next()
            .unwrap_or_default()
            .clone_into(&mut self.subject);
        self.script = Rc::new(Script::default());
        self.forced = None;
        self.pending.clear();
        self.tenants.clear();
        self.server = Some(
            InProcess::wrapped(
                settings(),
                Box::new(Scripted(Rc::clone(&self.script))),
                Limits::DEFAULT,
                self.wrap,
            )
            .with_access(access()),
        );
        Ok(())
    }

    /// Stores an `a2a.tasks.Task` as a2a-server's task store holds one
    /// ([`InProcess::establish_task`]); its fields are read as the `a2a.tasks.AllTasks` view
    /// spells a task, which carries every field the entity declares.
    fn establish(
        &mut self,
        entity: &str,
        identity: &Value,
        fields: &Value,
        state: &str,
    ) -> Result<(), TargetError> {
        if entity != TASK {
            return Err(TargetError::Unsupported(format!(
                "a2a-conformance sets up no {entity} in a2a-server: only {TASK}"
            )));
        }
        let Value::Text(id) = identity else {
            return Err(TargetError::Failed(format!(
                "a {TASK} is set up with a text identity, not {identity:?}"
            )));
        };
        let task = task_of(id, fields, state).map_err(|error| {
            TargetError::Failed(format!("the setup of {TASK} `{id}` is no task: {error}"))
        })?;
        self.tenants.push((id.clone(), task.data.tenant.clone()));
        self.server()?.establish_task(task);
        Ok(())
    }

    fn execute(
        &mut self,
        command: &str,
        actor: Option<&str>,
        input: &Value,
    ) -> Result<Answer, TargetError> {
        let caller = actor.map(caller).transpose()?;
        let mut input = Naming::contract().input_to_wire(command, input);
        let admitted = a2a_agent::admit(caller.as_ref(), command).is_ok();
        let across = match self.owner_elsewhere(command, &input) {
            Some(owner)
                if command != self.subject
                    && (input.member("tenant").is_none()
                        || REARRANGED.contains(&self.subject.as_str())) =>
            {
                input = with_tenant(input, owner);
                false
            }
            owner => owner.is_some(),
        };
        let tenant = text(input.member("tenant"));
        let forced = match self.forced.take() {
            Some((forced, outcome)) if forced == command && admitted => Some(outcome),
            kept => {
                self.forced = kept;
                None
            }
        };
        let run = match forced {
            Some(outcome) => self.arrange(command, &outcome, input)?,
            None => Run {
                version: VERSION.to_owned(),
                settings: settings(),
                limits: Limits::DEFAULT,
                input,
            },
        };
        let called = self.call(caller.as_ref(), command, run)?;
        for event in &called.events {
            if let SystemEvent::TaskCreated(created) = event {
                self.tenants
                    .push((created.task_id.0.clone(), tenant.clone()));
            }
        }
        self.keep(&called.events)?;
        let answer = answer(command, called.answer)?;
        if across && matches!(answer.outcome.as_str(), "task-not-found" | "not-found") {
            return Err(TargetError::Unsupported(across_tenants(command)));
        }
        Ok(answer)
    }

    fn query(
        &mut self,
        view: &str,
        reader: Reader<'_>,
        params: &Value,
    ) -> Result<Vec<Value>, TargetError> {
        let caller = match reader {
            Reader::Harness => Some(harness_reader(view)?),
            Reader::Actor(actor) => Some(caller(actor)?),
            Reader::Anonymous => None,
        };
        let run = Run {
            version: VERSION.to_owned(),
            settings: settings(),
            limits: Limits::DEFAULT,
            input: Naming::contract().input_to_wire(view, params),
        };
        let called = self.call(caller.as_ref(), view, run)?;
        self.keep(&called.events)?;
        match called.answer {
            Ok(answer) => match answer.member("rows") {
                Some(Value::Array(rows)) => Ok(rows
                    .iter()
                    .map(|row| Naming::contract().row_to_ess(view, row))
                    .collect()),
                _ => Err(TargetError::Failed(format!(
                    "{view} answered no rows: {answer:?}"
                ))),
            },
            Err(refused) => Err(refusal(view, refused)),
        }
    }

    fn force(&mut self, command: &str, outcome: &str) -> Result<(), TargetError> {
        if !arranged(command, outcome) {
            return Err(TargetError::Unsupported(format!(
                "a2a-conformance has no arrangement that brings a2a-server's {command} to `{outcome}`"
            )));
        }
        self.forced = Some((command.to_owned(), outcome.to_owned()));
        Ok(())
    }

    fn published(&mut self) -> Result<Vec<Event>, TargetError> {
        Ok(std::mem::take(&mut self.pending))
    }

    fn await_outcome(&mut self, command: &str, instance: &Value) -> Result<Awaited, TargetError> {
        let Value::Text(task) = instance else {
            return Err(TargetError::Failed(format!(
                "{command} is issued on a task id, and the instance is {instance:?}"
            )));
        };
        let step = match command {
            "a2a.tasks.StartWorking" => Step::Work(None),
            "a2a.tasks.CompleteTask" => Step::Complete(None),
            "a2a.tasks.FailTask" => Step::Fail(None),
            "a2a.tasks.RejectTask" => Step::Reject(None),
            "a2a.tasks.RequireInput" => Step::RequireInput(None),
            "a2a.tasks.RequireAuth" => Step::RequireAuth(None),
            "a2a.tasks.UpdateArtifact" => Step::Emit {
                artifact: chunk(),
                append: false,
                last_chunk: true,
            },
            other => {
                return Err(TargetError::Unsupported(format!(
                    "a2a-server's agent asks for no step that issues {other}"
                )));
            }
        };
        let Some((input, called)) = self.server()?.issue(&TaskId(task.clone()), step) else {
            return Ok(Awaited::NotIssued);
        };
        self.keep(&called.events)?;
        let answer = answer(command, called.answer)?;
        if answer.error.is_some() {
            return Ok(Awaited::NotIssued);
        }
        Ok(Awaited::Reached {
            answer,
            input: Naming::contract().input_to_ess(command, &input),
        })
    }

    fn end(&mut self) -> Result<(), TargetError> {
        self.server = None;
        Ok(())
    }
}

/// Whether [`ServerTarget::arrange`] brings `command` to `outcome`.
fn arranged(command: &str, outcome: &str) -> bool {
    let message = matches!(
        command,
        "a2a.tasks.SendMessage" | "a2a.tasks.SendStreamingMessage"
    );
    let configs = matches!(
        command,
        "a2a.tasks.CreateTaskPushNotificationConfig"
            | "a2a.tasks.GetTaskPushNotificationConfig"
            | "a2a.tasks.ListTaskPushNotificationConfigs"
            | "a2a.tasks.DeleteTaskPushNotificationConfig"
    );
    match outcome {
        "version-not-supported" | "extension-required" => true,
        "streaming-not-supported" => {
            matches!(
                command,
                "a2a.tasks.SendStreamingMessage" | "a2a.tasks.SubscribeToTask"
            )
        }
        "push-not-supported" => configs,
        "config-limit" => command == "a2a.tasks.CreateTaskPushNotificationConfig",
        "not-supported" | "not-configured" => command == "a2a.discovery.GetExtendedAgentCard",
        "not-cancelable-now" => command == "a2a.tasks.CancelTask",
        "content-type-not-supported"
        | "context-not-accepted"
        | "direct-message"
        | "task-limit"
        | "task-not-found"
        | "continued"
        | "history-limit"
        | "task-terminal"
        | "context-mismatch" => message,
        _ => false,
    }
}

/// The caller of the declared actor `name`.
fn caller(name: &str) -> Result<Caller, TargetError> {
    Actor::ALL
        .iter()
        .find(|actor| actor.name() == name)
        .map(|actor| Caller { actor: *actor })
        .ok_or_else(|| TargetError::Failed(format!("a2a-server declares no actor `{name}`")))
}

/// The actor whose read grant names `view`: who the harness reads it as.
fn harness_reader(view: &str) -> Result<Caller, TargetError> {
    Actor::ALL
        .iter()
        .map(|actor| Caller { actor: *actor })
        .find(|caller| caller.may_read(view))
        .ok_or_else(|| TargetError::Failed(format!("no actor's grant names the view `{view}`")))
}

/// What a command's envelope answered, ESS-named.
fn answer(command: &str, answered: Result<Value, Refused>) -> Result<Answer, TargetError> {
    let envelope = answered.map_err(|refused| refusal(command, refused))?;
    let Some(Value::Text(outcome)) = envelope.member("outcome") else {
        return Err(TargetError::Failed(format!(
            "{command} answered no outcome: {envelope:?}"
        )));
    };
    Ok(Answer {
        outcome: outcome.clone(),
        error: text(envelope.member("error")),
        response: envelope
            .member("response")
            .map(|response| Naming::contract().response_to_ess(command, response)),
    })
}

fn refusal(name: &str, refused: Refused) -> TargetError {
    match refused {
        Refused::NotGranted(actor) => TargetError::NotGranted(actor),
        other => TargetError::Failed(format!(
            "a2a-server answered {name} with no declared outcome: {other:?}"
        )),
    }
}

/// The message object of a wire-named message input.
fn message_of(input: &Value) -> Option<&Value> {
    input.member("message")
}

/// `input` with its message's member `name` set to `value`.
fn with_message_member(input: Value, name: &str, value: Value) -> Value {
    let Value::Object(mut members) = input else {
        return input;
    };
    for (key, member) in &mut members {
        if key == "message"
            && let Value::Object(fields) = member
        {
            fields.retain(|(field, _)| field != name);
            fields.push((name.to_owned(), value));
            break;
        }
    }
    Value::Object(members)
}

/// The task `id` in the lifecycle state `state` with the ESS-named `fields` of an entity setup.
fn task_of(id: &str, fields: &Value, state: &str) -> Result<TaskEntitySnapshot, DecodeError> {
    let row = Naming::contract().row_to_wire(ALL_TASKS, fields);
    let present = |name: &str| present(&row, name);
    let context_id = match present("context_id") {
        Some(value) => wire::decode_a2a_tasks_context_id(value, "context_id")?,
        None => {
            return Err(DecodeError {
                at: "context_id".to_owned(),
                expected: "a context id".to_owned(),
                found: "nothing".to_owned(),
            });
        }
    };
    Ok(TaskEntitySnapshot {
        state: wire::decode_a2a_tasks_task_state(&Value::Text(state.to_owned()), "state")?,
        data: TaskData {
            id: TaskId(id.to_owned()),
            context_id,
            tenant: text(present("tenant")),
            status_message: present("status_message")
                .map(|message| wire::decode_a2a_tasks_message(message, "status_message"))
                .transpose()?,
            status_timestamp: text(present("status_timestamp")).map(Timestamp),
            artifacts: list(&row, "artifacts", wire::decode_a2a_tasks_artifact)?,
            history: list(&row, "history", wire::decode_a2a_tasks_message)?,
            metadata: present("metadata").cloned(),
        },
    })
}

/// The member `name` of `row`, where it holds a value other than `null`.
fn present<'a>(row: &'a Value, name: &str) -> Option<&'a Value> {
    row.member(name).filter(|value| **value != Value::Null)
}

/// The list the member `name` of `row` holds, each item read by `decode`; `None` where it holds
/// none.
fn list<T>(
    row: &Value,
    name: &str,
    decode: fn(&Value, &str) -> Result<T, DecodeError>,
) -> Result<Option<Vec<T>>, DecodeError> {
    match present(row, name) {
        Some(Value::Array(items)) => items
            .iter()
            .enumerate()
            .map(|(at, item)| decode(item, &format!("{name}[{at}]")))
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        Some(other) => Err(DecodeError {
            at: name.to_owned(),
            expected: "a list".to_owned(),
            found: format!("{other:?}"),
        }),
        None => Ok(None),
    }
}

/// `input` under the tenant `tenant`: its `tenant` member set to it, or left out for none.
fn with_tenant(input: Value, tenant: Option<String>) -> Value {
    let Value::Object(mut members) = input else {
        return input;
    };
    members.retain(|(key, _)| key != "tenant");
    if let Some(tenant) = tenant {
        members.push(("tenant".to_owned(), Value::Text(tenant)));
    }
    Value::Object(members)
}

/// `input` with its message naming the task `task`.
fn naming_task(input: Value, task: &str) -> Value {
    with_message_member(input, "taskId", Value::Text(task.to_owned()))
}

fn text(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::Text(text)) => Some(text.clone()),
        _ => None,
    }
}

fn parse(text: &str) -> Result<Value, TargetError> {
    json::parse_written(text)
        .map_err(|error| TargetError::Failed(format!("the target wrote no JSON: {error}")))
}

/// The direct reply of a `direct-message` arrangement.
fn reply() -> Message {
    a2a_server::agent::agent_message(vec![text_part("a direct reply")])
}

/// The artifact an awaited `UpdateArtifact` adds.
fn chunk() -> Artifact {
    Artifact {
        artifact_id: "artifact-1".to_owned(),
        name: None,
        description: None,
        parts: vec![text_part("chunk")],
        metadata: None,
        extensions: None,
    }
}

fn text_part(text: &str) -> Part {
    Part {
        text: Some(text.to_owned()),
        raw: None,
        url: None,
        data: None,
        metadata: None,
        filename: None,
        media_type: None,
    }
}
