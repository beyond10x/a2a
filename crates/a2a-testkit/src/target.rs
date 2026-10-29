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
//! The events an outcome publishes are read from what the endpoint answered, never from what the
//! suite expects: `a2a.tasks.TaskCreated` from a message answered with a task the run had
//! not seen, `a2a.tasks.MessageStreamResponse` from the first element of a message stream,
//! `a2a.tasks.TaskStatusUpdated` from the task a cancel answers, and one `TaskStatusUpdated` or
//! `TaskArtifactUpdated` per status or artifact element a stream carries after its first. A stream
//! is followed until it closes or no element arrives for [`STREAM_QUIET`]. An event no answer
//! carries is `unsupported`, and so is an answer showing a task in another state than the
//! scenario's commands left it in: the agent moved it by steps of its own, which the wire neither
//! arranges nor tells apart from the command's events.
//!
//! The views read the tasks the open scenario created: `a2a.tasks.AllTasks` each read back with
//! GetTask under the tenant it was created under, `a2a.tasks.Tasks` listed with ListTasks under
//! the parameters' tenant, keeping those tasks only, as a live endpoint holds tasks no scenario
//! created and cannot be reset to the empty state each scenario starts from.
//!
//! Given the suite ([`WireTarget::with_suite`]), the target knows the outcome each step expects:
//! an answer that is not that outcome is reported as the outcome it is, or as the answer no
//! outcome declares, so the scenario fails on it rather than coming out `unsupported` on the
//! events the outcome would publish.

use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::time::Duration;

use a2a::discovery::AgentCard;
use a2a::json::{self, Value};
use a2a::server::wire;
use a2a::system::SystemEvent;
use a2a::tasks::{ContextId, TaskId, TaskSnapshot, TaskState, WireTaskState};
use a2a::{discovery, tasks};
use a2a_client::{Client, Error, Events, ListTasksRequest, Reason};
use a2a_conformance::naming::Naming;
use a2a_conformance::{
    ACROSS_TENANTS, Answer, Awaited, Event, Reader, Target, TargetError, across_tenants,
};

use crate::binding::carried;
use crate::contract::{Contract, Outcome, error_of};
use crate::expected::Expectations;

/// The one view A2A has an operation for: `GET /tasks`, ListTasks (A2A v1.0.1 section 9.4).
const TASKS: &str = "a2a.tasks.Tasks";

/// The view of every task, which A2A has no operation for: the target answers it with the tasks
/// the open scenario created, each read back with GetTask.
const ALL_TASKS: &str = "a2a.tasks.AllTasks";

/// The command whose answer A2A v1.0.1 section 3.1.11 gives to authenticated clients only.
const EXTENDED_CARD: &str = "a2a.discovery.GetExtendedAgentCard";

/// The event a message answered with a new task publishes.
const TASK_CREATED: &str = "a2a.tasks.TaskCreated";

/// The event the first element of a message stream is.
const MESSAGE_STREAM_RESPONSE: &str = "a2a.tasks.MessageStreamResponse";

/// The event a move of a task publishes.
const TASK_STATUS_UPDATED: &str = "a2a.tasks.TaskStatusUpdated";

/// How long a stream is followed for its next element: past it, the stream is closed and what it
/// carried so far is what the command's events are read from. Each element of a stream and the
/// head of its answer must arrive within it.
pub const STREAM_QUIET: Duration = Duration::from_secs(2);

/// The most elements one stream is followed for.
const MOST_ELEMENTS: usize = 1024;

/// The most pages one read of [`TASKS`] lists.
const MOST_PAGES: usize = 1024;

