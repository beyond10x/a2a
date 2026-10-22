// generated from a2a v1
// model digest 340c397069089debcfcb2cf241e311c06d40ac443ecea2a196508e49c5fb6859
// contract digest f01a7ba8cee02bc352a5a026f681d10fa8fa5237c90807e3a38f323f46cb67ea
// do not edit: regenerate with `ess synthesize --layout crate`

//! Tasks — `a2a.tasks`.
//!
//! Tasks an A2A agent runs for its clients, the messages and artifacts they carry, and the push-notification configurations that follow them (A2A v1.0.1 sections 3 and 4.1-4.3, proto service A2AService).
//!
//! Everything this bounded context declares that the synthesis plan marks generated.

/// Artifact — `a2a.tasks.Artifact`.
///
/// An output the agent produced for a task (A2A v1.0.1 section 4.1.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    /// `artifact_id` — `String`.
    pub artifact_id: String,
    /// `name` — `Optional<String>`.
    pub name: Option<String>,
    /// `description` — `Optional<String>`.
    pub description: Option<String>,
    /// `parts` — `List<a2a.tasks.Part>`.
    pub parts: Vec<Part>,
    /// `metadata` — `Optional<Json>`.
    pub metadata: Option<crate::json::Value>,
    /// `extensions` — `Optional<List<String>>`.
    pub extensions: Option<Vec<String>>,
}

/// AuthenticationInfo — `a2a.tasks.AuthenticationInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticationInfo {
    /// `scheme` — `String`.
    pub scheme: String,
    /// `credentials` — `Optional<String>`.
    pub credentials: Option<String>,
}

/// ContextId — `a2a.tasks.ContextId`: a distinct wrapper around `String`.
///
/// Identifier grouping related tasks and messages (A2A v1.0.1 section 3.4.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextId(pub String);

/// Message — `a2a.tasks.Message`.
///
/// One turn of communication between client and agent (A2A v1.0.1 section 4.1.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// `message_id` — `String`.
    pub message_id: String,
    /// `context_id` — `Optional<a2a.tasks.ContextId>`.
    pub context_id: Option<ContextId>,
    /// `task_id` — `Optional<a2a.tasks.TaskId>`.
    pub task_id: Option<TaskId>,
    /// `role` — `a2a.tasks.Role`.
    pub role: Role,
    /// `parts` — `List<a2a.tasks.Part>`.
    pub parts: Vec<Part>,
    /// `metadata` — `Optional<Json>`.
    pub metadata: Option<crate::json::Value>,
    /// `extensions` — `Optional<List<String>>`.
    pub extensions: Option<Vec<String>>,
    /// `reference_task_ids` — `Optional<List<a2a.tasks.TaskId>>`.
    pub reference_task_ids: Option<Vec<TaskId>>,
}

/// Part — `a2a.tasks.Part`.
///
/// One piece of content in a message or artifact (A2A v1.0.1 section 4.1.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    /// `text` — `Optional<String>`.
    pub text: Option<String>,
    /// `raw` — `Optional<Bytes>`.
    pub raw: Option<Vec<u8>>,
    /// `url` — `Optional<String>`.
    pub url: Option<String>,
    /// `data` — `Optional<Json>`.
    pub data: Option<crate::json::Value>,
    /// `metadata` — `Optional<Json>`.
    pub metadata: Option<crate::json::Value>,
    /// `filename` — `Optional<String>`.
    pub filename: Option<String>,
    /// `media_type` — `Optional<String>`.
    pub media_type: Option<String>,
}

/// The states of `a2a.tasks.PushNotificationConfig`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `PushNotificationConfig<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushNotificationConfigState {
    /// `Active`.
    Active,
}

/// PushNotificationConfigId — `a2a.tasks.PushNotificationConfigId`: a distinct wrapper around `String`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushNotificationConfigId(pub String);

/// Role — `a2a.tasks.Role`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// `ROLE_USER`.
    ROLEUSER,
    /// `ROLE_AGENT`.
    ROLEAGENT,
}

/// SendMessageConfiguration — `a2a.tasks.SendMessageConfiguration`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendMessageConfiguration {
    /// `accepted_output_modes` — `Optional<List<String>>`.
    pub accepted_output_modes: Option<Vec<String>>,
    /// `task_push_notification_config` — `Optional<a2a.tasks.TaskPushNotificationConfig>`.
    pub task_push_notification_config: Option<TaskPushNotificationConfig>,
    /// `history_length` — `Optional<Integer>`.
    pub history_length: Option<i64>,
    /// `return_immediately` — `Optional<Boolean>`.
    pub return_immediately: Option<bool>,
}

/// The states of `a2a.tasks.Task`, as runtime values.
///
/// Synthesised from the lifecycle, so the two cannot disagree. Which *moves* are legal is not
/// carried here — it is carried by `Task<S>`, where an undeclared move does not compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// `AuthRequired`.
    AuthRequired,
    /// `Canceled`.
    Canceled,
    /// `Completed`.
    Completed,
    /// `Failed`.
    Failed,
    /// `InputRequired`.
    InputRequired,
    /// `Rejected`.
    Rejected,
    /// `Submitted`.
    Submitted,
    /// `Working`.
    Working,
}

/// TaskId — `a2a.tasks.TaskId`: a distinct wrapper around `String`.
///
/// Server-generated, opaque task identifier (A2A v1.0.1 section 3.4.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskId(pub String);

/// TaskPushNotificationConfig — `a2a.tasks.TaskPushNotificationConfig`.
///
/// The wire form of a push-notification configuration (proto TaskPushNotificationConfig).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskPushNotificationConfig {
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
    /// `id` — `Optional<a2a.tasks.PushNotificationConfigId>`.
    pub id: Option<PushNotificationConfigId>,
    /// `task_id` — `Optional<a2a.tasks.TaskId>`.
    pub task_id: Option<TaskId>,
    /// `url` — `String`.
    pub url: String,
    /// `token` — `Optional<String>`.
    pub token: Option<String>,
    /// `authentication` — `Optional<a2a.tasks.AuthenticationInfo>`.
    pub authentication: Option<AuthenticationInfo>,
}

/// TaskSnapshot — `a2a.tasks.TaskSnapshot`.
///
/// The wire form of a task, proto message Task (A2A v1.0.1 section 4.1.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSnapshot {
    /// `id` — `a2a.tasks.TaskId`.
    pub id: TaskId,
    /// `context_id` — `a2a.tasks.ContextId`.
    pub context_id: ContextId,
    /// `status` — `a2a.tasks.TaskStatus`.
    pub status: TaskStatus,
    /// `artifacts` — `Optional<List<a2a.tasks.Artifact>>`.
    pub artifacts: Option<Vec<Artifact>>,
    /// `history` — `Optional<List<a2a.tasks.Message>>`.
    pub history: Option<Vec<Message>>,
    /// `metadata` — `Optional<Json>`.
    pub metadata: Option<crate::json::Value>,
}

/// TaskState — `a2a.tasks.TaskState`: one of a closed set of names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireTaskState {
    /// `TASK_STATE_SUBMITTED`.
    TASKSTATESUBMITTED,
    /// `TASK_STATE_WORKING`.
    TASKSTATEWORKING,
    /// `TASK_STATE_COMPLETED`.
    TASKSTATECOMPLETED,
    /// `TASK_STATE_FAILED`.
    TASKSTATEFAILED,
    /// `TASK_STATE_CANCELED`.
    TASKSTATECANCELED,
    /// `TASK_STATE_INPUT_REQUIRED`.
    TASKSTATEINPUTREQUIRED,
    /// `TASK_STATE_REJECTED`.
    TASKSTATEREJECTED,
    /// `TASK_STATE_AUTH_REQUIRED`.
    TASKSTATEAUTHREQUIRED,
}

/// TaskStatus — `a2a.tasks.TaskStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskStatus {
    /// `state` — `a2a.tasks.TaskState`.
    pub state: WireTaskState,
    /// `message` — `Optional<a2a.tasks.Message>`.
    pub message: Option<Message>,
    /// `timestamp` — `Optional<Timestamp>`.
    pub timestamp: Option<crate::primitives::Timestamp>,
}

