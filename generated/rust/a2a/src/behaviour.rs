// generated from a2a v1
// model digest d4cf404c98ec99676cdd8fe8e411c38768d3acb9560e07676018a3ce6a70e28d
// contract digest f51f3ef221113fd930cbd423a63039558132d9f28674116bc714692bd0a5ad44
// do not edit: regenerate with `ess synthesize --layout crate`

//! What the specification fully determines, generated: the behaviour of every command the plan
//! lists as generated, written against ports the implementor supplies.
//!
//! Storage is a port: one trait per entity, get, put and delete of a snapshot by identity. ess
//! preserves the trait; generated network entries supply an ephemeral store. `Context` carries the caller's attributes,
//! every identity and value the model says the implementation assigns, and the answer to each
//! `external:` branch. [`Generated`] implements every generated `…Behavior` trait over those ports
//! and forwards every behaviour and query the plan still owes to them, so it is a complete bundle
//! for every component port. To replace one generated behaviour, write a bundle of your own that
//! implements that trait and delegates the rest to a `Generated`.
//!
//! An `Err` from a generated behaviour is the typed refusal naming the command: the model declares
//! no outcome for the request (a guard is undecidable over it, or no declared branch answers it),
//! or — as `entity invariant` — the declared outcome would leave an entity breaking an invariant.

use crate::obligation::UnmetObligation;

/// Where `a2a.tasks.PushNotificationConfig` is stored — a port the implementor provides.
///
/// Keyed by the identity `row`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait PushNotificationConfigStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::tasks::PushNotificationConfigRow) -> Option<crate::tasks::PushNotificationConfigSnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::tasks::PushNotificationConfigSnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::tasks::PushNotificationConfigRow);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::tasks::PushNotificationConfigSnapshot>;
}

/// Where `a2a.tasks.Task` is stored — a port the implementor provides.
///
/// Keyed by the identity `id`. Generated network entries supply an ephemeral implementation; durable storage remains a port.
pub trait TaskStorage {
    /// The instance with this identity, or `None` where none is stored.
    fn get(&self, identity: &crate::tasks::TaskId) -> Option<crate::tasks::TaskEntitySnapshot>;

    /// Stores this instance under its identity, replacing what was held.
    fn put(&mut self, snapshot: crate::tasks::TaskEntitySnapshot);

    /// Removes the instance with this identity.
    fn delete(&mut self, identity: &crate::tasks::TaskId);

    /// Every stored instance, in the order the store keeps them: the order a generated query
    /// answers an unordered view in.
    fn list(&self) -> Vec<crate::tasks::TaskEntitySnapshot>;
}

/// What the specification leaves to the implementor's context — a port the implementor provides.
///
/// The caller's attributes, the values the model says the implementation assigns, and the answer
/// to each `external:` branch.
pub trait Context {
    /// A `Optional<Timestamp>` the model says the implementation assigns — a `{generated: true}` value —
    /// or `None` where there is nothing to report.
    fn generate_optional_timestamp(&mut self) -> Option<crate::primitives::Timestamp>;
}

/// Context answers that may be unavailable, without fabricated values.
/// Existing `Context` implementations receive the blanket adapter.
pub trait TryContext {
/// Assigns the value, or names the unavailable answer.
fn try_generate_optional_timestamp(&mut self) -> Result<Option<crate::primitives::Timestamp>, UnmetObligation>;
}

impl<T: Context + ?Sized> TryContext for T {
fn try_generate_optional_timestamp(&mut self) -> Result<Option<crate::primitives::Timestamp>, UnmetObligation> { Ok(Context::generate_optional_timestamp(self)) }
}

