//! The ports the generated behaviours run over, and the twelve obligations the generated crate
//! leaves open (`generated/rust/a2a/PLAN.md`).
//!
//! Storage is kept here, keyed as the generated in-memory store keys it, and read in place where
//! an answer needs part of a task; the clock stamps every status, once per command; the card
//! decides versions, extensions and capabilities; the [`Agent`] decides what the model leaves
//! externally decided. The steps an agent's [`Turn`] asks for are queued here as [`Work`] and
//! issued by the server loop as system commands (`crate::steps`), so the moves the model allows
//! are the only ones made and every event they publish lands on the system's log.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use a2a::behaviour::{Context, PushNotificationConfigStorage, TaskStorage};
use a2a::discovery::obligations::GetExtendedAgentCardBehavior;
use a2a::discovery::{
    AgentCard, ExtendedAgentCardNotConfigured, GetExtendedAgentCard, GetExtendedAgentCardOutcome,
    GetExtendedAgentCardResponse,
};
use a2a::obligation::UnmetObligation;
use a2a::primitives::Timestamp;
use a2a::tasks::obligations::{
    CancelTaskBehavior, CreateTaskPushNotificationConfigBehavior,
    DeleteTaskPushNotificationConfigBehavior, GetTaskBehavior,
    GetTaskPushNotificationConfigBehavior, ListTaskPushNotificationConfigsBehavior,
    SendMessageBehavior, SendStreamingMessageBehavior, SubscribeToTaskBehavior,
    TaskPushNotificationConfigsQuery, TasksQuery,
};
use a2a::tasks::{
    AnyPushNotificationConfig, AnyTask, Artifact, CancelTask, CancelTaskOutcome,
    CancelTaskResponse, ContentTypeNotSupported, ContextId, ContextMismatch, ContextNotAccepted,
    CreateTaskPushNotificationConfig, CreateTaskPushNotificationConfigOutcome,
    CreateTaskPushNotificationConfigResponse, DeleteTaskPushNotificationConfig,
    DeleteTaskPushNotificationConfigOutcome, ExtensionSupportRequired, GetTask, GetTaskOutcome,
    GetTaskPushNotificationConfig, GetTaskPushNotificationConfigOutcome,
    GetTaskPushNotificationConfigResponse, GetTaskResponse, ListTaskPushNotificationConfigs,
    ListTaskPushNotificationConfigsOutcome, ListTaskPushNotificationConfigsResponse, Message,
    MessageStreamResponse, PushNotificationConfig, PushNotificationConfigCreated,
    PushNotificationConfigData, PushNotificationConfigDeleted, PushNotificationConfigId,
    PushNotificationConfigRow, PushNotificationConfigSnapshot, PushNotificationNotSupported,
    ResourceExhausted, SendMessage, SendMessageConfiguration, SendMessageOutcome,
    SendMessageResponse, SendStreamingMessage, SendStreamingMessageOutcome,
    SendStreamingMessageResponse, SubscribeToTask, SubscribeToTaskOutcome, SubscribeToTaskResponse,
    Task, TaskCreated, TaskData, TaskEntitySnapshot, TaskId, TaskNotCancelable, TaskNotFound,
    TaskPushNotificationConfig, TaskPushNotificationConfigs, TaskSnapshot, TaskState, TaskStatus,
    TaskStatusUpdated, Tasks, UnsupportedOperation, VersionNotSupported, WireTaskState,
};

use crate::agent::{Agent, Refusal, Step, Turn};
use crate::paging::{self, List};
use crate::{Limits, Settings};

/// The protocol version a request that names none speaks (A2A v1.0.1 section 3.6.2).
const DEFAULT_VERSION: &str = "0.3";

/// The server's state behind the generated behaviours: one handle, cloned wherever a port is
/// needed, all clones sharing it.
#[derive(Clone)]
pub(crate) struct ServerPorts(Rc<Shared>);

struct Shared {
    tasks: RefCell<TaskStore>,
    configs: RefCell<BTreeMap<String, PushNotificationConfigSnapshot>>,
    limits: Limits,
    card: AgentCard,
    extended_card: Option<AgentCard>,
    agent: RefCell<Box<dyn Agent>>,
    version: RefCell<Option<String>>,
    extensions: RefCell<Vec<String>>,
    pending: RefCell<Vec<Work>>,
    last_stamp: Cell<i128>,
    command_stamp: RefCell<Option<Timestamp>>,
}

/// The steps an agent asked for on one task, for the server loop to issue as system commands.
pub(crate) struct Work {
    /// The task the steps move.
    pub(crate) task: TaskId,
    /// The steps, in order.
    pub(crate) steps: Vec<Step>,
    /// Whether the answer waits for the steps: a `SendMessage` without `returnImmediately`.
    pub(crate) awaited: bool,
    /// The `historyLength` the request asked its answer to keep.
    pub(crate) history_length: Option<i64>,
}

/// The stored tasks by id, in the order of the generated in-memory store, and how many each
/// tenant holds.
// ESS gap (ess 0.67.0): the generated `MemoryTaskStorage` answers every read with a copy of the
// whole task (`get`) or of every task (`list`), and no port reads part of one, so a task with a
// long history is copied whole for each answer about it. The store is kept here so that an
// answer copies only what it carries (`snapshot`, `state_of`, `update`).
#[derive(Default)]
struct TaskStore {
    held: BTreeMap<String, TaskEntitySnapshot>,
    per_tenant: HashMap<Option<String>, usize>,
}

impl TaskStore {
    fn put(&mut self, snapshot: TaskEntitySnapshot) {
        let tenant = snapshot.data.tenant.clone();
        if let Some(before) = self.held.insert(snapshot.data.id.0.clone(), snapshot) {
            self.forget(before.data.tenant);
        }
        *self.per_tenant.entry(tenant).or_default() += 1;
    }

    fn delete(&mut self, id: &TaskId) {
        if let Some(before) = self.held.remove(&id.0) {
            self.forget(before.data.tenant);
        }
    }