/// What PushNotificationConfig — `a2a.tasks.PushNotificationConfig` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`PushNotificationConfig<S>`], and at a boundary by [`PushNotificationConfigSnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushNotificationConfigData {
    /// The identity: `id` — `a2a.tasks.PushNotificationConfigId`.
    pub id: PushNotificationConfigId,
    /// `task_id` — `a2a.tasks.TaskId`.
    ///
    /// Carries `push_notification_configs`: `a2a.tasks.Task` owns many `a2a.tasks.PushNotificationConfig`.
    pub task_id: TaskId,
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
    /// `url` — `String`.
    pub url: String,
    /// `token` — `Optional<String>`.
    pub token: Option<String>,
    /// `authentication` — `Optional<a2a.tasks.AuthenticationInfo>`.
    pub authentication: Option<AuthenticationInfo>,
}

/// The states of `a2a.tasks.PushNotificationConfig`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](push_notification_config_state::Marker), so [`PushNotificationConfig<S>`](PushNotificationConfig) can only ever rest in a real state.
pub mod push_notification_config_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::Active {}
    }

    /// A declared state of `PushNotificationConfig`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::PushNotificationConfigState;
    }

    /// `Active`. Where a new instance starts.
    pub struct Active;

    impl Marker for Active {
        const STATE: super::PushNotificationConfigState = super::PushNotificationConfigState::Active;
    }
}

/// PushNotificationConfig — `a2a.tasks.PushNotificationConfig` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Active`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`PushNotificationConfigSnapshot`]
/// and [`PushNotificationConfigSnapshot::refine`].
pub struct PushNotificationConfig<S: push_notification_config_state::Marker> {
    data: PushNotificationConfigData,
    state: core::marker::PhantomData<S>,
}

impl<S: push_notification_config_state::Marker> PushNotificationConfig<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> PushNotificationConfigState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &PushNotificationConfigData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> PushNotificationConfigData {
        self.data
    }
}