/// An unavailable runtime context answer, rather than a new planned capability.
pub fn unmet_context(source: &'static str) -> UnmetObligation { UnmetObligation { capability: "context answer", source } }

/// Every generated behaviour of this workspace, over the ports `P` supplies.
///
/// `P` implements the storage trait of each entity a generated behaviour reads or writes,
/// `TryContext` (or its legacy `Context` blanket adapter) where one asks it anything, and every `…Behavior` and `…Query` trait the plan still
/// owes; `Generated<P>` forwards those to it.
pub struct Generated<P> {
    /// The storage and context ports, and every behaviour or query still owed.
    pub ports: P,
}

impl<P> Generated<P> {
    /// The generated behaviours, over `ports`.
    pub fn new(ports: P) -> Self {
        Self { ports }
    }
}

impl<P: crate::discovery::obligations::GetExtendedAgentCardBehavior> crate::discovery::obligations::GetExtendedAgentCardBehavior for Generated<P> {
    fn get_extended_agent_card(&mut self, input: crate::discovery::GetExtendedAgentCard) -> Result<crate::discovery::GetExtendedAgentCardOutcome, UnmetObligation> {
        crate::discovery::obligations::GetExtendedAgentCardBehavior::get_extended_agent_card(&mut self.ports, input)
    }
}

impl<P: crate::tasks::obligations::CancelTaskBehavior> crate::tasks::obligations::CancelTaskBehavior for Generated<P> {
    fn cancel_task(&mut self, input: crate::tasks::CancelTask) -> Result<crate::tasks::CancelTaskOutcome, UnmetObligation> {
        crate::tasks::obligations::CancelTaskBehavior::cancel_task(&mut self.ports, input)
    }
}

/// `a2a.tasks.CompleteTask`, generated: every outcome is one the specification fully determines.
impl<P> crate::tasks::obligations::CompleteTaskBehavior for Generated<P>
where
    P: TryContext + TaskStorage,
{
    fn complete_task(&mut self, input: crate::tasks::CompleteTask) -> Result<crate::tasks::CompleteTaskOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row's existence, at step 2 of the one precedence rule.
        if TaskStorage::get(&self.ports, &input.id).is_none() {
            return Ok(crate::tasks::CompleteTaskOutcome::NotFound { error: crate::tasks::TaskNotFound });
        }
        // `moved`: the default.
        let Some(held) = TaskStorage::get(&self.ports, &input.id) else {
            return Ok(crate::tasks::CompleteTaskOutcome::NotFound { error: crate::tasks::TaskNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let before = held.data.clone();
        let moved = match held.refine() {
            crate::tasks::AnyTask::AuthRequired(instance) => crate::tasks::AnyTask::Completed(instance.complete()),
            crate::tasks::AnyTask::InputRequired(instance) => crate::tasks::AnyTask::Completed(instance.complete()),
            crate::tasks::AnyTask::Submitted(instance) => crate::tasks::AnyTask::Completed(instance.complete()),
            crate::tasks::AnyTask::Working(instance) => crate::tasks::AnyTask::Completed(instance.complete()),
            _ => return Ok(crate::tasks::CompleteTaskOutcome::WrongState { error: crate::tasks::TaskStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.status_message = input.status_message.clone();
        next.data.status_timestamp = self.ports.try_generate_optional_timestamp()?;
        let answer = crate::tasks::CompleteTaskOutcome::Moved { task_status_updated: crate::tasks::TaskStatusUpdated { task_id: input.id.clone(), context_id: before.context_id.clone(), status: crate::tasks::TaskStatus { state: crate::tasks::WireTaskState::TASKSTATECOMPLETED, message: input.status_message.clone(), timestamp: self.ports.try_generate_optional_timestamp()? } } };
        TaskStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

impl<P: crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior> crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior for Generated<P> {
    fn create_task_push_notification_config(&mut self, input: crate::tasks::CreateTaskPushNotificationConfig) -> Result<crate::tasks::CreateTaskPushNotificationConfigOutcome, UnmetObligation> {
        crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior::create_task_push_notification_config(&mut self.ports, input)
    }
}

impl<P: crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior> crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior for Generated<P> {
    fn delete_task_push_notification_config(&mut self, input: crate::tasks::DeleteTaskPushNotificationConfig) -> Result<crate::tasks::DeleteTaskPushNotificationConfigOutcome, UnmetObligation> {
        crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior::delete_task_push_notification_config(&mut self.ports, input)
    }
}

/// `a2a.tasks.FailTask`, generated: every outcome is one the specification fully determines.
impl<P> crate::tasks::obligations::FailTaskBehavior for Generated<P>
where
    P: TryContext + TaskStorage,
{
    fn fail_task(&mut self, input: crate::tasks::FailTask) -> Result<crate::tasks::FailTaskOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row's existence, at step 2 of the one precedence rule.
        if TaskStorage::get(&self.ports, &input.id).is_none() {
            return Ok(crate::tasks::FailTaskOutcome::NotFound { error: crate::tasks::TaskNotFound });
        }
        // `moved`: the default.
        let Some(held) = TaskStorage::get(&self.ports, &input.id) else {
            return Ok(crate::tasks::FailTaskOutcome::NotFound { error: crate::tasks::TaskNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let before = held.data.clone();
        let moved = match held.refine() {
            crate::tasks::AnyTask::AuthRequired(instance) => crate::tasks::AnyTask::Failed(instance.fail()),
            crate::tasks::AnyTask::InputRequired(instance) => crate::tasks::AnyTask::Failed(instance.fail()),
            crate::tasks::AnyTask::Submitted(instance) => crate::tasks::AnyTask::Failed(instance.fail()),
            crate::tasks::AnyTask::Working(instance) => crate::tasks::AnyTask::Failed(instance.fail()),
            _ => return Ok(crate::tasks::FailTaskOutcome::WrongState { error: crate::tasks::TaskStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.status_message = input.status_message.clone();
        next.data.status_timestamp = self.ports.try_generate_optional_timestamp()?;
        let answer = crate::tasks::FailTaskOutcome::Moved { task_status_updated: crate::tasks::TaskStatusUpdated { task_id: input.id.clone(), context_id: before.context_id.clone(), status: crate::tasks::TaskStatus { state: crate::tasks::WireTaskState::TASKSTATEFAILED, message: input.status_message.clone(), timestamp: self.ports.try_generate_optional_timestamp()? } } };
        TaskStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

impl<P: crate::tasks::obligations::GetTaskBehavior> crate::tasks::obligations::GetTaskBehavior for Generated<P> {
    fn get_task(&mut self, input: crate::tasks::GetTask) -> Result<crate::tasks::GetTaskOutcome, UnmetObligation> {
        crate::tasks::obligations::GetTaskBehavior::get_task(&mut self.ports, input)
    }
}

impl<P: crate::tasks::obligations::GetTaskPushNotificationConfigBehavior> crate::tasks::obligations::GetTaskPushNotificationConfigBehavior for Generated<P> {
    fn get_task_push_notification_config(&mut self, input: crate::tasks::GetTaskPushNotificationConfig) -> Result<crate::tasks::GetTaskPushNotificationConfigOutcome, UnmetObligation> {
        crate::tasks::obligations::GetTaskPushNotificationConfigBehavior::get_task_push_notification_config(&mut self.ports, input)
    }
}

impl<P: crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior> crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior for Generated<P> {
    fn list_task_push_notification_configs(&mut self, input: crate::tasks::ListTaskPushNotificationConfigs) -> Result<crate::tasks::ListTaskPushNotificationConfigsOutcome, UnmetObligation> {
        crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior::list_task_push_notification_configs(&mut self.ports, input)
    }
}

/// `a2a.tasks.RejectTask`, generated: every outcome is one the specification fully determines.
impl<P> crate::tasks::obligations::RejectTaskBehavior for Generated<P>
where
    P: TryContext + TaskStorage,
{
    fn reject_task(&mut self, input: crate::tasks::RejectTask) -> Result<crate::tasks::RejectTaskOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row's existence, at step 2 of the one precedence rule.
        if TaskStorage::get(&self.ports, &input.id).is_none() {
            return Ok(crate::tasks::RejectTaskOutcome::NotFound { error: crate::tasks::TaskNotFound });
        }
        // `moved`: the default.
        let Some(held) = TaskStorage::get(&self.ports, &input.id) else {
            return Ok(crate::tasks::RejectTaskOutcome::NotFound { error: crate::tasks::TaskNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let before = held.data.clone();
        let moved = match held.refine() {
            crate::tasks::AnyTask::AuthRequired(instance) => crate::tasks::AnyTask::Rejected(instance.reject()),
            crate::tasks::AnyTask::InputRequired(instance) => crate::tasks::AnyTask::Rejected(instance.reject()),
            crate::tasks::AnyTask::Submitted(instance) => crate::tasks::AnyTask::Rejected(instance.reject()),
            crate::tasks::AnyTask::Working(instance) => crate::tasks::AnyTask::Rejected(instance.reject()),
            _ => return Ok(crate::tasks::RejectTaskOutcome::WrongState { error: crate::tasks::TaskStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.status_message = input.status_message.clone();
        next.data.status_timestamp = self.ports.try_generate_optional_timestamp()?;
        let answer = crate::tasks::RejectTaskOutcome::Moved { task_status_updated: crate::tasks::TaskStatusUpdated { task_id: input.id.clone(), context_id: before.context_id.clone(), status: crate::tasks::TaskStatus { state: crate::tasks::WireTaskState::TASKSTATEREJECTED, message: input.status_message.clone(), timestamp: self.ports.try_generate_optional_timestamp()? } } };
        TaskStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `a2a.tasks.RequireAuth`, generated: every outcome is one the specification fully determines.
impl<P> crate::tasks::obligations::RequireAuthBehavior for Generated<P>
where
    P: TryContext + TaskStorage,
{
    fn require_auth(&mut self, input: crate::tasks::RequireAuth) -> Result<crate::tasks::RequireAuthOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row's existence, at step 2 of the one precedence rule.
        if TaskStorage::get(&self.ports, &input.id).is_none() {
            return Ok(crate::tasks::RequireAuthOutcome::NotFound { error: crate::tasks::TaskNotFound });
        }
        // `moved`: the default.
        let Some(held) = TaskStorage::get(&self.ports, &input.id) else {
            return Ok(crate::tasks::RequireAuthOutcome::NotFound { error: crate::tasks::TaskNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let before = held.data.clone();
        let moved = match held.refine() {
            crate::tasks::AnyTask::InputRequired(instance) => crate::tasks::AnyTask::AuthRequired(instance.require_auth()),
            crate::tasks::AnyTask::Submitted(instance) => crate::tasks::AnyTask::AuthRequired(instance.require_auth()),
            crate::tasks::AnyTask::Working(instance) => crate::tasks::AnyTask::AuthRequired(instance.require_auth()),
            _ => return Ok(crate::tasks::RequireAuthOutcome::WrongState { error: crate::tasks::TaskStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.status_message = input.status_message.clone();
        next.data.status_timestamp = self.ports.try_generate_optional_timestamp()?;
        let answer = crate::tasks::RequireAuthOutcome::Moved { task_status_updated: crate::tasks::TaskStatusUpdated { task_id: input.id.clone(), context_id: before.context_id.clone(), status: crate::tasks::TaskStatus { state: crate::tasks::WireTaskState::TASKSTATEAUTHREQUIRED, message: input.status_message.clone(), timestamp: self.ports.try_generate_optional_timestamp()? } } };
        TaskStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `a2a.tasks.RequireInput`, generated: every outcome is one the specification fully determines.
impl<P> crate::tasks::obligations::RequireInputBehavior for Generated<P>
where
    P: TryContext + TaskStorage,
{
    fn require_input(&mut self, input: crate::tasks::RequireInput) -> Result<crate::tasks::RequireInputOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row's existence, at step 2 of the one precedence rule.
        if TaskStorage::get(&self.ports, &input.id).is_none() {
            return Ok(crate::tasks::RequireInputOutcome::NotFound { error: crate::tasks::TaskNotFound });
        }
        // `moved`: the default.
        let Some(held) = TaskStorage::get(&self.ports, &input.id) else {
            return Ok(crate::tasks::RequireInputOutcome::NotFound { error: crate::tasks::TaskNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let before = held.data.clone();
        let moved = match held.refine() {
            crate::tasks::AnyTask::AuthRequired(instance) => crate::tasks::AnyTask::InputRequired(instance.require_input()),
            crate::tasks::AnyTask::Submitted(instance) => crate::tasks::AnyTask::InputRequired(instance.require_input()),
            crate::tasks::AnyTask::Working(instance) => crate::tasks::AnyTask::InputRequired(instance.require_input()),
            _ => return Ok(crate::tasks::RequireInputOutcome::WrongState { error: crate::tasks::TaskStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.status_message = input.status_message.clone();
        next.data.status_timestamp = self.ports.try_generate_optional_timestamp()?;
        let answer = crate::tasks::RequireInputOutcome::Moved { task_status_updated: crate::tasks::TaskStatusUpdated { task_id: input.id.clone(), context_id: before.context_id.clone(), status: crate::tasks::TaskStatus { state: crate::tasks::WireTaskState::TASKSTATEINPUTREQUIRED, message: input.status_message.clone(), timestamp: self.ports.try_generate_optional_timestamp()? } } };
        TaskStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

impl<P: crate::tasks::obligations::SendMessageBehavior> crate::tasks::obligations::SendMessageBehavior for Generated<P> {
    fn send_message(&mut self, input: crate::tasks::SendMessage) -> Result<crate::tasks::SendMessageOutcome, UnmetObligation> {
        crate::tasks::obligations::SendMessageBehavior::send_message(&mut self.ports, input)
    }
}

impl<P: crate::tasks::obligations::SendStreamingMessageBehavior> crate::tasks::obligations::SendStreamingMessageBehavior for Generated<P> {
    fn send_streaming_message(&mut self, input: crate::tasks::SendStreamingMessage) -> Result<crate::tasks::SendStreamingMessageOutcome, UnmetObligation> {
        crate::tasks::obligations::SendStreamingMessageBehavior::send_streaming_message(&mut self.ports, input)
    }
}

/// `a2a.tasks.StartWorking`, generated: every outcome is one the specification fully determines.
impl<P> crate::tasks::obligations::StartWorkingBehavior for Generated<P>
where
    P: TryContext + TaskStorage,
{
    fn start_working(&mut self, input: crate::tasks::StartWorking) -> Result<crate::tasks::StartWorkingOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row's existence, at step 2 of the one precedence rule.
        if TaskStorage::get(&self.ports, &input.id).is_none() {
            return Ok(crate::tasks::StartWorkingOutcome::NotFound { error: crate::tasks::TaskNotFound });
        }
        // `moved`: the default.
        let Some(held) = TaskStorage::get(&self.ports, &input.id) else {
            return Ok(crate::tasks::StartWorkingOutcome::NotFound { error: crate::tasks::TaskNotFound });
        };
        let _ = &held;
        let held_state = held.state;
        let before = held.data.clone();
        let moved = match held.refine() {
            crate::tasks::AnyTask::AuthRequired(instance) => crate::tasks::AnyTask::Working(instance.work()),
            crate::tasks::AnyTask::InputRequired(instance) => crate::tasks::AnyTask::Working(instance.work()),
            crate::tasks::AnyTask::Submitted(instance) => crate::tasks::AnyTask::Working(instance.work()),
            _ => return Ok(crate::tasks::StartWorkingOutcome::WrongState { error: crate::tasks::TaskStateConflict { state: held_state } }),
        };
        let mut next = moved.snapshot();
        next.data.status_message = input.status_message.clone();
        next.data.status_timestamp = self.ports.try_generate_optional_timestamp()?;
        let answer = crate::tasks::StartWorkingOutcome::Moved { task_status_updated: crate::tasks::TaskStatusUpdated { task_id: input.id.clone(), context_id: before.context_id.clone(), status: crate::tasks::TaskStatus { state: crate::tasks::WireTaskState::TASKSTATEWORKING, message: input.status_message.clone(), timestamp: self.ports.try_generate_optional_timestamp()? } } };
        TaskStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

impl<P: crate::tasks::obligations::SubscribeToTaskBehavior> crate::tasks::obligations::SubscribeToTaskBehavior for Generated<P> {
    fn subscribe_to_task(&mut self, input: crate::tasks::SubscribeToTask) -> Result<crate::tasks::SubscribeToTaskOutcome, UnmetObligation> {
        crate::tasks::obligations::SubscribeToTaskBehavior::subscribe_to_task(&mut self.ports, input)
    }
}

/// `a2a.tasks.UpdateArtifact`, generated: every outcome is one the specification fully determines.
impl<P> crate::tasks::obligations::UpdateArtifactBehavior for Generated<P>
where
    P: TaskStorage,
{
    fn update_artifact(&mut self, input: crate::tasks::UpdateArtifact) -> Result<crate::tasks::UpdateArtifactOutcome, UnmetObligation> {
        let _ = &input;
        // The addressed row's existence, at step 2 of the one precedence rule.
        if TaskStorage::get(&self.ports, &input.id).is_none() {
            return Ok(crate::tasks::UpdateArtifactOutcome::NotFound { error: crate::tasks::TaskNotFound });
        }
        // `updated`: the default.
        let Some(held) = TaskStorage::get(&self.ports, &input.id) else {
            return Ok(crate::tasks::UpdateArtifactOutcome::NotFound { error: crate::tasks::TaskNotFound });
        };
        let _ = &held;
        let before = held.data.clone();
        let next = held;
        let answer = crate::tasks::UpdateArtifactOutcome::Updated { task_artifact_updated: crate::tasks::TaskArtifactUpdated { task_id: input.id.clone(), context_id: before.context_id.clone(), artifact: input.artifact.clone(), append: Some(input.append.clone()), last_chunk: Some(input.last_chunk.clone()) } };
        TaskStorage::put(&mut self.ports, next);
        return Ok(answer);
    }
}

/// `a2a.tasks.AllTasks`, generated: every row is one the specification fully determines from the stored `a2a.tasks.Task`s.
impl<P> crate::tasks::obligations::AllTasksQuery for Generated<P>
where
    P: TaskStorage,
{
    fn all_tasks(&self) -> Result<Vec<crate::tasks::AllTasks>, UnmetObligation> {
        let admitted = TaskStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::tasks::AllTasks {
                id: held.data.id,
                context_id: held.data.context_id,
                state: held.state,
                status_message: held.data.status_message,
                status_timestamp: held.data.status_timestamp,
                artifacts: held.data.artifacts,
                history: held.data.history,
                metadata: held.data.metadata,
            })
            .collect())
    }
}

/// `a2a.tasks.PushNotificationConfigs`, generated: every row is one the specification fully determines from the stored `a2a.tasks.PushNotificationConfig`s.
impl<P> crate::tasks::obligations::PushNotificationConfigsQuery for Generated<P>
where
    P: PushNotificationConfigStorage,
{
    fn push_notification_configs(&self) -> Result<Vec<crate::tasks::PushNotificationConfigs>, UnmetObligation> {
        let admitted = PushNotificationConfigStorage::list(&self.ports);
        Ok(admitted
            .into_iter()
            .map(|held| crate::tasks::PushNotificationConfigs {
                row: held.data.row,
                config_id: held.data.config_id,
                state: held.state,
                task_id: held.data.task_id,
                tenant: held.data.tenant,
                url: held.data.url,
                token: held.data.token,
                authentication: held.data.authentication,
            })
            .collect())
    }
}

impl<P: crate::tasks::obligations::TaskPushNotificationConfigsQuery> crate::tasks::obligations::TaskPushNotificationConfigsQuery for Generated<P> {
    fn task_push_notification_configs(&self, task_id: crate::tasks::TaskId) -> Result<Vec<crate::tasks::TaskPushNotificationConfigs>, UnmetObligation> {
        crate::tasks::obligations::TaskPushNotificationConfigsQuery::task_push_notification_configs(&self.ports, task_id)
    }
}

impl<P: crate::tasks::obligations::TasksQuery> crate::tasks::obligations::TasksQuery for Generated<P> {
    fn tasks(&self, context_id: Option<crate::tasks::ContextId>, status: Option<crate::tasks::TaskState>, status_timestamp_after: Option<crate::primitives::Timestamp>) -> Result<Vec<crate::tasks::Tasks>, UnmetObligation> {
        crate::tasks::obligations::TasksQuery::tasks(&self.ports, context_id, status, status_timestamp_after)
    }
}