/// The state a created task is in until a command moves it: `initial: Submitted` of
/// `a2a.tasks.Task` (`ess/domains/tasks.yaml`).
const INITIAL: WireTaskState = WireTaskState::TASKSTATESUBMITTED;

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
    /// What the last command published, read from its answer, until [`Target::published`] takes
    /// it: the events, or why they cannot be observed.
    pending: Option<Result<Vec<Event>, TargetError>>,
    /// Whether a command of the open scenario took an accepting outcome, so a record it named
    /// may exist on the endpoint.
    recorded: bool,
    /// The tasks the open scenario created, in order.
    created: Vec<Created>,
    /// Every task the endpoint answered in this run, across scenarios: a task an earlier answer
    /// showed is never one a later message created.
    answered: BTreeSet<String>,
}

/// A task the open scenario created: what the answer and the request carried of it, and the state
/// the endpoint last answered it in.
#[derive(Debug, Clone)]
struct Created {
    id: String,
    /// The request's tenant.
    tenant: Option<String>,
    state: WireTaskState,
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
            pending: None,
            recorded: false,
            created: Vec::new(),
            answered: BTreeSet::new(),
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

    /// The client a stream is read with: each element, and the head, within [`STREAM_QUIET`].
    fn streaming(&self) -> Client {
        self.client.clone().with_timeout(Some(STREAM_QUIET))
    }

    /// The state the scenario's commands left the task `id` in, where the scenario created it.
    fn left_in(&self, id: &str) -> Option<WireTaskState> {
        self.created
            .iter()
            .find(|created| created.id == id)
            .map(|created| created.state)
    }

    /// What a success answer holding a task the request did not ask for answered: a task other
    /// than the one `input` names, or, for a request naming none, a task an earlier scenario's
    /// answer showed, which no command of this scenario can have reached.
    fn foreign(&self, input: &Value, carried: &Carried) -> Option<String> {
        let task = match carried {
            Carried::Nothing => return None,
            Carried::Task(task) | Carried::Subscription { task, .. } => task,
            Carried::Message { first, .. } => first.task.as_ref()?,
        };
        let id = task.id.0.as_str();
        if let Some(asked) = text(input.member("id")) {
            return (asked != id).then(|| {
                format!("a success answer holding task `{id}` to a request naming task `{asked}`")
            });
        }
        let named = text(
            input
                .member("message")
                .and_then(|message| message.member("taskId")),
        );
        (self.answered.contains(id) && self.left_in(id).is_none() && named.as_deref() != Some(id))
            .then(|| {
                format!(
                    "a success answer holding task `{id}`, which an earlier scenario's answer \
                     showed"
                )
            })
    }