impl PushNotificationConfig<push_notification_config_state::Active> {
    /// A new instance, resting in `Active` — the only state the lifecycle starts one in.
    pub fn new(data: PushNotificationConfigData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

/// `a2a.tasks.PushNotificationConfig` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`PushNotificationConfigSnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushNotificationConfigSnapshot {
    /// Where the instance is in its lifecycle.
    pub state: PushNotificationConfigState,
    /// What it holds.
    pub data: PushNotificationConfigData,
}

/// An `PushNotificationConfig` in whichever declared state it was found.
pub enum AnyPushNotificationConfig {
    /// Resting in `Active`.
    Active(PushNotificationConfig<push_notification_config_state::Active>),
}

impl PushNotificationConfigSnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `PushNotificationConfigState` cannot spell one.
    pub fn refine(self) -> AnyPushNotificationConfig {
        match self.state {
            PushNotificationConfigState::Active => AnyPushNotificationConfig::Active(PushNotificationConfig {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyPushNotificationConfig {
    /// The state, as the runtime value.
    pub fn state(&self) -> PushNotificationConfigState {
        match self {
            Self::Active(_) => PushNotificationConfigState::Active,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> PushNotificationConfigSnapshot {
        match self {
            Self::Active(instance) => PushNotificationConfigSnapshot {
                state: PushNotificationConfigState::Active,
                data: instance.into_data(),
            },
        }
    }
}

/// What Task — `a2a.tasks.Task` — holds, apart from where it is in its lifecycle.
///
/// The identity and every declared field. The state is deliberately not one: inside the domain it
/// is carried by the type parameter of [`Task<S>`], and at a boundary by [`TaskEntitySnapshot::state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskData {
    /// The identity: `id` — `a2a.tasks.TaskId`.
    pub id: TaskId,
    /// `context_id` — `a2a.tasks.ContextId`.
    pub context_id: ContextId,
    /// `status_message` — `Optional<a2a.tasks.Message>`.
    pub status_message: Option<Message>,
    /// `status_timestamp` — `Optional<Timestamp>`.
    pub status_timestamp: Option<crate::primitives::Timestamp>,
    /// `artifacts` — `Optional<List<a2a.tasks.Artifact>>`.
    pub artifacts: Option<Vec<Artifact>>,
    /// `history` — `Optional<List<a2a.tasks.Message>>`.
    pub history: Option<Vec<Message>>,
    /// `metadata` — `Optional<Json>`.
    pub metadata: Option<crate::json::Value>,
}

/// The states of `a2a.tasks.Task`, at the type level.
///
/// One marker type per declared state, sealed: a state the lifecycle does not declare cannot
/// implement [`Marker`](task_state::Marker), so [`Task<S>`](Task) can only ever rest in a real state.
pub mod task_state {
    /// Closes [`Marker`] over the declared states.
    mod sealed {
        /// Implemented only by the marker types beside this module.
        pub trait Sealed {}
        impl Sealed for super::AuthRequired {}
        impl Sealed for super::Canceled {}
        impl Sealed for super::Completed {}
        impl Sealed for super::Failed {}
        impl Sealed for super::InputRequired {}
        impl Sealed for super::Rejected {}
        impl Sealed for super::Submitted {}
        impl Sealed for super::Working {}
    }

    /// A declared state of `Task`, as a type.
    pub trait Marker: sealed::Sealed {
        /// The same state, as the runtime value.
        const STATE: super::TaskState;
    }

    /// `AuthRequired`.
    pub struct AuthRequired;

    impl Marker for AuthRequired {
        const STATE: super::TaskState = super::TaskState::AuthRequired;
    }

    /// `Canceled`. Terminal: an instance may rest here forever.
    pub struct Canceled;

    impl Marker for Canceled {
        const STATE: super::TaskState = super::TaskState::Canceled;
    }

    /// `Completed`. Terminal: an instance may rest here forever.
    pub struct Completed;

    impl Marker for Completed {
        const STATE: super::TaskState = super::TaskState::Completed;
    }

    /// `Failed`. Terminal: an instance may rest here forever.
    pub struct Failed;

    impl Marker for Failed {
        const STATE: super::TaskState = super::TaskState::Failed;
    }

    /// `InputRequired`.
    pub struct InputRequired;

    impl Marker for InputRequired {
        const STATE: super::TaskState = super::TaskState::InputRequired;
    }

    /// `Rejected`. Terminal: an instance may rest here forever.
    pub struct Rejected;

    impl Marker for Rejected {
        const STATE: super::TaskState = super::TaskState::Rejected;
    }

    /// `Submitted`. Where a new instance starts.
    pub struct Submitted;

    impl Marker for Submitted {
        const STATE: super::TaskState = super::TaskState::Submitted;
    }

    /// `Working`.
    pub struct Working;

    impl Marker for Working {
        const STATE: super::TaskState = super::TaskState::Working;
    }
}

/// Task — `a2a.tasks.Task` — with its lifecycle state carried by the type.
///
/// The one constructor rests in `Submitted`, and the only way to change `S` is a method generated from
/// a declared transition. A move the specification does not declare is therefore not an error
/// case: it does not compile. Where the state is data — wire, storage — use [`TaskEntitySnapshot`]
/// and [`TaskEntitySnapshot::refine`].
pub struct Task<S: task_state::Marker> {
    data: TaskData,
    state: core::marker::PhantomData<S>,
}

impl<S: task_state::Marker> Task<S> {
    /// The state this instance rests in, as the runtime value.
    pub fn state(&self) -> TaskState {
        S::STATE
    }

    /// What it holds.
    pub fn data(&self) -> &TaskData {
        &self.data
    }

    /// Hands the data back, giving up the typed state.
    pub fn into_data(self) -> TaskData {
        self.data
    }
}

impl Task<task_state::Submitted> {
    /// A new instance, resting in `Submitted` — the only state the lifecycle starts one in.
    pub fn new(data: TaskData) -> Self {
        Self {
            data,
            state: core::marker::PhantomData,
        }
    }
}

impl Task<task_state::AuthRequired> {
    /// `work` — `AuthRequired` → `Working`. Taken by the `moved` outcome of `a2a.tasks.StartWorking`.
    pub fn work(self) -> Task<task_state::Working> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `complete` — `AuthRequired` → `Completed`. Taken by the `moved` outcome of `a2a.tasks.CompleteTask`.
    pub fn complete(self) -> Task<task_state::Completed> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `fail` — `AuthRequired` → `Failed`. Taken by the `moved` outcome of `a2a.tasks.FailTask`.
    pub fn fail(self) -> Task<task_state::Failed> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `reject` — `AuthRequired` → `Rejected`. Taken by the `moved` outcome of `a2a.tasks.RejectTask`.
    pub fn reject(self) -> Task<task_state::Rejected> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `cancel` — `AuthRequired` → `Canceled`. Taken by the `canceled` outcome of `a2a.tasks.CancelTask`.
    pub fn cancel(self) -> Task<task_state::Canceled> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl Task<task_state::InputRequired> {
    /// `work` — `InputRequired` → `Working`. Taken by the `moved` outcome of `a2a.tasks.StartWorking`.
    pub fn work(self) -> Task<task_state::Working> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `complete` — `InputRequired` → `Completed`. Taken by the `moved` outcome of `a2a.tasks.CompleteTask`.
    pub fn complete(self) -> Task<task_state::Completed> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `fail` — `InputRequired` → `Failed`. Taken by the `moved` outcome of `a2a.tasks.FailTask`.
    pub fn fail(self) -> Task<task_state::Failed> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `reject` — `InputRequired` → `Rejected`. Taken by the `moved` outcome of `a2a.tasks.RejectTask`.
    pub fn reject(self) -> Task<task_state::Rejected> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `cancel` — `InputRequired` → `Canceled`. Taken by the `canceled` outcome of `a2a.tasks.CancelTask`.
    pub fn cancel(self) -> Task<task_state::Canceled> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl Task<task_state::Submitted> {
    /// `work` — `Submitted` → `Working`. Taken by the `moved` outcome of `a2a.tasks.StartWorking`.
    pub fn work(self) -> Task<task_state::Working> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `require_input` — `Submitted` → `InputRequired`. Taken by the `moved` outcome of `a2a.tasks.RequireInput`.
    pub fn require_input(self) -> Task<task_state::InputRequired> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `require_auth` — `Submitted` → `AuthRequired`. Taken by the `moved` outcome of `a2a.tasks.RequireAuth`.
    pub fn require_auth(self) -> Task<task_state::AuthRequired> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `complete` — `Submitted` → `Completed`. Taken by the `moved` outcome of `a2a.tasks.CompleteTask`.
    pub fn complete(self) -> Task<task_state::Completed> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `fail` — `Submitted` → `Failed`. Taken by the `moved` outcome of `a2a.tasks.FailTask`.
    pub fn fail(self) -> Task<task_state::Failed> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `reject` — `Submitted` → `Rejected`. Taken by the `moved` outcome of `a2a.tasks.RejectTask`.
    pub fn reject(self) -> Task<task_state::Rejected> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `cancel` — `Submitted` → `Canceled`. Taken by the `canceled` outcome of `a2a.tasks.CancelTask`.
    pub fn cancel(self) -> Task<task_state::Canceled> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

impl Task<task_state::Working> {
    /// `require_input` — `Working` → `InputRequired`. Taken by the `moved` outcome of `a2a.tasks.RequireInput`.
    pub fn require_input(self) -> Task<task_state::InputRequired> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `require_auth` — `Working` → `AuthRequired`. Taken by the `moved` outcome of `a2a.tasks.RequireAuth`.
    pub fn require_auth(self) -> Task<task_state::AuthRequired> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `complete` — `Working` → `Completed`. Taken by the `moved` outcome of `a2a.tasks.CompleteTask`.
    pub fn complete(self) -> Task<task_state::Completed> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `fail` — `Working` → `Failed`. Taken by the `moved` outcome of `a2a.tasks.FailTask`.
    pub fn fail(self) -> Task<task_state::Failed> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `reject` — `Working` → `Rejected`. Taken by the `moved` outcome of `a2a.tasks.RejectTask`.
    pub fn reject(self) -> Task<task_state::Rejected> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }

    /// `cancel` — `Working` → `Canceled`. Taken by the `canceled` outcome of `a2a.tasks.CancelTask`.
    pub fn cancel(self) -> Task<task_state::Canceled> {
        Task {
            data: self.data,
            state: core::marker::PhantomData,
        }
    }
}

/// `a2a.tasks.Task` as it crosses a boundary: the state as a value beside the data.
///
/// Wire and storage know states only at runtime; [`TaskEntitySnapshot::refine`] is the one door back
/// into the typed lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskEntitySnapshot {
    /// Where the instance is in its lifecycle.
    pub state: TaskState,
    /// What it holds.
    pub data: TaskData,
}

/// An `Task` in whichever declared state it was found.
pub enum AnyTask {
    /// Resting in `AuthRequired`.
    AuthRequired(Task<task_state::AuthRequired>),
    /// Resting in `Canceled`.
    Canceled(Task<task_state::Canceled>),
    /// Resting in `Completed`.
    Completed(Task<task_state::Completed>),
    /// Resting in `Failed`.
    Failed(Task<task_state::Failed>),
    /// Resting in `InputRequired`.
    InputRequired(Task<task_state::InputRequired>),
    /// Resting in `Rejected`.
    Rejected(Task<task_state::Rejected>),
    /// Resting in `Submitted`.
    Submitted(Task<task_state::Submitted>),
    /// Resting in `Working`.
    Working(Task<task_state::Working>),
}

impl TaskEntitySnapshot {
    /// Refines the runtime state into the typed one.
    ///
    /// Total: every declared state has an arm, and an undeclared state cannot reach here because
    /// `TaskState` cannot spell one.
    pub fn refine(self) -> AnyTask {
        match self.state {
            TaskState::AuthRequired => AnyTask::AuthRequired(Task {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            TaskState::Canceled => AnyTask::Canceled(Task {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            TaskState::Completed => AnyTask::Completed(Task {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            TaskState::Failed => AnyTask::Failed(Task {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            TaskState::InputRequired => AnyTask::InputRequired(Task {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            TaskState::Rejected => AnyTask::Rejected(Task {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            TaskState::Submitted => AnyTask::Submitted(Task {
                data: self.data,
                state: core::marker::PhantomData,
            }),
            TaskState::Working => AnyTask::Working(Task {
                data: self.data,
                state: core::marker::PhantomData,
            }),
        }
    }
}

impl AnyTask {
    /// The state, as the runtime value.
    pub fn state(&self) -> TaskState {
        match self {
            Self::AuthRequired(_) => TaskState::AuthRequired,
            Self::Canceled(_) => TaskState::Canceled,
            Self::Completed(_) => TaskState::Completed,
            Self::Failed(_) => TaskState::Failed,
            Self::InputRequired(_) => TaskState::InputRequired,
            Self::Rejected(_) => TaskState::Rejected,
            Self::Submitted(_) => TaskState::Submitted,
            Self::Working(_) => TaskState::Working,
        }
    }

    /// Back to the boundary shape.
    pub fn snapshot(self) -> TaskEntitySnapshot {
        match self {
            Self::AuthRequired(instance) => TaskEntitySnapshot {
                state: TaskState::AuthRequired,
                data: instance.into_data(),
            },
            Self::Canceled(instance) => TaskEntitySnapshot {
                state: TaskState::Canceled,
                data: instance.into_data(),
            },
            Self::Completed(instance) => TaskEntitySnapshot {
                state: TaskState::Completed,
                data: instance.into_data(),
            },
            Self::Failed(instance) => TaskEntitySnapshot {
                state: TaskState::Failed,
                data: instance.into_data(),
            },
            Self::InputRequired(instance) => TaskEntitySnapshot {
                state: TaskState::InputRequired,
                data: instance.into_data(),
            },
            Self::Rejected(instance) => TaskEntitySnapshot {
                state: TaskState::Rejected,
                data: instance.into_data(),
            },
            Self::Submitted(instance) => TaskEntitySnapshot {
                state: TaskState::Submitted,
                data: instance.into_data(),
            },
            Self::Working(instance) => TaskEntitySnapshot {
                state: TaskState::Working,
                data: instance.into_data(),
            },
        }
    }
}

/// Cancel task — the input of `a2a.tasks.CancelTask`.
///
/// Ask the agent to cancel a task (A2A v1.0.1 section 3.1.5; POST /tasks/{id}:cancel).
///
/// Everything it can result in is [`CancelTaskOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelTask {
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
    /// `id` — `a2a.tasks.TaskId`.
    pub id: TaskId,
    /// `metadata` — `Optional<Json>`.
    pub metadata: Option<crate::json::Value>,
}

/// Actual typed response of `a2a.tasks.CancelTask`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelTaskResponse {
    /// `task` — `a2a.tasks.TaskSnapshot`.
    pub task: TaskSnapshot,
}

/// Everything `a2a.tasks.CancelTask` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelTaskOutcome {
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `not-cancelable-now` — externally decided (the agent cannot cancel the task at its current stage).
    NotCancelableNow {
        /// Why it was refused: `a2a.tasks.TaskNotCancelable`.
        error: TaskNotCancelable,
    },
    /// `canceled` — otherwise.
    Canceled {
        /// Actual response returned by this branch.
        response: CancelTaskResponse,
        /// The `a2a.tasks.TaskStatusUpdated` this outcome publishes.
        task_status_updated: TaskStatusUpdated,
    },
    /// `not-cancelable` — from a state no declared move starts in.
    ///
    /// The task is in a terminal state.
    NotCancelable {
        /// Why it was refused: `a2a.tasks.TaskNotCancelable`.
        error: TaskNotCancelable,
    },
    /// `version-not-supported` — externally decided (the A2A-Version service parameter names a version the agent does not support).
    VersionNotSupported {
        /// Why it was refused: `a2a.tasks.VersionNotSupported`.
        error: VersionNotSupported,
    },
    /// `extension-required` — externally decided (the agent card marks an extension required and the request does not declare it).
    ExtensionRequired {
        /// Why it was refused: `a2a.tasks.ExtensionSupportRequired`.
        error: ExtensionSupportRequired,
    },
}

/// Complete task — the input of `a2a.tasks.CompleteTask`.
///
/// The task finished successfully (TASK_STATE_COMPLETED).
///
/// Everything it can result in is [`CompleteTaskOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteTask {
    /// `id` — `a2a.tasks.TaskId`.
    pub id: TaskId,
    /// `status_message` — `Optional<a2a.tasks.Message>`.
    pub status_message: Option<Message>,
}

/// Everything `a2a.tasks.CompleteTask` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompleteTaskOutcome {
    /// `moved` — otherwise.
    Moved {
        /// The `a2a.tasks.TaskStatusUpdated` this outcome publishes.
        task_status_updated: TaskStatusUpdated,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `a2a.tasks.TaskStateConflict`.
        error: TaskStateConflict,
    },
}

/// Create push notification config — the input of `a2a.tasks.CreateTaskPushNotificationConfig`.
///
/// Register a webhook for a task's updates (A2A v1.0.1 section 3.1.7; POST /tasks/{task_id}/pushNotificationConfigs).
///
/// Everything it can result in is [`CreateTaskPushNotificationConfigOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTaskPushNotificationConfig {
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
    /// `id` — `Optional<a2a.tasks.PushNotificationConfigId>`.
    pub id: Option<PushNotificationConfigId>,
    /// `task_id` — `a2a.tasks.TaskId`.
    pub task_id: TaskId,
    /// `url` — `String`.
    pub url: String,
    /// `token` — `Optional<String>`.
    pub token: Option<String>,
    /// `authentication` — `Optional<a2a.tasks.AuthenticationInfo>`.
    pub authentication: Option<AuthenticationInfo>,
}

/// Actual typed response of `a2a.tasks.CreateTaskPushNotificationConfig`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTaskPushNotificationConfigResponse {
    /// `config` — `a2a.tasks.TaskPushNotificationConfig`.
    pub config: TaskPushNotificationConfig,
}

/// Everything `a2a.tasks.CreateTaskPushNotificationConfig` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateTaskPushNotificationConfigOutcome {
    /// `task-not-found` — when no `a2a.tasks.Task` carries the identity `input.task_id` names.
    TaskNotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `created` — otherwise.
    Created {
        /// Actual response returned by this branch.
        response: CreateTaskPushNotificationConfigResponse,
        /// The `a2a.tasks.PushNotificationConfigCreated` this outcome publishes.
        push_notification_config_created: PushNotificationConfigCreated,
    },
    /// `version-not-supported` — externally decided (the A2A-Version service parameter names a version the agent does not support).
    VersionNotSupported {
        /// Why it was refused: `a2a.tasks.VersionNotSupported`.
        error: VersionNotSupported,
    },
    /// `extension-required` — externally decided (the agent card marks an extension required and the request does not declare it).
    ExtensionRequired {
        /// Why it was refused: `a2a.tasks.ExtensionSupportRequired`.
        error: ExtensionSupportRequired,
    },
    /// `push-not-supported` — externally decided (the agent card does not declare capabilities.pushNotifications).
    PushNotSupported {
        /// Why it was refused: `a2a.tasks.PushNotificationNotSupported`.
        error: PushNotificationNotSupported,
    },
}

/// Delete push notification config — the input of `a2a.tasks.DeleteTaskPushNotificationConfig`.
///
/// Remove a push-notification configuration; deleting it again has the same effect (A2A v1.0.1 section 3.1.10; DELETE /tasks/{task_id}/pushNotificationConfigs/{id}).
///
/// Everything it can result in is [`DeleteTaskPushNotificationConfigOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteTaskPushNotificationConfig {
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
    /// `task_id` — `a2a.tasks.TaskId`.
    pub task_id: TaskId,
    /// `id` — `a2a.tasks.PushNotificationConfigId`.
    pub id: PushNotificationConfigId,
}

/// Everything `a2a.tasks.DeleteTaskPushNotificationConfig` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteTaskPushNotificationConfigOutcome {
    /// `task-not-found` — when no `a2a.tasks.Task` carries the identity `input.task_id` names.
    TaskNotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `already-gone` — for an identity no record carries.
    AlreadyGone,
    /// `deleted` — otherwise.
    Deleted {
        /// The `a2a.tasks.PushNotificationConfigDeleted` this outcome publishes.
        push_notification_config_deleted: PushNotificationConfigDeleted,
    },
    /// `version-not-supported` — externally decided (the A2A-Version service parameter names a version the agent does not support).
    VersionNotSupported {
        /// Why it was refused: `a2a.tasks.VersionNotSupported`.
        error: VersionNotSupported,
    },
    /// `extension-required` — externally decided (the agent card marks an extension required and the request does not declare it).
    ExtensionRequired {
        /// Why it was refused: `a2a.tasks.ExtensionSupportRequired`.
        error: ExtensionSupportRequired,
    },
    /// `push-not-supported` — externally decided (the agent card does not declare capabilities.pushNotifications).
    PushNotSupported {
        /// Why it was refused: `a2a.tasks.PushNotificationNotSupported`.
        error: PushNotificationNotSupported,
    },
}

/// Fail task — the input of `a2a.tasks.FailTask`.
///
/// The task finished with an error (TASK_STATE_FAILED).
///
/// Everything it can result in is [`FailTaskOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailTask {
    /// `id` — `a2a.tasks.TaskId`.
    pub id: TaskId,
    /// `status_message` — `Optional<a2a.tasks.Message>`.
    pub status_message: Option<Message>,
}

/// Everything `a2a.tasks.FailTask` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailTaskOutcome {
    /// `moved` — otherwise.
    Moved {
        /// The `a2a.tasks.TaskStatusUpdated` this outcome publishes.
        task_status_updated: TaskStatusUpdated,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `a2a.tasks.TaskStateConflict`.
        error: TaskStateConflict,
    },
}

/// Get task — the input of `a2a.tasks.GetTask`.
///
/// Read a task's current state (A2A v1.0.1 section 3.1.3; GET /tasks/{id}).
///
/// Everything it can result in is [`GetTaskOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTask {
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
    /// `id` — `a2a.tasks.TaskId`.
    pub id: TaskId,
    /// `history_length` — `Optional<Integer>`.
    pub history_length: Option<i64>,
}

/// Actual typed response of `a2a.tasks.GetTask`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTaskResponse {
    /// `task` — `a2a.tasks.TaskSnapshot`.
    pub task: TaskSnapshot,
}

/// Everything `a2a.tasks.GetTask` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetTaskOutcome {
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `found` — otherwise.
    Found {
        /// Actual response returned by this branch.
        response: GetTaskResponse,
    },
    /// `version-not-supported` — externally decided (the A2A-Version service parameter names a version the agent does not support).
    VersionNotSupported {
        /// Why it was refused: `a2a.tasks.VersionNotSupported`.
        error: VersionNotSupported,
    },
    /// `extension-required` — externally decided (the agent card marks an extension required and the request does not declare it).
    ExtensionRequired {
        /// Why it was refused: `a2a.tasks.ExtensionSupportRequired`.
        error: ExtensionSupportRequired,
    },
}

/// Get push notification config — the input of `a2a.tasks.GetTaskPushNotificationConfig`.
///
/// Read one push-notification configuration (A2A v1.0.1 section 3.1.8; GET /tasks/{task_id}/pushNotificationConfigs/{id}).
///
/// Everything it can result in is [`GetTaskPushNotificationConfigOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTaskPushNotificationConfig {
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
    /// `task_id` — `a2a.tasks.TaskId`.
    pub task_id: TaskId,
    /// `id` — `a2a.tasks.PushNotificationConfigId`.
    pub id: PushNotificationConfigId,
}

/// Actual typed response of `a2a.tasks.GetTaskPushNotificationConfig`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTaskPushNotificationConfigResponse {
    /// `config` — `a2a.tasks.TaskPushNotificationConfig`.
    pub config: TaskPushNotificationConfig,
}

/// Everything `a2a.tasks.GetTaskPushNotificationConfig` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetTaskPushNotificationConfigOutcome {
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `other-task` — when the existing subject's stored fields satisfy `task_id != input.task_id`.
    OtherTask {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `found` — otherwise.
    Found {
        /// Actual response returned by this branch.
        response: GetTaskPushNotificationConfigResponse,
    },
    /// `version-not-supported` — externally decided (the A2A-Version service parameter names a version the agent does not support).
    VersionNotSupported {
        /// Why it was refused: `a2a.tasks.VersionNotSupported`.
        error: VersionNotSupported,
    },
    /// `extension-required` — externally decided (the agent card marks an extension required and the request does not declare it).
    ExtensionRequired {
        /// Why it was refused: `a2a.tasks.ExtensionSupportRequired`.
        error: ExtensionSupportRequired,
    },
    /// `push-not-supported` — externally decided (the agent card does not declare capabilities.pushNotifications).
    PushNotSupported {
        /// Why it was refused: `a2a.tasks.PushNotificationNotSupported`.
        error: PushNotificationNotSupported,
    },
}

/// Reject task — the input of `a2a.tasks.RejectTask`.
///
/// The agent declines the task (TASK_STATE_REJECTED).
///
/// Everything it can result in is [`RejectTaskOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectTask {
    /// `id` — `a2a.tasks.TaskId`.
    pub id: TaskId,
    /// `status_message` — `Optional<a2a.tasks.Message>`.
    pub status_message: Option<Message>,
}

/// Everything `a2a.tasks.RejectTask` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectTaskOutcome {
    /// `moved` — otherwise.
    Moved {
        /// The `a2a.tasks.TaskStatusUpdated` this outcome publishes.
        task_status_updated: TaskStatusUpdated,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `a2a.tasks.TaskStateConflict`.
        error: TaskStateConflict,
    },
}