    fn forget(&mut self, tenant: Option<String>) {
        if let Some(count) = self.per_tenant.get_mut(&tenant) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.per_tenant.remove(&tenant);
            }
        }
    }
}

/// Which service parameter of the request the card does not admit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mismatch {
    /// `A2A-Version` names a version no interface of the card declares.
    Version,
    /// The card requires an extension `A2A-Extensions` does not name.
    Extension,
}

/// What a message to the agent comes to, short of the version, extension and capability checks.
enum Taken {
    NotFound,
    Terminal,
    ContextMismatch,
    ContextNotAccepted,
    ContentType,
    /// The tenant holds [`Limits::max_tasks`] tasks.
    TaskLimit,
    /// The task's history holds [`Limits::max_history`] messages.
    HistoryLimit,
    Direct(Message),
    Created(TaskSnapshot, TaskId),
    Continued(TaskSnapshot),
}

/// Why a message cannot continue the task it names.
#[derive(Debug, Clone, Copy)]
enum Barred {
    NotFound,
    Terminal,
    ContextMismatch,
}

impl From<Barred> for Taken {
    fn from(barred: Barred) -> Self {
        match barred {
            Barred::NotFound => Self::NotFound,
            Barred::Terminal => Self::Terminal,
            Barred::ContextMismatch => Self::ContextMismatch,
        }
    }
}

impl ServerPorts {
    pub(crate) fn new(settings: Settings, agent: Box<dyn Agent>, limits: Limits) -> Self {
        Self(Rc::new(Shared {
            tasks: RefCell::new(TaskStore::default()),
            configs: RefCell::new(BTreeMap::new()),
            limits,
            card: settings.card,
            extended_card: settings.extended_card,
            agent: RefCell::new(agent),
            version: RefCell::new(None),
            extensions: RefCell::new(Vec::new()),
            pending: RefCell::new(Vec::new()),
            last_stamp: Cell::new(0),
            command_stamp: RefCell::new(None),
        }))
    }

    /// The public agent card.
    pub(crate) fn card(&self) -> &AgentCard {
        &self.0.card
    }

    /// Takes the service parameters of the request about to run: its `A2A-Version`, `None`
    /// where it names none, and every extension its `A2A-Extensions` names.
    pub(crate) fn negotiate(&self, version: Option<String>, extensions: Vec<String>) {
        *self.0.version.borrow_mut() = version;
        *self.0.extensions.borrow_mut() = extensions;
    }

    /// The service parameter of the current request the card does not admit, if one.
    pub(crate) fn mismatch(&self) -> Option<Mismatch> {
        let asked = self
            .0
            .version
            .borrow()
            .clone()
            .filter(|version| !version.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_VERSION.to_owned());
        let asked = major_minor(&asked);
        let spoken = self
            .0
            .card
            .supported_interfaces
            .iter()
            .any(|interface| major_minor(&interface.protocol_version) == asked);
        if !spoken {
            return Some(Mismatch::Version);
        }
        let declared = self.0.extensions.borrow();
        let missing = self
            .0
            .card
            .capabilities
            .extensions
            .iter()
            .flatten()
            .filter(|extension| extension.required == Some(true))
            .filter_map(|extension| extension.uri.as_ref())
            .any(|uri| !declared.iter().any(|named| named == uri));
        missing.then_some(Mismatch::Extension)
    }

    fn streams(&self) -> bool {
        self.0.card.capabilities.streaming == Some(true)
    }

    fn pushes(&self) -> bool {
        self.0.card.capabilities.push_notifications == Some(true)
    }

    /// Whether `GetExtendedAgentCard` answers an extended card: the card declares one and one is
    /// configured.
    pub(crate) fn serves_extended_card(&self) -> bool {
        self.0.card.capabilities.extended_agent_card == Some(true) && self.0.extended_card.is_some()
    }

    /// A copy of the stored task with this id, whole.
    fn task(&self, id: &TaskId) -> Option<TaskEntitySnapshot> {
        self.0.tasks.borrow().held.get(&id.0).cloned()
    }

    /// What `read` answers of the stored task with this id, read in place.
    fn read<R>(&self, id: &TaskId, read: impl FnOnce(&TaskEntitySnapshot) -> R) -> Option<R> {
        self.0.tasks.borrow().held.get(&id.0).map(read)
    }

    /// Changes the stored task with this id in place; `false` where there is none. `change` keeps
    /// its id and tenant.
    fn update(&self, id: &TaskId, change: impl FnOnce(&mut TaskEntitySnapshot)) -> bool {
        match self.0.tasks.borrow_mut().held.get_mut(&id.0) {
            Some(held) => {
                change(held);
                true
            }
            None => false,
        }
    }

    /// Whether a task with this id is stored where a request under `tenant` may reach it: not
    /// where it is of another tenant (A2A v1.0.1 section 13.1).
    pub(crate) fn reachable(&self, id: &TaskId, tenant: Option<&str>) -> bool {
        self.read(id, |held| same_tenant(held.data.tenant.as_deref(), tenant))
            .unwrap_or(false)
    }

    /// The lifecycle state and context of the stored task with this id.
    pub(crate) fn state_of(&self, id: &TaskId) -> Option<(TaskState, ContextId)> {
        self.read(id, |held| (held.state, held.data.context_id.clone()))
    }

    /// The stored task with this id as the wire carries it, with only its latest
    /// `history_length` messages where a length is asked for and its artifacts where
    /// `artifacts`; copies only what it answers.
    pub(crate) fn snapshot(
        &self,
        id: &TaskId,
        history_length: Option<i64>,
        artifacts: bool,
    ) -> Option<TaskSnapshot> {
        self.read(id, |held| wire_of(held, history_length, artifacts))
    }

    fn store(&self, snapshot: TaskEntitySnapshot) {
        self.0.tasks.borrow_mut().put(snapshot);
    }