    /// Whether `input` of `command` names a task the scenario created under another tenant than
    /// its own, for a command whose tenant guard the model leaves unmapped ([`ACROSS_TENANTS`]).
    fn across(&self, command: &str, input: &Value) -> bool {
        if !ACROSS_TENANTS.contains(&command) {
            return false;
        }
        let Some(task) = text(input.member("taskId")) else {
            return false;
        };
        let tenant = text(input.member("tenant"));
        self.created
            .iter()
            .any(|created| created.id == task && created.tenant != tenant)
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
        let (candidates, response, carried, answered) = match called {
            Ok(Accepted {
                response,
                outcome,
                carried,
            }) => (
                outcomes
                    .iter()
                    .filter(|declared| {
                        declared.error.is_none()
                            && outcome.is_none_or(|named| declared.name == named)
                    })
                    .collect(),
                response,
                carried,
                "a success answer".to_owned(),
            ),
            Err(error) => {
                let (candidates, answered) = self.refusals(command, outcomes, &error)?;
                (candidates, None, Carried::Nothing, answered)
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
        let taken = match (self.foreign(input, &carried), left.as_slice()) {
            (Some(foreign), _) => Err(foreign),
            (None, []) if ruled_out.is_empty() => Err(format!("no declared outcome: {answered}")),
            (None, []) => Err(format!(
                "no declared outcome: {answered}, whose outcome {} the circumstances rule out",
                ruled_out.join(" and ")
            )),
            (None, [only]) => Ok(*only),
            (None, several) => {
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
                self.pending = None;
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
        let published = self.derive(command, outcome, input, carried);
        self.pending = Some(match published {
            Err(_) if differs => Ok(Vec::new()),
            published => published,
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

    /// The events `command` published in taking `outcome`, read from what its answer `carried`,
    /// and the tasks of the scenario that answer shows: a task it created, and the state a cancel
    /// moved one to.
    fn derive(
        &mut self,
        command: &str,
        outcome: &Outcome,
        input: &Value,
        carried: Carried,
    ) -> Result<Vec<Event>, TargetError> {
        let doing = format!("{command} took `{}`", outcome.name);
        if outcome.publishes && outcome.events.is_empty() {
            return Err(TargetError::Unsupported(format!(
                "{doing}, which publishes events the generated contract names none of"
            )));
        }
        let (task, first, rest) = match carried {
            Carried::Nothing => (None, None, None),
            Carried::Task(task) => (Some(task), None, None),
            Carried::Message { first, rest } => (first.task.clone(), Some(first), Some(rest)),
            Carried::Subscription { task, rest } => (Some(task), None, Some(rest)),
        };
        let earlier = task
            .as_ref()
            .is_some_and(|task| !self.answered.insert(task.id.0.clone()));
        let declares = |event: &str| outcome.events.iter().any(|declared| declared == event);
        if let Some(task) = &task {
            let (id, state) = (task.id.0.as_str(), task.status.state);
            if declares(TASK_STATUS_UPDATED) {
                if let Some(created) = self.created.iter_mut().find(|created| created.id == id) {
                    created.state = state;
                }
            } else if let Some(left) = self
                .left_in(id)
                .or_else(|| declares(TASK_CREATED).then_some(INITIAL))
                && left != state
            {
                return Err(moved(&doing, id, state, left));
            }
        }
        let mut events = Vec::new();
        for event in &outcome.events {
            events.push(match event.as_str() {
                TASK_CREATED => self.created(&doing, input, task.as_ref(), earlier)?,
                MESSAGE_STREAM_RESPONSE => match &first {
                    Some(first) => {
                        event_of(&SystemEvent::MessageStreamResponse((**first).clone()))?
                    }
                    None => return Err(not_carried(&doing, event)),
                },
                TASK_STATUS_UPDATED => match &task {
                    Some(task) => status_of(&doing, task)?,
                    None => return Err(not_carried(&doing, event)),
                },
                _ => return Err(not_carried(&doing, event)),
            });
        }
        let Some(rest) = rest else {
            return Ok(events);
        };
        let rest = rest.map_err(|error| TargetError::Failed(format!("{command}: {error}")))?;
        let opened = task
            .as_ref()
            .map(|task| (task.id.0.as_str(), task.status.state));
        for element in rest {
            events.push(match element {
                Element::Status(update) => {
                    let id = update.task_id.0.as_str();
                    let left = self.left_in(id).or_else(|| {
                        opened
                            .filter(|(opened, _)| *opened == id)
                            .map(|(_, state)| state)
                    });
                    if let Some(left) = left
                        && left != update.status.state
                    {
                        return Err(moved(&doing, id, update.status.state, left));
                    }
                    self.answered.insert(update.task_id.0.clone());
                    event_of(&SystemEvent::TaskStatusUpdated(tasks::TaskStatusUpdated {
                        task_id: update.task_id,
                        context_id: update.context_id,
                        status: update.status,
                    }))?
                }
                Element::Artifact(update) => {
                    self.answered.insert(update.task_id.0.clone());
                    event_of(&SystemEvent::TaskArtifactUpdated(
                        tasks::TaskArtifactUpdated {
                            task_id: update.task_id,
                            context_id: update.context_id,
                            artifact: update.artifact,
                            append: update.append,
                            last_chunk: update.last_chunk,
                        },
                    ))?
                }
                Element::Other(kind) => {
                    return Err(TargetError::Unsupported(format!(
                        "{doing}, and its stream carried {kind} after its first element, which no \
                         event of ess/domains/tasks.yaml stands for"
                    )));
                }
            });
        }
        Ok(events)
    }

    /// `a2a.tasks.TaskCreated` of `task`, a message's answer, which the scenario then holds with
    /// the request's tenant; none where an `earlier` answer of the run already showed the task.
    fn created(
        &mut self,
        doing: &str,
        input: &Value,
        task: Option<&TaskSnapshot>,
        earlier: bool,
    ) -> Result<Event, TargetError> {
        let Some(task) = task else {
            return Err(TargetError::Unsupported(format!(
                "{doing}, and its answer holds no task to read {TASK_CREATED} from"
            )));
        };
        if earlier {
            return Err(TargetError::Unsupported(format!(
                "{doing}, and its answer holds task `{}`, which the endpoint had already answered \
                 in this run: it carries no new task to read {TASK_CREATED} from",
                task.id.0
            )));
        }
        self.created.push(Created {
            id: task.id.0.clone(),
            tenant: text(input.member("tenant")),
            state: task.status.state,
        });
        event_of(&SystemEvent::TaskCreated(tasks::TaskCreated {
            task_id: task.id.clone(),
        }))
    }

    /// The rows of [`ALL_TASKS`]: each task the scenario created, read back with GetTask under the
    /// tenant it was created under. A task the endpoint answers `TaskNotFound` for has no row.
    fn all_tasks(&self) -> Result<Vec<Value>, TargetError> {
        let mut rows = Vec::new();
        for created in &self.created {
            let request = tasks::GetTask {
                tenant: created.tenant.clone(),
                id: TaskId(created.id.clone()),
                history_length: None,
            };
            let task = match self.client.get_task(&request) {
                Ok(task) => task,
                Err(Error::Status {
                    http: 404,
                    reason: Some(Reason::TaskNotFound),
                    ..
                }) => continue,
                Err(error) => {
                    return Err(unread(
                        ALL_TASKS,
                        &format!("GetTask of task `{}`", created.id),
                        &error,
                    ));
                }
            };
            unmoved(ALL_TASKS, created, &task)?;
            let context_id = context_of(ALL_TASKS, created, &task)?;
            let row = tasks::AllTasks {
                id: task.id,
                context_id,
                tenant: created.tenant.clone(),
                state: lifecycle(task.status.state),
                status_message: task.status.message,
                status_timestamp: task.status.timestamp,
                artifacts: task.artifacts,
                history: task.history,
                metadata: task.metadata,
            };
            let mut text = String::new();
            wire::encode_view_a2a_tasks_all_tasks(&row, &mut text);
            rows.push(row_of(ALL_TASKS, &text)?);
        }
        Ok(rows)
    }

    /// The rows of [`TASKS`]: ListTasks under the tenant `params` names, every page, keeping the
    /// tasks the scenario created.
    fn tasks(&self, params: &Value) -> Result<Vec<Value>, TargetError> {
        let params = Naming::contract().input_to_wire(TASKS, params);
        let (context_id, status, status_timestamp_after, tenant) =
            wire::decode_params_a2a_tasks_tasks(&params, "params").map_err(|error| {
                TargetError::Failed(format!(
                    "the parameters of {TASKS} are not its wire parameters: {error}"
                ))
            })?;
        let mut request = ListTasksRequest {
            tenant,
            context_id,
            status: status.map(wire_state),
            page_size: None,
            page_token: None,
            history_length: None,
            status_timestamp_after,
            include_artifacts: Some(true),
        };
        let mut tokens: Vec<String> = Vec::new();
        let mut rows = Vec::new();
        loop {
            let page = self
                .client
                .list_tasks(&request)
                .map_err(|error| unread(TASKS, "ListTasks", &error))?;
            for task in page.tasks {
                let Some(created) = self.created.iter().find(|created| created.id == task.id.0)
                else {
                    continue;
                };
                unmoved(TASKS, created, &task)?;
                let context_id = context_of(TASKS, created, &task)?;
                let row = tasks::Tasks {
                    id: task.id,
                    context_id,
                    state: lifecycle(task.status.state),
                    status_message: task.status.message,
                    status_timestamp: task.status.timestamp,
                    artifacts: task.artifacts,
                    history: task.history,
                    metadata: task.metadata,
                };
                let mut text = String::new();
                wire::encode_view_a2a_tasks_tasks(&row, &mut text);
                rows.push(row_of(TASKS, &text)?);
            }
            let token = page.next_page_token;
            if token.is_empty() {
                return Ok(rows);
            }
            if tokens.contains(&token) || tokens.len() >= MOST_PAGES {
                return Err(TargetError::Failed(format!(
                    "reading {TASKS}: ListTasks answered the page token `{token}` after {} pages",
                    tokens.len() + 1
                )));
            }
            tokens.push(token.clone());
            request.page_token = Some(token);
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
                    let carried = answered
                        .task
                        .clone()
                        .map_or(Carried::Nothing, Carried::Task);
                    Accepted::of(&answered, wire::encode_response_a2a_tasks_send_message)
                        .named(outcome)
                        .carrying(carried)
                })
            }
            "a2a.tasks.SendStreamingMessage" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_tasks_send_streaming_message(input, "input"),
                )?;
                self.streaming()
                    .send_streaming_message(&request)
                    .and_then(|mut events| {
                        let first = first(&mut events)?;
                        let rest = follow(&mut events, |element: tasks::MessageStreamResponse| {
                            Element::of(
                                element.task.is_some(),
                                element.message.is_some(),
                                element.status_update,
                                element.artifact_update,
                            )
                        });
                        Ok((first, rest))
                    })
                    .map(|(first, rest)| {
                        let outcome = sent(&request.message, first.message.is_some());
                        Accepted::of(
                            &tasks::SendStreamingMessageResponse {
                                task: first.task.clone(),
                                reply: first.message.clone(),
                            },
                            wire::encode_response_a2a_tasks_send_streaming_message,
                        )
                        .named(outcome)
                        .carrying(Carried::Message {
                            first: Box::new(first),
                            rest,
                        })
                    })
            }
            "a2a.tasks.GetTask" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_tasks_get_task(input, "input"),
                )?;
                client.get_task(&request).map(|task| {
                    Accepted::of(
                        &tasks::GetTaskResponse { task: task.clone() },
                        wire::encode_response_a2a_tasks_get_task,
                    )
                    .carrying(Carried::Task(task))
                })
            }
            "a2a.tasks.CancelTask" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_tasks_cancel_task(input, "input"),
                )?;
                client.cancel_task(&request).map(|task| {
                    Accepted::of(
                        &tasks::CancelTaskResponse { task: task.clone() },
                        wire::encode_response_a2a_tasks_cancel_task,
                    )
                    .carrying(Carried::Task(task))
                })
            }
            "a2a.tasks.SubscribeToTask" => {
                let request = decoded(
                    command,
                    wire::decode_command_a2a_tasks_subscribe_to_task(input, "input"),
                )?;
                self.streaming()
                    .subscribe_to_task(&request)
                    .and_then(|mut events| {
                        let task = first(&mut events)?.task.ok_or_else(|| {
                            Error::Protocol(
                                "the first event of a SubscribeToTask stream is not the task \
                                 (A2A v1.0.1 section 3.1.6)"
                                    .to_owned(),
                            )
                        })?;
                        let rest =
                            follow(&mut events, |element: tasks::TaskSubscriptionResponse| {
                                Element::of(
                                    element.task.is_some(),
                                    element.message.is_some(),
                                    element.status_update,
                                    element.artifact_update,
                                )
                            });
                        Ok((task, rest))
                    })
                    .map(|(task, rest)| {
                        Accepted::of(
                            &tasks::SubscribeToTaskResponse { task: task.clone() },
                            wire::encode_response_a2a_tasks_subscribe_to_task,
                        )
                        .carrying(Carried::Subscription { task, rest })
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
                        carried: Carried::Nothing,
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
    /// What the answer carried that the command's events and the scenario's tasks are read from.
    carried: Carried,
}

impl Accepted {
    fn of<T>(response: &T, encode: fn(&T, &mut String)) -> Self {
        let mut text = String::new();
        encode(response, &mut text);
        Self {
            response: Some(text),
            outcome: None,
            carried: Carried::Nothing,
        }
    }

    fn named(self, outcome: &'static str) -> Self {
        Self {
            outcome: Some(outcome),
            ..self
        }
    }

    fn carrying(self, carried: Carried) -> Self {
        Self { carried, ..self }
    }
}

/// What a success answer carried of tasks and their events.
enum Carried {
    /// No task: a direct message, a configuration, a card, an empty answer.
    Nothing,
    /// A task: SendMessage, GetTask, CancelTask.
    Task(TaskSnapshot),
    /// A message stream: its first element, and the elements after it.
    Message {
        first: Box<tasks::MessageStreamResponse>,
        rest: Followed,
    },
    /// A task's stream: the task it opens with, and the elements after it.
    Subscription { task: TaskSnapshot, rest: Followed },
}

/// The elements a stream carried after its first, until it closed or went quiet; what it answered
/// instead where an element was not a stream response.
type Followed = Result<Vec<Element>, String>;

/// One element of a stream after its first.
enum Element {
    Status(tasks::TaskStatusUpdateEvent),
    Artifact(tasks::TaskArtifactUpdateEvent),
    /// An element that is neither, named for a reason.
    Other(&'static str),
}

impl Element {
    /// The element whose members are these: one of a task, a message, a status update and an
    /// artifact update (a2a.proto `StreamResponse.payload`, a oneof).
    fn of(
        task: bool,
        message: bool,
        status: Option<tasks::TaskStatusUpdateEvent>,
        artifact: Option<tasks::TaskArtifactUpdateEvent>,
    ) -> Self {
        match (task, message, status, artifact) {
            (false, false, Some(status), None) => Self::Status(status),
            (false, false, None, Some(artifact)) => Self::Artifact(artifact),
            (true, false, None, None) => Self::Other("a task"),
            (false, true, None, None) => Self::Other("a message"),
            (false, false, None, None) => Self::Other("an element with no member"),
            _ => Self::Other("an element with several members"),
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

/// The first event of a stream.
fn first<T>(events: &mut Events<T>) -> Result<T, Error> {
    events.next().unwrap_or_else(|| {
        Err(Error::Protocol(
            "the stream ended before its first event".to_owned(),
        ))
    })
}

/// The elements of `events` until the stream closes, or none arrives within [`STREAM_QUIET`], or
/// [`MOST_ELEMENTS`] have; the stream is closed when it is dropped.
fn follow<T>(events: &mut Events<T>, element: fn(T) -> Element) -> Followed {
    let mut followed = Vec::new();
    while followed.len() < MOST_ELEMENTS {
        match events.next() {
            None => break,
            Some(Ok(item)) => followed.push(element(item)),
            Some(Err(Error::Transport(error)))
                if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
            {
                break;
            }
            Some(Err(error)) => {
                return Err(format!("the stream after its first element: {error}"));
            }
        }
    }
    Ok(followed)
}

fn decoded<T>(command: &str, decoded: Result<T, json::DecodeError>) -> Result<T, TargetError> {
    decoded.map_err(|error| {
        TargetError::Failed(format!(
            "the scenario's input of {command} is not the command's wire input: {error}"
        ))
    })
}

/// The text `value` holds, where it is one.
fn text(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::Text(text)) => Some(text.clone()),
        _ => None,
    }
}

/// `event`, its payload named as the generated contract names it.
fn event_of(event: &SystemEvent) -> Result<Event, TargetError> {
    let name = event.name();
    let envelope = json::parse_written(&wire::encode_system_event(event))
        .map_err(|error| TargetError::Failed(format!("{name} re-encoded is not JSON: {error}")))?;
    let payload = envelope.member("payload").cloned().unwrap_or(Value::Null);
    Ok(Event {
        name: name.to_owned(),
        payload: Naming::contract().event_to_ess(name, &payload),
    })
}

/// `a2a.tasks.TaskStatusUpdated` of the task a cancel answered.
fn status_of(doing: &str, task: &TaskSnapshot) -> Result<Event, TargetError> {
    let Some(context_id) = task.context_id.clone() else {
        return Err(TargetError::Unsupported(format!(
            "{doing}, and its answer holds task `{}` without the contextId \
             {TASK_STATUS_UPDATED} carries",
            task.id.0
        )));
    };
    event_of(&SystemEvent::TaskStatusUpdated(tasks::TaskStatusUpdated {
        task_id: task.id.clone(),
        context_id,
        status: task.status.clone(),
    }))
}

fn not_carried(doing: &str, event: &str) -> TargetError {
    TargetError::Unsupported(format!(
        "{doing}, which publishes {event}, and its answer over the A2A HTTP+JSON binding does not \
         carry it"
    ))
}

/// Why an answer showing task `id` in `answered`, where the scenario's commands left it in
/// `left`, cannot be held to the scenario.
fn moved(doing: &str, id: &str, answered: WireTaskState, left: WireTaskState) -> TargetError {
    TargetError::Unsupported(format!(
        "{doing}, and the endpoint answered task `{id}` in {}, where the scenario's commands left \
         it in {}: the agent moved it by steps of its own (a2a.tasks.StartWorking, \
         a2a.tasks.RequireInput, …), which the wire neither arranges nor tells apart from what \
         the scenario ran",
        spelled(answered),
        spelled(left)
    ))
}

/// The task a view read answered for `created`, where its state is the one the scenario's
/// commands left it in.
fn unmoved(view: &str, created: &Created, task: &TaskSnapshot) -> Result<(), TargetError> {
    if task.status.state == created.state {
        return Ok(());
    }
    Err(moved(
        &format!("reading {view}"),
        &created.id,
        task.status.state,
        created.state,
    ))
}

/// The context of a row: the task's as read. The scenario's request never stands in for it: a row
/// whose task was answered without one claims nothing the endpoint answered.
fn context_of(
    view: &str,
    created: &Created,
    task: &TaskSnapshot,
) -> Result<ContextId, TargetError> {
    task.context_id.clone().ok_or_else(|| {
        TargetError::Unsupported(format!(
            "reading {view}: task `{}` was answered without a contextId, which every row \
                 carries",
            created.id
        ))
    })
}

/// Why `view` could not be read: what `operation` answered.
fn unread(view: &str, operation: &str, error: &Error) -> TargetError {
    match error {
        Error::Status { .. } => TargetError::Unsupported(format!(
            "reading {view}: {operation} answered {error}, so the wire cannot observe its rows"
        )),
        other => TargetError::Failed(format!("reading {view}: {operation}: {other}")),
    }
}

/// A row of `view`, from its wire encoding, named as the generated contract names it.
fn row_of(view: &str, encoded: &str) -> Result<Value, TargetError> {
    let row = json::parse_written(encoded).map_err(|error| {
        TargetError::Failed(format!("a row of {view} re-encoded is not JSON: {error}"))
    })?;
    Ok(Naming::contract().row_to_ess(view, &row))
}

/// The lifecycle state a wire state is (`ess/domains/tasks.yaml`, a2a.tasks.Task; the
/// specification leaves the map from the wire's `TASK_STATE_*` UNMAPPED).
fn lifecycle(state: WireTaskState) -> TaskState {
    match state {
        WireTaskState::TASKSTATESUBMITTED => TaskState::Submitted,
        WireTaskState::TASKSTATEWORKING => TaskState::Working,
        WireTaskState::TASKSTATECOMPLETED => TaskState::Completed,
        WireTaskState::TASKSTATEFAILED => TaskState::Failed,
        WireTaskState::TASKSTATECANCELED => TaskState::Canceled,
        WireTaskState::TASKSTATEINPUTREQUIRED => TaskState::InputRequired,
        WireTaskState::TASKSTATEREJECTED => TaskState::Rejected,
        WireTaskState::TASKSTATEAUTHREQUIRED => TaskState::AuthRequired,
    }
}

/// The wire state a lifecycle state is spelled as, the inverse of [`lifecycle`].
fn wire_state(state: TaskState) -> WireTaskState {
    match state {
        TaskState::Submitted => WireTaskState::TASKSTATESUBMITTED,
        TaskState::Working => WireTaskState::TASKSTATEWORKING,
        TaskState::Completed => WireTaskState::TASKSTATECOMPLETED,
        TaskState::Failed => WireTaskState::TASKSTATEFAILED,
        TaskState::Canceled => WireTaskState::TASKSTATECANCELED,
        TaskState::InputRequired => WireTaskState::TASKSTATEINPUTREQUIRED,
        TaskState::Rejected => WireTaskState::TASKSTATEREJECTED,
        TaskState::AuthRequired => WireTaskState::TASKSTATEAUTHREQUIRED,
    }
}

/// `TASK_STATE_WORKING`.
fn spelled(state: WireTaskState) -> String {
    let mut text = String::new();
    wire::encode_a2a_tasks_task_state_2(&state, &mut text);
    text.trim_matches('"').to_owned()
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
        self.pending = None;
        self.recorded = false;
        self.created.clear();
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
        self.pending = None;
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
        let across = self.across(command, &input);
        let called = self.call(command, &input)?;
        if across
            && matches!(
                &called,
                Err(Error::Status {
                    http: 404,
                    reason: Some(Reason::TaskNotFound),
                    ..
                })
            )
        {
            return Err(TargetError::Unsupported(across_tenants(command)));
        }
        self.read(command, &input, expected.as_deref(), called)
    }

    fn query(
        &mut self,
        view: &str,
        reader: Reader<'_>,
        params: &Value,
    ) -> Result<Vec<Value>, TargetError> {
        if view != ALL_TASKS && (view != TASKS || !carried(view)) {
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
        if self.created.is_empty() {
            return Err(TargetError::Unsupported(format!(
                "reading {view}: the scenario created no task, and the wire's rows are the tasks \
                 the scenario created, so the read observes nothing the endpoint answered"
            )));
        }
        if view == ALL_TASKS {
            self.all_tasks()
        } else {
            self.tasks(params)
        }
    }

    fn force(&mut self, command: &str, outcome: &str) -> Result<(), TargetError> {
        Err(TargetError::Unsupported(format!(
            "forcing {command} to `{outcome}`: the specification decides it outside the request, \
             and the wire has no way to arrange those circumstances"
        )))
    }

    fn published(&mut self) -> Result<Vec<Event>, TargetError> {
        self.pending.take().unwrap_or(Ok(Vec::new()))
    }

    fn await_outcome(&mut self, command: &str, _instance: &Value) -> Result<Awaited, TargetError> {
        Err(TargetError::Unsupported(format!(
            "{command} has no A2A operation: the agent issues it inside the server, so the wire \
             cannot await its outcome"
        )))
    }

    fn end(&mut self) -> Result<(), TargetError> {
        self.pending = None;
        Ok(())
    }
}