/// Require authorization — the input of `a2a.tasks.RequireAuth`.
///
/// The agent needs the client to authorize (TASK_STATE_AUTH_REQUIRED, section 7.6).
///
/// Everything it can result in is [`RequireAuthOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequireAuth {
    /// `id` — `a2a.tasks.TaskId`.
    pub id: TaskId,
    /// `status_message` — `Optional<a2a.tasks.Message>`.
    pub status_message: Option<Message>,
}

/// Everything `a2a.tasks.RequireAuth` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequireAuthOutcome {
    /// `moved` — otherwise.
    Moved {
        /// The `a2a.tasks.TaskStatusUpdated` this outcome publishes.
        task_status_updated: TaskStatusUpdated,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `a2a.tasks.TaskStateConflict`.
        error: TaskStateConflict,
    },
}

/// Require input — the input of `a2a.tasks.RequireInput`.
///
/// The agent needs more input from the client (TASK_STATE_INPUT_REQUIRED, section 3.4.3).
///
/// Everything it can result in is [`RequireInputOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequireInput {
    /// `id` — `a2a.tasks.TaskId`.
    pub id: TaskId,
    /// `status_message` — `Optional<a2a.tasks.Message>`.
    pub status_message: Option<Message>,
}

/// Everything `a2a.tasks.RequireInput` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequireInputOutcome {
    /// `moved` — otherwise.
    Moved {
        /// The `a2a.tasks.TaskStatusUpdated` this outcome publishes.
        task_status_updated: TaskStatusUpdated,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `a2a.tasks.TaskStateConflict`.
        error: TaskStateConflict,
    },
}