    /// The steps every message so far left for the server loop to issue, taken.
    pub(crate) fn take_work(&self) -> Vec<Work> {
        std::mem::take(&mut *self.0.pending.borrow_mut())
    }

    /// Starts the next command: the status timestamp it asks for is a new one.
    pub(crate) fn next_command(&self) {
        *self.0.command_stamp.borrow_mut() = None;
    }

    /// The status timestamp of the command running: RFC 3339 in UTC with milliseconds (A2A
    /// v1.0.1 section 5.6.1), later than the one of every command before it. Every call within one
    /// command answers the same, so the status a command stores and the event it publishes carry
    /// one instant.
    // ESS gap (ess 0.67.0): a generated behaviour asks the clock once for the status it stores
    // and once for the event it publishes, and the event's `{subject:}` source reads the value
    // before the outcome, so nothing in the model makes the two one instant; this cache does.
    fn stamp(&self) -> Option<Timestamp> {
        if let Some(stamp) = self.0.command_stamp.borrow().clone() {
            return Some(stamp);
        }
        let now = time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000;
        let next = now.max(self.0.last_stamp.get() + 1);
        self.0.last_stamp.set(next);
        let at = time::OffsetDateTime::from_unix_timestamp_nanos(next * 1_000_000).ok()?;
        let stamp = Timestamp(format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            at.year(),
            u8::from(at.month()),
            at.day(),
            at.hour(),
            at.minute(),
            at.second(),
            at.millisecond()
        ));
        *self.0.command_stamp.borrow_mut() = Some(stamp.clone());
        Some(stamp)
    }

    /// The message `message` comes to, the version, extension and capability checks passed. A
    /// task's steps are queued as [`Work`]; `streamed` says the answer is a stream, which never
    /// waits for them.
    fn take(
        &self,
        message: Message,
        configuration: Option<&SendMessageConfiguration>,
        streamed: bool,
        tenant: Option<String>,
    ) -> Taken {
        let history_length = configuration.and_then(|configuration| configuration.history_length);
        let awaited = !streamed
            && configuration.and_then(|configuration| configuration.return_immediately)
                != Some(true);
        // The task a message continues: its id, context and history length, and the view the
        // agent is handed.
        let held = match &message.task_id {
            Some(id) => {
                // ESS gap (ess 0.67.0): the task lookup of a message is `external:` in
                // `ess/domains/tasks.yaml` (SendMessage, `message.taskId` is no top-level input),
                // so the tenant rule of GetTask `other-tenant` is kept here by hand.
                let checked = self.read(id, |held| {
                    if !same_tenant(held.data.tenant.as_deref(), tenant.as_deref()) {
                        return Err(Barred::NotFound);
                    }
                    if terminal(held.state) {
                        return Err(Barred::Terminal);
                    }
                    if message
                        .context_id
                        .as_ref()
                        .is_some_and(|context| *context != held.data.context_id)
                    {
                        return Err(Barred::ContextMismatch);
                    }
                    let length = held.data.history.as_ref().map_or(0, Vec::len);
                    Ok((
                        held.data.id.clone(),
                        held.data.context_id.clone(),
                        length,
                        wire_of(held, None, true),
                    ))
                });
                match checked {
                    None => return Taken::NotFound,
                    Some(Err(barred)) => return barred.into(),
                    Some(Ok(held)) => Some(held),
                }
            }
            None => None,
        };
        let (task_id, context_id) = match &held {
            Some((id, context, _, _)) => (id.clone(), context.clone()),
            None => (
                TaskId(uuid::Uuid::new_v4().to_string()),
                message
                    .context_id
                    .clone()
                    .unwrap_or_else(|| ContextId(uuid::Uuid::new_v4().to_string())),
            ),
        };
        let mut kept = message.clone();
        kept.task_id = Some(task_id.clone());
        kept.context_id = Some(context_id.clone());
        let (history, view) = match held {
            Some((_, _, length, mut view)) => {
                view.history.get_or_insert_with(Vec::new).push(kept.clone());
                (Some(length), Some(view))
            }
            None => (None, None),
        };
        let turn = self.0.agent.borrow_mut().turn(&message, view.as_ref());
        drop(view);
        match turn {
            Turn::Refuse(Refusal::ContentType) => Taken::ContentType,
            Turn::Refuse(Refusal::Context) => Taken::ContextNotAccepted,
            Turn::Reply(mut reply) => {
                if history.is_some() {
                    reply.task_id = Some(task_id);
                    reply.context_id = Some(context_id);
                } else {
                    reply.context_id = reply.context_id.or(message.context_id);
                }
                Taken::Direct(reply)
            }
            Turn::Steps(steps) => {
                let limits = self.0.limits;
                match history {
                    Some(length) if length >= limits.max_history => return Taken::HistoryLimit,
                    Some(_) => {
                        self.update(&task_id, |held| push_history(&mut held.data, kept));
                    }
                    None if self.tasks_held(tenant.as_deref()) >= limits.max_tasks => {
                        return Taken::TaskLimit;
                    }
                    None => {
                        let data = TaskData {
                            id: task_id.clone(),
                            context_id,
                            tenant,
                            status_message: None,
                            status_timestamp: self.stamp(),
                            artifacts: None,
                            history: Some(vec![kept]),
                            metadata: None,
                        };
                        self.store(AnyTask::Submitted(Task::new(data)).snapshot());
                    }
                }
                self.0.pending.borrow_mut().push(Work {
                    task: task_id.clone(),
                    steps,
                    awaited,
                    history_length,
                });
                let Some(task) = self.snapshot(&task_id, history_length, true) else {
                    return Taken::NotFound;
                };
                if history.is_none() {
                    Taken::Created(task, task_id)
                } else {
                    Taken::Continued(task)
                }
            }
        }
    }

    /// How many tasks the tenant holds, every task without a tenant counting as one tenant's.
    fn tasks_held(&self, tenant: Option<&str>) -> usize {
        self.0
            .tasks
            .borrow()
            .per_tenant
            .get(&tenant.map(str::to_owned))
            .copied()
            .unwrap_or(0)
    }

    /// Keeps an emitted artifact on its task: added, replacing the one of the same id, or with
    /// its parts appended to it. The generated `UpdateArtifact` publishes the artifact but stores
    /// none (`ess/domains/tasks.yaml`, its `updated` outcome sets nothing).
    // ESS gap (ess 0.67.0): `sets:` has no append to, or replace by key in, a stored list
    // (ESS-SPEC-012 on `coalesce(subject.artifacts, []) + [input.artifact]`), so the artifact is
    // stored here by hand.
    pub(crate) fn keep_artifact(&self, task: &TaskId, artifact: Artifact, append: bool) {
        self.update(task, |held| {
            let artifacts = held.data.artifacts.get_or_insert_with(Vec::new);
            match artifacts
                .iter_mut()
                .find(|kept| kept.artifact_id == artifact.artifact_id)
            {
                Some(kept) if append => kept.parts.extend(artifact.parts),
                Some(kept) => *kept = artifact,
                None => artifacts.push(artifact),
            }
        });
    }

    /// Keeps a status message a step moved its task with in the task's history, unless the
    /// history holds [`Limits::max_history`] messages: the step has moved the task already, and
    /// its status carries the message.
    // ESS gap (ess 0.67.0): as for `keep_artifact`, `sets:` cannot append to the stored
    // `history`, so the message is stored here by hand.
    pub(crate) fn record(&self, task: &TaskId, message: Message) {
        let cap = self.0.limits.max_history;
        self.update(task, |held| {
            if held.data.history.as_ref().map_or(0, Vec::len) < cap {
                push_history(&mut held.data, message);
            }
        });
    }

    fn store_config(&self, data: PushNotificationConfigData) {
        let snapshot =
            AnyPushNotificationConfig::Active(PushNotificationConfig::new(data)).snapshot();
        self.0
            .configs
            .borrow_mut()
            .insert(snapshot.data.row.0.clone(), snapshot);
    }

    /// The configurations of this task, in the order of their rows.
    fn configs_of(&self, task: &TaskId) -> Vec<PushNotificationConfigData> {
        let prefix = format!("{}/", task.0);
        self.0
            .configs
            .borrow()
            .range(prefix.clone()..)
            .take_while(|(row, _)| row.starts_with(&prefix))
            .map(|(_, snapshot)| &snapshot.data)
            .filter(|data| data.task_id == *task)
            .cloned()
            .collect()
    }

    /// How many configurations this task holds.
    fn configs_held(&self, task: &TaskId) -> usize {
        let prefix = format!("{}/", task.0);
        self.0
            .configs
            .borrow()
            .range(prefix.clone()..)
            .take_while(|(row, _)| row.starts_with(&prefix))
            .filter(|(_, snapshot)| snapshot.data.task_id == *task)
            .count()
    }
}