/// Send message — the input of `a2a.tasks.SendMessage`.
///
/// Send a message to the agent, which answers with a task tracking its work or with a direct message (A2A v1.0.1 section 3.1.1; POST /message:send).
///
/// Everything it can result in is [`SendMessageOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendMessage {
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
    /// `message` — `a2a.tasks.Message`.
    pub message: Message,
    /// `configuration` — `Optional<a2a.tasks.SendMessageConfiguration>`.
    pub configuration: Option<SendMessageConfiguration>,
    /// `metadata` — `Optional<Json>`.
    pub metadata: Option<crate::json::Value>,
}

/// Actual typed response of `a2a.tasks.SendMessage`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendMessageResponse {
    /// `task` — `Optional<a2a.tasks.TaskSnapshot>`.
    pub task: Option<TaskSnapshot>,
    /// `reply` — `Optional<a2a.tasks.Message>`.
    pub reply: Option<Message>,
}

/// Everything `a2a.tasks.SendMessage` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendMessageOutcome {
    /// `task-not-found` — externally decided (message.taskId names no task the caller can access (section 3.4.2)).
    TaskNotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `task-terminal` — externally decided (message.taskId names a task in Completed, Failed, Canceled or Rejected).
    TaskTerminal {
        /// Why it was refused: `a2a.tasks.UnsupportedOperation`.
        error: UnsupportedOperation,
    },
    /// `context-mismatch` — externally decided (message.contextId differs from the contextId of the task message.taskId names).
    ContextMismatch {
        /// Why it was refused: `a2a.tasks.ContextMismatch`.
        error: ContextMismatch,
    },
    /// `content-type-not-supported` — externally decided (a part's media type is not supported by the agent).
    ContentTypeNotSupported {
        /// Why it was refused: `a2a.tasks.ContentTypeNotSupported`.
        error: ContentTypeNotSupported,
    },
    /// `continued` — externally decided (message.taskId names a non-terminal task, and the agent takes the message into it).
    ///
    /// The agent answers with the task the message continued.
    Continued {
        /// Actual response returned by this branch.
        response: SendMessageResponse,
    },
    /// `direct-message` — externally decided (the agent answers with a message and creates no task).
    DirectMessage {
        /// Actual response returned by this branch.
        response: SendMessageResponse,
    },
    /// `task-created` — otherwise.
    ///
    /// A new task in Submitted; the agent works on it after the answer.
    TaskCreated {
        /// Actual response returned by this branch.
        response: SendMessageResponse,
        /// The `a2a.tasks.TaskCreated` this outcome publishes.
        task_created: TaskCreated,
    },
    /// `version-not-supported` — externally decided (the A2A-Version service parameter names a version the agent does not support).
    VersionNotSupported {
        /// Why it was refused: `a2a.tasks.VersionNotSupported`.
        error: VersionNotSupported,
    },
    /// `extension-required` — externally decided (the agent card marks an extension required and the request does not declare it).
    ExtensionRequired {
        /// Why it was refused: `a2a.tasks.ExtensionSupportRequired`.
        error: ExtensionSupportRequired,
    },
}

/// Send streaming message — the input of `a2a.tasks.SendStreamingMessage`.
///
/// Send a message and stream the answer: one message, or the task followed by its status and artifact updates until it is terminal or interrupted (A2A v1.0.1 section 3.1.2; POST /message:stream).
///
/// Everything it can result in is [`SendStreamingMessageOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendStreamingMessage {
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
    /// `message` — `a2a.tasks.Message`.
    pub message: Message,
    /// `configuration` — `Optional<a2a.tasks.SendMessageConfiguration>`.
    pub configuration: Option<SendMessageConfiguration>,
    /// `metadata` — `Optional<Json>`.
    pub metadata: Option<crate::json::Value>,
}

/// Actual typed response of `a2a.tasks.SendStreamingMessage`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendStreamingMessageResponse {
    /// `task` — `Optional<a2a.tasks.TaskSnapshot>`.
    pub task: Option<TaskSnapshot>,
    /// `reply` — `Optional<a2a.tasks.Message>`.
    pub reply: Option<Message>,
}

/// Everything `a2a.tasks.SendStreamingMessage` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendStreamingMessageOutcome {
    /// `task-not-found` — externally decided (message.taskId names no task the caller can access (section 3.4.2)).
    TaskNotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `task-terminal` — externally decided (message.taskId names a task in Completed, Failed, Canceled or Rejected).
    TaskTerminal {
        /// Why it was refused: `a2a.tasks.UnsupportedOperation`.
        error: UnsupportedOperation,
    },
    /// `context-mismatch` — externally decided (message.contextId differs from the contextId of the task message.taskId names).
    ContextMismatch {
        /// Why it was refused: `a2a.tasks.ContextMismatch`.
        error: ContextMismatch,
    },
    /// `content-type-not-supported` — externally decided (a part's media type is not supported by the agent).
    ContentTypeNotSupported {
        /// Why it was refused: `a2a.tasks.ContentTypeNotSupported`.
        error: ContentTypeNotSupported,
    },
    /// `continued` — externally decided (message.taskId names a non-terminal task, and the agent takes the message into it).
    ///
    /// The agent answers with the task the message continued.
    Continued {
        /// Actual response returned by this branch.
        response: SendStreamingMessageResponse,
    },
    /// `direct-message` — externally decided (the agent answers with a message and creates no task).
    DirectMessage {
        /// Actual response returned by this branch.
        response: SendStreamingMessageResponse,
    },
    /// `task-created` — otherwise.
    ///
    /// A new task in Submitted; the agent works on it after the answer.
    TaskCreated {
        /// Actual response returned by this branch.
        response: SendStreamingMessageResponse,
        /// The `a2a.tasks.TaskCreated` this outcome publishes.
        task_created: TaskCreated,
    },
    /// `version-not-supported` — externally decided (the A2A-Version service parameter names a version the agent does not support).
    VersionNotSupported {
        /// Why it was refused: `a2a.tasks.VersionNotSupported`.
        error: VersionNotSupported,
    },
    /// `extension-required` — externally decided (the agent card marks an extension required and the request does not declare it).
    ExtensionRequired {
        /// Why it was refused: `a2a.tasks.ExtensionSupportRequired`.
        error: ExtensionSupportRequired,
    },
    /// `streaming-not-supported` — externally decided (the agent card does not declare capabilities.streaming).
    StreamingNotSupported {
        /// Why it was refused: `a2a.tasks.UnsupportedOperation`.
        error: UnsupportedOperation,
    },
}

/// Start working — the input of `a2a.tasks.StartWorking`.
///
/// The agent works on the task (TASK_STATE_WORKING).
///
/// Everything it can result in is [`StartWorkingOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartWorking {
    /// `id` — `a2a.tasks.TaskId`.
    pub id: TaskId,
    /// `status_message` — `Optional<a2a.tasks.Message>`.
    pub status_message: Option<Message>,
}

/// Everything `a2a.tasks.StartWorking` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartWorkingOutcome {
    /// `moved` — otherwise.
    Moved {
        /// The `a2a.tasks.TaskStatusUpdated` this outcome publishes.
        task_status_updated: TaskStatusUpdated,
    },
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `wrong-state` — from a state no declared move starts in.
    WrongState {
        /// Why it was refused: `a2a.tasks.TaskStateConflict`.
        error: TaskStateConflict,
    },
}

/// Subscribe to task — the input of `a2a.tasks.SubscribeToTask`.
///
/// Open a stream of a task's updates, starting with the task itself (A2A v1.0.1 section 3.1.6; GET /tasks/{id}:subscribe).
///
/// Everything it can result in is [`SubscribeToTaskOutcome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeToTask {
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
    /// `id` — `a2a.tasks.TaskId`.
    pub id: TaskId,
}

/// Actual typed response of `a2a.tasks.SubscribeToTask`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeToTaskResponse {
    /// `task` — `a2a.tasks.TaskSnapshot`.
    pub task: TaskSnapshot,
}

/// Everything `a2a.tasks.SubscribeToTask` can result in — one variant per declared outcome.
///
/// An infrastructure failure is deliberately not in here: a refusal is a fact about the domain,
/// a transport fault is a fact about the run, and conflating the two is what the declared
/// outcomes exist to prevent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscribeToTaskOutcome {
    /// `not-found` — for an identity no record carries.
    NotFound {
        /// Why it was refused: `a2a.tasks.TaskNotFound`.
        error: TaskNotFound,
    },
    /// `task-terminal` — when the existing subject's stored fields satisfy `state in [Completed, Failed, Canceled, Rejected]`.
    TaskTerminal {
        /// Why it was refused: `a2a.tasks.UnsupportedOperation`.
        error: UnsupportedOperation,
    },
    /// `subscribed` — otherwise.
    Subscribed {
        /// Actual response returned by this branch.
        response: SubscribeToTaskResponse,
    },
    /// `version-not-supported` — externally decided (the A2A-Version service parameter names a version the agent does not support).
    VersionNotSupported {
        /// Why it was refused: `a2a.tasks.VersionNotSupported`.
        error: VersionNotSupported,
    },
    /// `extension-required` — externally decided (the agent card marks an extension required and the request does not declare it).
    ExtensionRequired {
        /// Why it was refused: `a2a.tasks.ExtensionSupportRequired`.
        error: ExtensionSupportRequired,
    },
    /// `streaming-not-supported` — externally decided (the agent card does not declare capabilities.streaming).
    StreamingNotSupported {
        /// Why it was refused: `a2a.tasks.UnsupportedOperation`.
        error: UnsupportedOperation,
    },
}

/// PushNotificationConfigCreated — the event `a2a.tasks.PushNotificationConfigCreated`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushNotificationConfigCreated {
    /// `config_id` — `a2a.tasks.PushNotificationConfigId`.
    pub config_id: PushNotificationConfigId,
    /// `task_id` — `a2a.tasks.TaskId`.
    pub task_id: TaskId,
}

/// PushNotificationConfigDeleted — the event `a2a.tasks.PushNotificationConfigDeleted`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushNotificationConfigDeleted {
    /// `config_id` — `a2a.tasks.PushNotificationConfigId`.
    pub config_id: PushNotificationConfigId,
    /// `task_id` — `a2a.tasks.TaskId`.
    pub task_id: TaskId,
}

/// TaskCreated — the event `a2a.tasks.TaskCreated`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskCreated {
    /// `task_id` — `a2a.tasks.TaskId`.
    pub task_id: TaskId,
}

/// TaskStatusUpdated — the event `a2a.tasks.TaskStatusUpdated`.
///
/// Emitted on every lifecycle move; streamed and pushed as TaskStatusUpdateEvent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskStatusUpdated {
    /// `task_id` — `a2a.tasks.TaskId`.
    pub task_id: TaskId,
    /// `context_id` — `a2a.tasks.ContextId`.
    pub context_id: ContextId,
}

/// The declared error `a2a.tasks.ContentTypeNotSupported`.
///
/// ContentTypeNotSupportedError (JSON-RPC -32005, HTTP 400).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentTypeNotSupported;

/// The declared error `a2a.tasks.ContextMismatch`.
///
/// The message's contextId differs from the contextId of the task it names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextMismatch;

/// The declared error `a2a.tasks.ExtensionSupportRequired`.
///
/// ExtensionSupportRequiredError (JSON-RPC -32008, HTTP 400).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionSupportRequired;

/// The declared error `a2a.tasks.PushNotificationNotSupported`.
///
/// PushNotificationNotSupportedError (JSON-RPC -32003, HTTP 400): the agent card does not declare capabilities.pushNotifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushNotificationNotSupported;

/// The declared error `a2a.tasks.TaskNotCancelable`.
///
/// TaskNotCancelableError (JSON-RPC -32002, HTTP 400).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskNotCancelable;

/// The declared error `a2a.tasks.TaskNotFound`.
///
/// TaskNotFoundError (JSON-RPC -32001, HTTP 404): the task, or the push-notification configuration, does not exist or is not accessible to the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskNotFound;

/// The declared error `a2a.tasks.TaskStateConflict`.
///
/// The task is not in a state this agent-side move starts from, so nothing moved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskStateConflict {
    /// `state` — `a2a.tasks.Task.State`.
    pub state: TaskState,
}

/// The declared error `a2a.tasks.UnsupportedOperation`.
///
/// UnsupportedOperationError (JSON-RPC -32004, HTTP 400).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedOperation;

/// The declared error `a2a.tasks.VersionNotSupported`.
///
/// VersionNotSupportedError (JSON-RPC -32009, HTTP 400).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionNotSupported;