/// Major and minor of a version text: `1.0.7` and `1.0` are both `("1", "0")`.
fn major_minor(version: &str) -> (String, String) {
    let mut parts = version.trim().split('.');
    let major = parts.next().unwrap_or("").trim().to_owned();
    let minor = parts.next().unwrap_or("0").trim().to_owned();
    (major, minor)
}

/// Whether a request under `asked` may reach a task of `held`: a task answers only requests under
/// the tenant it was created under, and one created without a tenant only requests without one.
/// Any other request is answered as for an unknown id (`ess/domains/tasks.yaml`, GetTask
/// `other-tenant`).
fn same_tenant(held: Option<&str>, asked: Option<&str>) -> bool {
    held == asked
}

/// Whether a task in this state takes no further move.
pub(crate) fn terminal(state: TaskState) -> bool {
    matches!(
        state,
        TaskState::Completed | TaskState::Failed | TaskState::Canceled | TaskState::Rejected
    )
}

/// Whether a task in this state waits for its client.
pub(crate) fn interrupted(state: TaskState) -> bool {
    matches!(state, TaskState::InputRequired | TaskState::AuthRequired)
}

/// The wire spelling of a lifecycle state.
pub(crate) fn wire_state(state: TaskState) -> WireTaskState {
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

/// The lifecycle state of a wire state.
pub(crate) fn lifecycle_state(state: WireTaskState) -> TaskState {
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

/// A stored task as the wire carries it.
pub(crate) fn wire(held: TaskEntitySnapshot) -> TaskSnapshot {
    TaskSnapshot {
        id: held.data.id,
        context_id: Some(held.data.context_id),
        status: TaskStatus {
            state: wire_state(held.state),
            message: held.data.status_message,
            timestamp: held.data.status_timestamp,
        },
        artifacts: held.data.artifacts,
        history: held.data.history,
        metadata: held.data.metadata,
    }
}

/// A stored task as the wire carries it, with only its latest `history_length` messages where a
/// length is asked for (none for 0), and its artifacts where `artifacts`; copies only what it
/// answers.
// ESS gap (ess 0.67.0): no construct bounds a response list by an input
// (`ess/domains/tasks.yaml`, GetTask `history_length`), so the history is cut here.
pub(crate) fn wire_of(
    held: &TaskEntitySnapshot,
    history_length: Option<i64>,
    artifacts: bool,
) -> TaskSnapshot {
    let history = match history_length.and_then(|length| usize::try_from(length).ok()) {
        None => held.data.history.clone(),
        Some(0) => None,
        Some(length) => held
            .data
            .history
            .as_ref()
            .map(|history| history[history.len().saturating_sub(length)..].to_vec()),
    };
    TaskSnapshot {
        id: held.data.id.clone(),
        context_id: Some(held.data.context_id.clone()),
        status: TaskStatus {
            state: wire_state(held.state),
            message: held.data.status_message.clone(),
            timestamp: held.data.status_timestamp.clone(),
        },
        artifacts: if artifacts {
            held.data.artifacts.clone()
        } else {
            None
        },
        history,
        metadata: held.data.metadata.clone(),
    }
}

/// An RFC 3339 timestamp as an instant, `None` where it is not one.
pub(crate) fn instant(text: &str) -> Option<time::OffsetDateTime> {
    time::OffsetDateTime::parse(text, &time::format_description::well_known::Rfc3339).ok()
}

fn push_history(data: &mut TaskData, message: Message) {
    data.history.get_or_insert_with(Vec::new).push(message);
}

/// A step's status message, attached to its task and context where it names neither.
pub(crate) fn attached(
    message: Option<Message>,
    task: &TaskId,
    context: &ContextId,
) -> Option<Message> {
    message.map(|mut message| {
        message.task_id.get_or_insert_with(|| task.clone());
        message.context_id.get_or_insert_with(|| context.clone());
        message
    })
}

fn row_of(task: &TaskId, config: &PushNotificationConfigId) -> PushNotificationConfigRow {
    PushNotificationConfigRow(format!("{}/{}", task.0, config.0))
}

fn wire_config(data: PushNotificationConfigData) -> TaskPushNotificationConfig {
    TaskPushNotificationConfig {
        tenant: data.tenant,
        id: Some(data.config_id),
        task_id: Some(data.task_id),
        url: data.url,
        token: data.token,
        authentication: data.authentication,
    }
}

impl TaskStorage for ServerPorts {
    fn get(&self, identity: &TaskId) -> Option<TaskEntitySnapshot> {
        self.task(identity)
    }

    fn put(&mut self, snapshot: TaskEntitySnapshot) {
        self.store(snapshot);
    }

    fn delete(&mut self, identity: &TaskId) {
        self.0.tasks.borrow_mut().delete(identity);
    }

    fn list(&self) -> Vec<TaskEntitySnapshot> {
        self.0.tasks.borrow().held.values().cloned().collect()
    }
}

impl PushNotificationConfigStorage for ServerPorts {
    fn get(&self, identity: &PushNotificationConfigRow) -> Option<PushNotificationConfigSnapshot> {
        self.0.configs.borrow().get(&identity.0).cloned()
    }

    fn put(&mut self, snapshot: PushNotificationConfigSnapshot) {
        self.0
            .configs
            .borrow_mut()
            .insert(snapshot.data.row.0.clone(), snapshot);
    }

    fn delete(&mut self, identity: &PushNotificationConfigRow) {
        self.0.configs.borrow_mut().remove(&identity.0);
    }

    fn list(&self) -> Vec<PushNotificationConfigSnapshot> {
        self.0.configs.borrow().values().cloned().collect()
    }
}

impl Context for ServerPorts {
    fn generate_optional_timestamp(&mut self) -> Option<Timestamp> {
        self.stamp()
    }
}

impl SendMessageBehavior for ServerPorts {
    fn send_message(&mut self, input: SendMessage) -> Result<SendMessageOutcome, UnmetObligation> {
        match self.mismatch() {
            Some(Mismatch::Version) => {
                return Ok(SendMessageOutcome::VersionNotSupported {
                    error: VersionNotSupported,
                });
            }
            Some(Mismatch::Extension) => {
                return Ok(SendMessageOutcome::ExtensionRequired {
                    error: ExtensionSupportRequired,
                });
            }
            None => {}
        }
        Ok(
            match self.take(
                input.message,
                input.configuration.as_ref(),
                false,
                input.tenant,
            ) {
                Taken::NotFound => SendMessageOutcome::TaskNotFound {
                    error: TaskNotFound,
                },
                Taken::Terminal => SendMessageOutcome::TaskTerminal {
                    error: UnsupportedOperation,
                },
                Taken::ContextMismatch => SendMessageOutcome::ContextMismatch {
                    error: ContextMismatch,
                },
                Taken::ContextNotAccepted => SendMessageOutcome::ContextNotAccepted {
                    error: ContextNotAccepted,
                },
                Taken::ContentType => SendMessageOutcome::ContentTypeNotSupported {
                    error: ContentTypeNotSupported,
                },
                Taken::TaskLimit => SendMessageOutcome::TaskLimit {
                    error: ResourceExhausted,
                },
                Taken::HistoryLimit => SendMessageOutcome::HistoryLimit {
                    error: ResourceExhausted,
                },
                Taken::Direct(reply) => SendMessageOutcome::DirectMessage {
                    response: SendMessageResponse {
                        task: None,
                        reply: Some(reply),
                    },
                },
                Taken::Created(task, task_id) => SendMessageOutcome::TaskCreated {
                    response: SendMessageResponse {
                        task: Some(task),
                        reply: None,
                    },
                    task_created: TaskCreated { task_id },
                },
                Taken::Continued(task) => SendMessageOutcome::Continued {
                    response: SendMessageResponse {
                        task: Some(task),
                        reply: None,
                    },
                },
            },
        )
    }
}

impl SendStreamingMessageBehavior for ServerPorts {
    fn send_streaming_message(
        &mut self,
        input: SendStreamingMessage,
    ) -> Result<SendStreamingMessageOutcome, UnmetObligation> {
        match self.mismatch() {
            Some(Mismatch::Version) => {
                return Ok(SendStreamingMessageOutcome::VersionNotSupported {
                    error: VersionNotSupported,
                });
            }
            Some(Mismatch::Extension) => {
                return Ok(SendStreamingMessageOutcome::ExtensionRequired {
                    error: ExtensionSupportRequired,
                });
            }
            None => {}
        }
        if !self.streams() {
            return Ok(SendStreamingMessageOutcome::StreamingNotSupported {
                error: UnsupportedOperation,
            });
        }
        let opened = |task: Option<TaskSnapshot>, reply: Option<Message>| {
            (
                SendStreamingMessageResponse {
                    task: task.clone(),
                    reply: reply.clone(),
                },
                MessageStreamResponse {
                    task,
                    message: reply,
                    status_update: None,
                    artifact_update: None,
                },
            )
        };
        Ok(
            match self.take(
                input.message,
                input.configuration.as_ref(),
                true,
                input.tenant,
            ) {
                Taken::NotFound => SendStreamingMessageOutcome::TaskNotFound {
                    error: TaskNotFound,
                },
                Taken::Terminal => SendStreamingMessageOutcome::TaskTerminal {
                    error: UnsupportedOperation,
                },
                Taken::ContextMismatch => SendStreamingMessageOutcome::ContextMismatch {
                    error: ContextMismatch,
                },
                Taken::ContextNotAccepted => SendStreamingMessageOutcome::ContextNotAccepted {
                    error: ContextNotAccepted,
                },
                Taken::ContentType => SendStreamingMessageOutcome::ContentTypeNotSupported {
                    error: ContentTypeNotSupported,
                },
                Taken::TaskLimit => SendStreamingMessageOutcome::TaskLimit {
                    error: ResourceExhausted,
                },
                Taken::HistoryLimit => SendStreamingMessageOutcome::HistoryLimit {
                    error: ResourceExhausted,
                },
                Taken::Direct(reply) => {
                    let (response, message_stream_response) = opened(None, Some(reply));
                    SendStreamingMessageOutcome::DirectMessage {
                        response,
                        message_stream_response,
                    }
                }
                Taken::Created(task, task_id) => {
                    let (response, message_stream_response) = opened(Some(task), None);
                    SendStreamingMessageOutcome::TaskCreated {
                        response,
                        task_created: TaskCreated { task_id },
                        message_stream_response,
                    }
                }
                Taken::Continued(task) => {
                    let (response, message_stream_response) = opened(Some(task), None);
                    SendStreamingMessageOutcome::Continued {
                        response,
                        message_stream_response,
                    }
                }
            },
        )
    }
}

impl GetTaskBehavior for ServerPorts {
    fn get_task(&mut self, input: GetTask) -> Result<GetTaskOutcome, UnmetObligation> {
        Ok(match self.mismatch() {
            Some(Mismatch::Version) => GetTaskOutcome::VersionNotSupported {
                error: VersionNotSupported,
            },
            Some(Mismatch::Extension) => GetTaskOutcome::ExtensionRequired {
                error: ExtensionSupportRequired,
            },
            None => match self.read(&input.id, |held| {
                same_tenant(held.data.tenant.as_deref(), input.tenant.as_deref())
                    .then(|| wire_of(held, input.history_length, true))
            }) {
                None => GetTaskOutcome::NotFound {
                    error: TaskNotFound,
                },
                Some(None) => GetTaskOutcome::OtherTenant {
                    error: TaskNotFound,
                },
                Some(Some(task)) => GetTaskOutcome::Found {
                    response: GetTaskResponse { task },
                },
            },
        })
    }
}

impl CancelTaskBehavior for ServerPorts {
    fn cancel_task(&mut self, input: CancelTask) -> Result<CancelTaskOutcome, UnmetObligation> {
        match self.mismatch() {
            Some(Mismatch::Version) => {
                return Ok(CancelTaskOutcome::VersionNotSupported {
                    error: VersionNotSupported,
                });
            }
            Some(Mismatch::Extension) => {
                return Ok(CancelTaskOutcome::ExtensionRequired {
                    error: ExtensionSupportRequired,
                });
            }
            None => {}
        }
        let Some(held) = self.task(&input.id) else {
            return Ok(CancelTaskOutcome::NotFound {
                error: TaskNotFound,
            });
        };
        if !same_tenant(held.data.tenant.as_deref(), input.tenant.as_deref()) {
            return Ok(CancelTaskOutcome::OtherTenant {
                error: TaskNotFound,
            });
        }
        if terminal(held.state) {
            return Ok(CancelTaskOutcome::NotCancelable {
                error: TaskNotCancelable,
            });
        }
        if !self.0.agent.borrow().cancelable(&wire(held.clone())) {
            return Ok(CancelTaskOutcome::NotCancelableNow {
                error: TaskNotCancelable,
            });
        }
        let canceled = match held.refine() {
            AnyTask::Submitted(task) => task.cancel(),
            AnyTask::Working(task) => task.cancel(),
            AnyTask::InputRequired(task) => task.cancel(),
            AnyTask::AuthRequired(task) => task.cancel(),
            _ => {
                return Ok(CancelTaskOutcome::NotCancelable {
                    error: TaskNotCancelable,
                });
            }
        };
        let mut next = AnyTask::Canceled(canceled).snapshot();
        next.data.status_message = None;
        next.data.status_timestamp = self.stamp();
        self.store(next.clone());
        self.0
            .pending
            .borrow_mut()
            .retain(|work| work.task != input.id);
        let status = TaskStatus {
            state: WireTaskState::TASKSTATECANCELED,
            message: None,
            timestamp: next.data.status_timestamp.clone(),
        };
        let task_status_updated = TaskStatusUpdated {
            task_id: next.data.id.clone(),
            context_id: next.data.context_id.clone(),
            status,
        };
        Ok(CancelTaskOutcome::Canceled {
            response: CancelTaskResponse { task: wire(next) },
            task_status_updated,
        })
    }
}

impl SubscribeToTaskBehavior for ServerPorts {
    fn subscribe_to_task(
        &mut self,
        input: SubscribeToTask,
    ) -> Result<SubscribeToTaskOutcome, UnmetObligation> {
        Ok(match self.mismatch() {
            Some(Mismatch::Version) => SubscribeToTaskOutcome::VersionNotSupported {
                error: VersionNotSupported,
            },
            Some(Mismatch::Extension) => SubscribeToTaskOutcome::ExtensionRequired {
                error: ExtensionSupportRequired,
            },
            None if !self.streams() => SubscribeToTaskOutcome::StreamingNotSupported {
                error: UnsupportedOperation,
            },
            None => match self.task(&input.id) {
                None => SubscribeToTaskOutcome::NotFound {
                    error: TaskNotFound,
                },
                Some(held)
                    if !same_tenant(held.data.tenant.as_deref(), input.tenant.as_deref()) =>
                {
                    SubscribeToTaskOutcome::OtherTenant {
                        error: TaskNotFound,
                    }
                }
                Some(held) if terminal(held.state) => SubscribeToTaskOutcome::TaskTerminal {
                    error: UnsupportedOperation,
                },
                Some(held) => SubscribeToTaskOutcome::Subscribed {
                    response: SubscribeToTaskResponse { task: wire(held) },
                },
            },
        })
    }
}

impl CreateTaskPushNotificationConfigBehavior for ServerPorts {
    fn create_task_push_notification_config(
        &mut self,
        input: CreateTaskPushNotificationConfig,
    ) -> Result<CreateTaskPushNotificationConfigOutcome, UnmetObligation> {
        type Outcome = CreateTaskPushNotificationConfigOutcome;
        match self.mismatch() {
            Some(Mismatch::Version) => {
                return Ok(Outcome::VersionNotSupported {
                    error: VersionNotSupported,
                });
            }
            Some(Mismatch::Extension) => {
                return Ok(Outcome::ExtensionRequired {
                    error: ExtensionSupportRequired,
                });
            }
            None => {}
        }
        if !self.pushes() {
            return Ok(Outcome::PushNotSupported {
                error: PushNotificationNotSupported,
            });
        }
        // ESS gap (ess 0.67.0): the tenant guard of this command is UNMAPPED in
        // `ess/domains/tasks.yaml` (CreateTaskPushNotificationConfig, ESS-SYNTH-003), so another
        // tenant's task is answered as an unknown one here by hand.
        if !self.reachable(&input.task_id, input.tenant.as_deref()) {
            return Ok(Outcome::TaskNotFound {
                error: TaskNotFound,
            });
        }
        let named = input.id.is_some();
        let config_id = input
            .id
            .unwrap_or_else(|| PushNotificationConfigId(uuid::Uuid::new_v4().to_string()));
        let row = row_of(&input.task_id, &config_id);
        let replaces = self.0.configs.borrow().contains_key(&row.0);
        if !replaces && self.configs_held(&input.task_id) >= self.0.limits.max_push_configs {
            return Ok(Outcome::ConfigLimit {
                error: ResourceExhausted,
            });
        }
        let data = PushNotificationConfigData {
            row: row.clone(),
            config_id: config_id.clone(),
            task_id: input.task_id.clone(),
            tenant: input.tenant,
            url: input.url,
            token: input.token,
            authentication: input.authentication,
        };
        self.store_config(data.clone());
        let response = CreateTaskPushNotificationConfigResponse {
            config: wire_config(data),
        };
        let push_notification_config_created = PushNotificationConfigCreated {
            row,
            config_id,
            task_id: input.task_id,
        };
        Ok(if named {
            Outcome::Created {
                response,
                push_notification_config_created,
            }
        } else {
            Outcome::Assigned {
                response,
                push_notification_config_created,
            }
        })
    }
}

impl GetTaskPushNotificationConfigBehavior for ServerPorts {
    fn get_task_push_notification_config(
        &mut self,
        input: GetTaskPushNotificationConfig,
    ) -> Result<GetTaskPushNotificationConfigOutcome, UnmetObligation> {
        type Outcome = GetTaskPushNotificationConfigOutcome;
        Ok(match self.mismatch() {
            Some(Mismatch::Version) => Outcome::VersionNotSupported {
                error: VersionNotSupported,
            },
            Some(Mismatch::Extension) => Outcome::ExtensionRequired {
                error: ExtensionSupportRequired,
            },
            None if !self.pushes() => Outcome::PushNotSupported {
                error: PushNotificationNotSupported,
            },
            // ESS gap (ess 0.67.0): the tenant guard of this command is UNMAPPED in
            // `ess/domains/tasks.yaml` (GetTaskPushNotificationConfig, ESS-COMMAND-009), so
            // another tenant's task is answered as an unknown one here by hand.
            None if !self.reachable(&input.task_id, input.tenant.as_deref()) => Outcome::NotFound {
                error: TaskNotFound,
            },
            None => {
                match PushNotificationConfigStorage::get(self, &row_of(&input.task_id, &input.id)) {
                    None => Outcome::ConfigNotFound {
                        error: TaskNotFound,
                    },
                    Some(snapshot) => Outcome::Found {
                        response: GetTaskPushNotificationConfigResponse {
                            config: wire_config(snapshot.data),
                        },
                    },
                }
            }
        })
    }
}

impl ListTaskPushNotificationConfigsBehavior for ServerPorts {
    fn list_task_push_notification_configs(
        &mut self,
        input: ListTaskPushNotificationConfigs,
    ) -> Result<ListTaskPushNotificationConfigsOutcome, UnmetObligation> {
        type Outcome = ListTaskPushNotificationConfigsOutcome;
        match self.mismatch() {
            Some(Mismatch::Version) => {
                return Ok(Outcome::VersionNotSupported {
                    error: VersionNotSupported,
                });
            }
            Some(Mismatch::Extension) => {
                return Ok(Outcome::ExtensionRequired {
                    error: ExtensionSupportRequired,
                });
            }
            None => {}
        }
        if !self.pushes() {
            return Ok(Outcome::PushNotSupported {
                error: PushNotificationNotSupported,
            });
        }
        match self.read(&input.task_id, |held| {
            same_tenant(held.data.tenant.as_deref(), input.tenant.as_deref())
        }) {
            None => {
                return Ok(Outcome::TaskNotFound {
                    error: TaskNotFound,
                });
            }
            Some(false) => {
                return Ok(Outcome::OtherTenant {
                    error: TaskNotFound,
                });
            }
            Some(true) => {}
        }
        // ESS gap (ess 0.67.0): cursor paging has no construct (`ess/domains/tasks.yaml`, the
        // `Tasks` view), so the page and its token are made here.
        let mut all = self.configs_of(&input.task_id);
        all.sort_by(|left, right| left.config_id.0.cmp(&right.config_id.0));
        let start = match input
            .page_token
            .as_deref()
            .filter(|token| !token.is_empty())
        {
            None => 0,
            Some(token) => match paging::key(List::Configs, token) {
                Some(after) => all.partition_point(|config| config.config_id.0 <= after),
                None => all.len(),
            },
        };
        let size = input
            .page_size
            .and_then(|size| usize::try_from(size).ok())
            .filter(|size| *size > 0)
            .unwrap_or(all.len());
        let end = start.saturating_add(size).min(all.len());
        let next = match all.get(..end).and_then(<[_]>::last) {
            Some(last) if end < all.len() => paging::token(List::Configs, &last.config_id.0),
            _ => String::new(),
        };
        let configs = all[start..end].iter().cloned().map(wire_config).collect();
        Ok(Outcome::Listed {
            response: ListTaskPushNotificationConfigsResponse {
                configs: Some(configs),
                next_page_token: Some(next),
            },
        })
    }
}

impl DeleteTaskPushNotificationConfigBehavior for ServerPorts {
    fn delete_task_push_notification_config(
        &mut self,
        input: DeleteTaskPushNotificationConfig,
    ) -> Result<DeleteTaskPushNotificationConfigOutcome, UnmetObligation> {
        type Outcome = DeleteTaskPushNotificationConfigOutcome;
        Ok(match self.mismatch() {
            Some(Mismatch::Version) => Outcome::VersionNotSupported {
                error: VersionNotSupported,
            },
            Some(Mismatch::Extension) => Outcome::ExtensionRequired {
                error: ExtensionSupportRequired,
            },
            None if !self.pushes() => Outcome::PushNotSupported {
                error: PushNotificationNotSupported,
            },
            // ESS gap (ess 0.67.0): the tenant guard of this command is UNMAPPED in
            // `ess/domains/tasks.yaml` (DeleteTaskPushNotificationConfig, synthesis arranges
            // `deleted` under another tenant), so another tenant's task is answered as an unknown
            // one here by hand.
            None if !self.reachable(&input.task_id, input.tenant.as_deref()) => {
                Outcome::TaskNotFound {
                    error: TaskNotFound,
                }
            }
            None => {
                PushNotificationConfigStorage::delete(self, &row_of(&input.task_id, &input.id));
                Outcome::Deleted {
                    push_notification_config_deleted: PushNotificationConfigDeleted {
                        config_id: input.id,
                        task_id: input.task_id,
                    },
                }
            }
        })
    }
}

impl GetExtendedAgentCardBehavior for ServerPorts {
    fn get_extended_agent_card(
        &mut self,
        input: GetExtendedAgentCard,
    ) -> Result<GetExtendedAgentCardOutcome, UnmetObligation> {
        let _ = input;
        Ok(match self.mismatch() {
            Some(Mismatch::Version) => GetExtendedAgentCardOutcome::VersionNotSupported {
                error: a2a::discovery::VersionNotSupported,
            },
            Some(Mismatch::Extension) => GetExtendedAgentCardOutcome::ExtensionRequired {
                error: a2a::discovery::ExtensionSupportRequired,
            },
            None if self.0.card.capabilities.extended_agent_card != Some(true) => {
                GetExtendedAgentCardOutcome::NotSupported {
                    error: a2a::discovery::UnsupportedOperation,
                }
            }
            None => match &self.0.extended_card {
                None => GetExtendedAgentCardOutcome::NotConfigured {
                    error: ExtendedAgentCardNotConfigured,
                },
                Some(card) => GetExtendedAgentCardOutcome::Returned {
                    response: GetExtendedAgentCardResponse { card: card.clone() },
                },
            },
        })
    }
}

impl TasksQuery for ServerPorts {
    fn tasks(
        &self,
        context_id: Option<ContextId>,
        status: Option<TaskState>,
        status_timestamp_after: Option<Timestamp>,
        tenant: Option<String>,
    ) -> Result<Vec<Tasks>, UnmetObligation> {
        // ESS gap (ess 0.67.0): a view has no order, no page and no row built after its page is
        // chosen (`ess/domains/tasks.yaml`, the `Tasks` view), so building every row whole copies
        // every task of the tenant for a page of one. The rows carry only what the filter, the
        // order and the page read (id, context, state, status timestamp); the router builds the
        // page's tasks from the store (`ServerPorts::snapshot`).
        // upstream/a2a.proto:694-696: "greater than or equal to this value".
        let after = status_timestamp_after.map(|after| instant(&after.0));
        Ok(self
            .0
            .tasks
            .borrow()
            .held
            .values()
            .filter(|held| same_tenant(held.data.tenant.as_deref(), tenant.as_deref()))
            .filter(|held| {
                context_id
                    .as_ref()
                    .is_none_or(|context| *context == held.data.context_id)
            })
            .filter(|held| status.is_none_or(|status| status == held.state))
            .filter(|held| {
                after.is_none_or(|after| {
                    let at = held
                        .data
                        .status_timestamp
                        .as_ref()
                        .and_then(|at| instant(&at.0));
                    matches!((at, after), (Some(at), Some(after)) if at >= after)
                })
            })
            .map(|held| Tasks {
                id: held.data.id.clone(),
                context_id: held.data.context_id.clone(),
                state: held.state,
                status_message: None,
                status_timestamp: held.data.status_timestamp.clone(),
                artifacts: None,
                history: None,
                metadata: None,
            })
            .collect())
    }
}

impl TaskPushNotificationConfigsQuery for ServerPorts {
    fn task_push_notification_configs(
        &self,
        task_id: TaskId,
    ) -> Result<Vec<TaskPushNotificationConfigs>, UnmetObligation> {
        Ok(self
            .configs_of(&task_id)
            .into_iter()
            .map(|data| TaskPushNotificationConfigs {
                config_id: data.config_id,
                task_id: data.task_id,
                tenant: data.tenant,
                url: data.url,
                token: data.token,
                authentication: data.authentication,
            })
            .collect())
    }
}