/// All push notification configs — one row of the view `a2a.tasks.PushNotificationConfigs`.
///
/// Projects `a2a.tasks.PushNotificationConfig` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushNotificationConfigs {
    /// `id` — `a2a.tasks.PushNotificationConfigId`.
    pub id: PushNotificationConfigId,
    /// `state` — `a2a.tasks.PushNotificationConfig.State`.
    pub state: PushNotificationConfigState,
    /// `task_id` — `a2a.tasks.TaskId`.
    pub task_id: TaskId,
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
    /// `url` — `String`.
    pub url: String,
    /// `token` — `Optional<String>`.
    pub token: Option<String>,
    /// `authentication` — `Optional<a2a.tasks.AuthenticationInfo>`.
    pub authentication: Option<AuthenticationInfo>,
}

/// List push notification configs — one row of the view `a2a.tasks.TaskPushNotificationConfigs`.
///
/// Projects `a2a.tasks.PushNotificationConfig` at `read_your_writes` consistency, containing instances where `task_id == param.task_id`.
/// Serving it is an implementation obligation — see the plan — because how a projection is kept
/// current is a storage decision the specification does not take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskPushNotificationConfigs {
    /// `id` — `a2a.tasks.PushNotificationConfigId`.
    pub id: PushNotificationConfigId,
    /// `task_id` — `a2a.tasks.TaskId`.
    pub task_id: TaskId,
    /// `tenant` — `Optional<String>`.
    pub tenant: Option<String>,
    /// `url` — `String`.
    pub url: String,
    /// `token` — `Optional<String>`.
    pub token: Option<String>,
    /// `authentication` — `Optional<a2a.tasks.AuthenticationInfo>`.
    pub authentication: Option<AuthenticationInfo>,
}

/// List tasks — one row of the view `a2a.tasks.Tasks`.
///
/// Projects `a2a.tasks.Task` at `read_your_writes` consistency.
/// The specification fully determines every row, so its query is generated over the storage port —
/// see the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tasks {
    /// `id` — `a2a.tasks.TaskId`.
    pub id: TaskId,
    /// `context_id` — `a2a.tasks.ContextId`.
    pub context_id: ContextId,
    /// `state` — `a2a.tasks.Task.State`.
    pub state: TaskState,
    /// `status_message` — `Optional<a2a.tasks.Message>`.
    pub status_message: Option<Message>,
    /// `status_timestamp` — `Optional<Timestamp>`.
    pub status_timestamp: Option<crate::primitives::Timestamp>,
    /// `artifacts` — `Optional<List<a2a.tasks.Artifact>>`.
    pub artifacts: Option<Vec<Artifact>>,
    /// `history` — `Optional<List<a2a.tasks.Message>>`.
    pub history: Option<Vec<Message>>,
    /// `metadata` — `Optional<Json>`.
    pub metadata: Option<crate::json::Value>,
}

/// What this bounded context owes its implementor, and the seams of what is generated.
///
/// One trait per obligation in the synthesis plan, each carrying the plan's own contract, and one
/// per generated behaviour, which [`Generated`](crate::behaviour::Generated) implements.
/// [`Unimplemented`](obligations::Unimplemented) satisfies every owed trait by refusing in the type system.
pub mod obligations {
    /// The behaviour `a2a.tasks.CancelTask` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a typed response (`response:`).
    ///
    /// Contract: given `a2a.tasks.CancelTask` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `not-found` for an identity no record carries, error `a2a.tasks.TaskNotFound`; `not-cancelable-now` externally decided (the agent cannot cancel the task at its current stage), error `a2a.tasks.TaskNotCancelable`; `canceled` otherwise, takes `cancel` of `a2a.tasks.Task`, emits `a2a.tasks.TaskStatusUpdated`; `not-cancelable` from a state no declared move starts in, error `a2a.tasks.TaskNotCancelable`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired`.
    pub trait CancelTaskBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.CancelTask`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn cancel_task(&mut self, input: super::CancelTask) -> Result<super::CancelTaskOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `a2a.tasks.CompleteTask` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait CompleteTaskBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.CompleteTask`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn complete_task(&mut self, input: super::CompleteTask) -> Result<super::CompleteTaskOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `a2a.tasks.CreateTaskPushNotificationConfig` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a typed response (`response:`).
    ///
    /// Contract: given `a2a.tasks.CreateTaskPushNotificationConfig` input, decide and enact exactly one outcome. Selection precedence: on commands with `when_related:`, check `existing_instance` then `exists: false` before input-guarded refusals; choose the first declared input refusal whose guard holds; then check addressed-row existence (`unknown_instance`, and `existing_instance` on commands without `when_related:`); then the held state (`when_subject_state` and `when_subject`), with `wrong_state` only if the selected branch moves from a state the row does not hold; then accepting and external branches in declaration order. An accepting branch that moves nothing answers in every state. Related-presence predicates do not precede input-guarded refusals. Declared outcomes (declaration order, not selection precedence): `task-not-found` when no `a2a.tasks.Task` carries the identity `input.task_id` names, error `a2a.tasks.TaskNotFound`; `created` otherwise, creates `a2a.tasks.PushNotificationConfig`, emits `a2a.tasks.PushNotificationConfigCreated`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired`; `push-not-supported` externally decided (the agent card does not declare capabilities.pushNotifications), error `a2a.tasks.PushNotificationNotSupported`.
    pub trait CreateTaskPushNotificationConfigBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.CreateTaskPushNotificationConfig`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn create_task_push_notification_config(&mut self, input: super::CreateTaskPushNotificationConfig) -> Result<super::CreateTaskPushNotificationConfigOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `a2a.tasks.DeleteTaskPushNotificationConfig` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by `external:` beside `when_related:` in one command.
    ///
    /// Contract: given `a2a.tasks.DeleteTaskPushNotificationConfig` input, decide and enact exactly one outcome. Selection precedence: on commands with `when_related:`, check `existing_instance` then `exists: false` before input-guarded refusals; choose the first declared input refusal whose guard holds; then check addressed-row existence (`unknown_instance`, and `existing_instance` on commands without `when_related:`); then the held state (`when_subject_state` and `when_subject`), with `wrong_state` only if the selected branch moves from a state the row does not hold; then accepting and external branches in declaration order. An accepting branch that moves nothing answers in every state. Related-presence predicates do not precede input-guarded refusals. Declared outcomes (declaration order, not selection precedence): `task-not-found` when no `a2a.tasks.Task` carries the identity `input.task_id` names, error `a2a.tasks.TaskNotFound`; `already-gone` for an identity no record carries; `deleted` otherwise, deletes `a2a.tasks.PushNotificationConfig`, emits `a2a.tasks.PushNotificationConfigDeleted`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired`; `push-not-supported` externally decided (the agent card does not declare capabilities.pushNotifications), error `a2a.tasks.PushNotificationNotSupported`.
    pub trait DeleteTaskPushNotificationConfigBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.DeleteTaskPushNotificationConfig`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn delete_task_push_notification_config(&mut self, input: super::DeleteTaskPushNotificationConfig) -> Result<super::DeleteTaskPushNotificationConfigOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `a2a.tasks.FailTask` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait FailTaskBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.FailTask`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn fail_task(&mut self, input: super::FailTask) -> Result<super::FailTaskOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `a2a.tasks.GetTask` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a typed response (`response:`).
    ///
    /// Contract: given `a2a.tasks.GetTask` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `not-found` for an identity no record carries, error `a2a.tasks.TaskNotFound`; `found` otherwise, preserves `a2a.tasks.Task`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired`.
    pub trait GetTaskBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.GetTask`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn get_task(&mut self, input: super::GetTask) -> Result<super::GetTaskOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `a2a.tasks.GetTaskPushNotificationConfig` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a typed response (`response:`).
    ///
    /// Contract: given `a2a.tasks.GetTaskPushNotificationConfig` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `not-found` for an identity no record carries, error `a2a.tasks.TaskNotFound`; `other-task` when the existing subject's stored fields satisfy `task_id != input.task_id`, error `a2a.tasks.TaskNotFound`; `found` otherwise, preserves `a2a.tasks.PushNotificationConfig`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired`; `push-not-supported` externally decided (the agent card does not declare capabilities.pushNotifications), error `a2a.tasks.PushNotificationNotSupported`.
    pub trait GetTaskPushNotificationConfigBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.GetTaskPushNotificationConfig`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn get_task_push_notification_config(&mut self, input: super::GetTaskPushNotificationConfig) -> Result<super::GetTaskPushNotificationConfigOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `a2a.tasks.RejectTask` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait RejectTaskBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.RejectTask`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn reject_task(&mut self, input: super::RejectTask) -> Result<super::RejectTaskOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `a2a.tasks.RequireAuth` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait RequireAuthBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.RequireAuth`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn require_auth(&mut self, input: super::RequireAuth) -> Result<super::RequireAuthOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `a2a.tasks.RequireInput` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait RequireInputBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.RequireInput`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn require_input(&mut self, input: super::RequireInput) -> Result<super::RequireInputOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `a2a.tasks.SendMessage` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a typed response (`response:`).
    ///
    /// Contract: given `a2a.tasks.SendMessage` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `task-not-found` externally decided (message.taskId names no task the caller can access (section 3.4.2)), error `a2a.tasks.TaskNotFound`; `task-terminal` externally decided (message.taskId names a task in Completed, Failed, Canceled or Rejected), error `a2a.tasks.UnsupportedOperation`; `context-mismatch` externally decided (message.contextId differs from the contextId of the task message.taskId names), error `a2a.tasks.ContextMismatch`; `content-type-not-supported` externally decided (a part's media type is not supported by the agent), error `a2a.tasks.ContentTypeNotSupported`; `continued` externally decided (message.taskId names a non-terminal task, and the agent takes the message into it); `direct-message` externally decided (the agent answers with a message and creates no task); `task-created` otherwise, creates `a2a.tasks.Task`, emits `a2a.tasks.TaskCreated`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired`.
    pub trait SendMessageBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.SendMessage`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn send_message(&mut self, input: super::SendMessage) -> Result<super::SendMessageOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `a2a.tasks.SendStreamingMessage` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a typed response (`response:`).
    ///
    /// Contract: given `a2a.tasks.SendStreamingMessage` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `task-not-found` externally decided (message.taskId names no task the caller can access (section 3.4.2)), error `a2a.tasks.TaskNotFound`; `task-terminal` externally decided (message.taskId names a task in Completed, Failed, Canceled or Rejected), error `a2a.tasks.UnsupportedOperation`; `context-mismatch` externally decided (message.contextId differs from the contextId of the task message.taskId names), error `a2a.tasks.ContextMismatch`; `content-type-not-supported` externally decided (a part's media type is not supported by the agent), error `a2a.tasks.ContentTypeNotSupported`; `continued` externally decided (message.taskId names a non-terminal task, and the agent takes the message into it); `direct-message` externally decided (the agent answers with a message and creates no task); `task-created` otherwise, creates `a2a.tasks.Task`, emits `a2a.tasks.TaskCreated`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired`; `streaming-not-supported` externally decided (the agent card does not declare capabilities.streaming), error `a2a.tasks.UnsupportedOperation`.
    pub trait SendStreamingMessageBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.SendStreamingMessage`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn send_streaming_message(&mut self, input: super::SendStreamingMessage) -> Result<super::SendStreamingMessageOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `a2a.tasks.StartWorking` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage and context ports. Implement it yourself to replace that behaviour.
    pub trait StartWorkingBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.StartWorking`.
        ///
        /// `Err` is the typed refusal of a request the model declares no outcome for.
        fn start_working(&mut self, input: super::StartWorking) -> Result<super::StartWorkingOutcome, crate::obligation::UnmetObligation>;
    }

    /// The behaviour `a2a.tasks.SubscribeToTask` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a typed response (`response:`).
    ///
    /// Contract: given `a2a.tasks.SubscribeToTask` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `not-found` for an identity no record carries, error `a2a.tasks.TaskNotFound`; `task-terminal` when the existing subject's stored fields satisfy `state in [Completed, Failed, Canceled, Rejected]`, error `a2a.tasks.UnsupportedOperation`; `subscribed` otherwise, preserves `a2a.tasks.Task`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired`; `streaming-not-supported` externally decided (the agent card does not declare capabilities.streaming), error `a2a.tasks.UnsupportedOperation`.
    pub trait SubscribeToTaskBehavior {
        /// Decides and enacts exactly one declared outcome of `a2a.tasks.SubscribeToTask`.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn subscribe_to_task(&mut self, input: super::SubscribeToTask) -> Result<super::SubscribeToTaskOutcome, crate::obligation::UnmetObligation>;
    }

    /// The query `a2a.tasks.PushNotificationConfigs` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait PushNotificationConfigsQuery {
        /// Serves `a2a.tasks.PushNotificationConfigs` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn push_notification_configs(&self) -> Result<Vec<super::PushNotificationConfigs>, crate::obligation::UnmetObligation>;
    }

    /// The query `a2a.tasks.TaskPushNotificationConfigs` — an implementation obligation.
    ///
    /// Why it is not generated: kept an obligation by a view parameter (`params:`), which a generated query does not apply.
    ///
    /// Contract: a query answering `a2a.tasks.TaskPushNotificationConfigs` with rows projected from `a2a.tasks.PushNotificationConfig` at `read_your_writes` consistency, containing instances where `task_id == param.task_id`.
    pub trait TaskPushNotificationConfigsQuery {
        /// Serves `a2a.tasks.TaskPushNotificationConfigs` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of an obligation nothing has satisfied; a satisfying
        /// implementation never returns it.
        fn task_push_notification_configs(&self, task_id: crate::tasks::TaskId) -> Result<Vec<super::TaskPushNotificationConfigs>, crate::obligation::UnmetObligation>;
    }

    /// The query `a2a.tasks.Tasks` — generated.
    ///
    /// The specification fully determines it: [`crate::behaviour::Generated`] implements it
    /// over the storage port. Implement it yourself to replace that query.
    pub trait TasksQuery {
        /// Serves `a2a.tasks.Tasks` rows at the view's declared consistency.
        ///
        /// `Err` is the typed refusal of a row whose declared type cannot hold its value.
        fn tasks(&self) -> Result<Vec<super::Tasks>, crate::obligation::UnmetObligation>;
    }

    /// Every obligation of this bounded context, refused in the type system.
    ///
    /// Each method returns the typed refusal naming what is owed — never a panic, never a guessed
    /// value — so a workspace built on this stub compiles and reports its own gaps.
    pub struct Unimplemented;

    impl CancelTaskBehavior for Unimplemented {
        fn cancel_task(&mut self, _input: super::CancelTask) -> Result<super::CancelTaskOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "a2a.tasks.CancelTask" })
        }
    }

    impl CreateTaskPushNotificationConfigBehavior for Unimplemented {
        fn create_task_push_notification_config(&mut self, _input: super::CreateTaskPushNotificationConfig) -> Result<super::CreateTaskPushNotificationConfigOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "a2a.tasks.CreateTaskPushNotificationConfig" })
        }
    }

    impl DeleteTaskPushNotificationConfigBehavior for Unimplemented {
        fn delete_task_push_notification_config(&mut self, _input: super::DeleteTaskPushNotificationConfig) -> Result<super::DeleteTaskPushNotificationConfigOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "a2a.tasks.DeleteTaskPushNotificationConfig" })
        }
    }

    impl GetTaskBehavior for Unimplemented {
        fn get_task(&mut self, _input: super::GetTask) -> Result<super::GetTaskOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "a2a.tasks.GetTask" })
        }
    }

    impl GetTaskPushNotificationConfigBehavior for Unimplemented {
        fn get_task_push_notification_config(&mut self, _input: super::GetTaskPushNotificationConfig) -> Result<super::GetTaskPushNotificationConfigOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "a2a.tasks.GetTaskPushNotificationConfig" })
        }
    }

    impl SendMessageBehavior for Unimplemented {
        fn send_message(&mut self, _input: super::SendMessage) -> Result<super::SendMessageOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "a2a.tasks.SendMessage" })
        }
    }

    impl SendStreamingMessageBehavior for Unimplemented {
        fn send_streaming_message(&mut self, _input: super::SendStreamingMessage) -> Result<super::SendStreamingMessageOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "a2a.tasks.SendStreamingMessage" })
        }
    }

    impl SubscribeToTaskBehavior for Unimplemented {
        fn subscribe_to_task(&mut self, _input: super::SubscribeToTask) -> Result<super::SubscribeToTaskOutcome, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "command behaviour", source: "a2a.tasks.SubscribeToTask" })
        }
    }

    impl TaskPushNotificationConfigsQuery for Unimplemented {
        fn task_push_notification_configs(&self, _task_id: crate::tasks::TaskId) -> Result<Vec<super::TaskPushNotificationConfigs>, crate::obligation::UnmetObligation> {
            Err(crate::obligation::UnmetObligation { capability: "view query", source: "a2a.tasks.TaskPushNotificationConfigs" })
        }
    }
}
