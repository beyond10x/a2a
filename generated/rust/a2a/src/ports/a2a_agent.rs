// generated from a2a v1
// model digest 19f81099dbb59817313f63273ff10776f7d98070eb5de7dda53e5d67514ef5bf
// contract digest b81ca6ca77a699d33385c2f8b48b31221205b1879fc0f96d44d1a695ab4bff58
// do not edit: regenerate with `ess synthesize --layout crate`

//! a2a-agent — the `a2a-agent` component of `a2a` v1.
//!
//! An A2A server agent, reached over JSON-RPC, gRPC or HTTP+JSON (A2A v1.0.1 section 5).
//!
//! The component's outer surface exactly as the specification declares it: accepted commands as
//! handlers, declared views as queries, published events as a typed outbox. The behaviour behind
//! every handler is an implementation obligation — see the `PLAN.md` beside this workspace — and
//! until one is satisfied, its stub answers with a typed refusal naming what is owed.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// An event this component declares it publishes, on its way to the system's transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishedEvent {
    /// `a2a.tasks.MessageStreamResponse`.
    MessageStreamResponse(crate::tasks::MessageStreamResponse),
    /// `a2a.tasks.PushNotificationConfigCreated`.
    PushNotificationConfigCreated(crate::tasks::PushNotificationConfigCreated),
    /// `a2a.tasks.PushNotificationConfigDeleted`.
    PushNotificationConfigDeleted(crate::tasks::PushNotificationConfigDeleted),
    /// `a2a.tasks.TaskArtifactUpdated`.
    TaskArtifactUpdated(crate::tasks::TaskArtifactUpdated),
    /// `a2a.tasks.TaskCreated`.
    TaskCreated(crate::tasks::TaskCreated),
    /// `a2a.tasks.TaskStatusUpdated`.
    TaskStatusUpdated(crate::tasks::TaskStatusUpdated),
    /// `a2a.tasks.TaskSubscriptionResponse`.
    TaskSubscriptionResponse(crate::tasks::TaskSubscriptionResponse),
}

/// a2a-agent — the port over the component's obligations.
///
/// `B` bundles every behaviour and query this component owes; constructing it over the domain's
/// `obligations::Unimplemented` yields a component that compiles and refuses, in the type system,
/// everything not yet implemented.
pub struct A2aAgent<B> {
    behaviors: B,
    outbox: Vec<PublishedEvent>,
}

impl<B> A2aAgent<B> {
    /// A new port over the given obligation implementations.
    pub fn new(behaviors: B) -> Self {
        Self {
            behaviors,
            outbox: Vec::new(),
        }
    }

    /// Hands over everything published since the last drain, in publication order.
    ///
    /// The system's transport calls this; anything else reading it is taking events the transport
    /// will then never deliver.
    pub fn drain_outbox(&mut self) -> Vec<PublishedEvent> {
        core::mem::take(&mut self.outbox)
    }
}

impl<B> A2aAgent<B>
where
    B: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    /// Accepts `a2a.discovery.GetExtendedAgentCard`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn get_extended_agent_card(&mut self, input: crate::discovery::GetExtendedAgentCard) -> Result<crate::discovery::GetExtendedAgentCardOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.get_extended_agent_card(input)?;
        match &outcome {
            crate::discovery::GetExtendedAgentCardOutcome::NotSupported { .. } => {}
            crate::discovery::GetExtendedAgentCardOutcome::NotConfigured { .. } => {}
            crate::discovery::GetExtendedAgentCardOutcome::Returned { .. } => {}
            crate::discovery::GetExtendedAgentCardOutcome::VersionNotSupported { .. } => {}
            crate::discovery::GetExtendedAgentCardOutcome::ExtensionRequired { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.CancelTask`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn cancel_task(&mut self, input: crate::tasks::CancelTask) -> Result<crate::tasks::CancelTaskOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.cancel_task(input)?;
        match &outcome {
            crate::tasks::CancelTaskOutcome::NotFound { .. } => {}
            crate::tasks::CancelTaskOutcome::NotCancelableNow { .. } => {}
            crate::tasks::CancelTaskOutcome::Canceled { task_status_updated, .. } => {
                self.outbox.push(PublishedEvent::TaskStatusUpdated(task_status_updated.clone()));
            }
            crate::tasks::CancelTaskOutcome::NotCancelable { .. } => {}
            crate::tasks::CancelTaskOutcome::VersionNotSupported { .. } => {}
            crate::tasks::CancelTaskOutcome::ExtensionRequired { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.CompleteTask`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn complete_task(&mut self, input: crate::tasks::CompleteTask) -> Result<crate::tasks::CompleteTaskOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.complete_task(input)?;
        match &outcome {
            crate::tasks::CompleteTaskOutcome::Moved { task_status_updated, .. } => {
                self.outbox.push(PublishedEvent::TaskStatusUpdated(task_status_updated.clone()));
            }
            crate::tasks::CompleteTaskOutcome::NotFound { .. } => {}
            crate::tasks::CompleteTaskOutcome::WrongState { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.CreateTaskPushNotificationConfig`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn create_task_push_notification_config(&mut self, input: crate::tasks::CreateTaskPushNotificationConfig) -> Result<crate::tasks::CreateTaskPushNotificationConfigOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.create_task_push_notification_config(input)?;
        match &outcome {
            crate::tasks::CreateTaskPushNotificationConfigOutcome::TaskNotFound { .. } => {}
            crate::tasks::CreateTaskPushNotificationConfigOutcome::Created { push_notification_config_created, .. } => {
                self.outbox.push(PublishedEvent::PushNotificationConfigCreated(push_notification_config_created.clone()));
            }
            crate::tasks::CreateTaskPushNotificationConfigOutcome::Assigned { push_notification_config_created, .. } => {
                self.outbox.push(PublishedEvent::PushNotificationConfigCreated(push_notification_config_created.clone()));
            }
            crate::tasks::CreateTaskPushNotificationConfigOutcome::VersionNotSupported { .. } => {}
            crate::tasks::CreateTaskPushNotificationConfigOutcome::ExtensionRequired { .. } => {}
            crate::tasks::CreateTaskPushNotificationConfigOutcome::PushNotSupported { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.DeleteTaskPushNotificationConfig`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn delete_task_push_notification_config(&mut self, input: crate::tasks::DeleteTaskPushNotificationConfig) -> Result<crate::tasks::DeleteTaskPushNotificationConfigOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.delete_task_push_notification_config(input)?;
        match &outcome {
            crate::tasks::DeleteTaskPushNotificationConfigOutcome::TaskNotFound { .. } => {}
            crate::tasks::DeleteTaskPushNotificationConfigOutcome::Deleted { push_notification_config_deleted, .. } => {
                self.outbox.push(PublishedEvent::PushNotificationConfigDeleted(push_notification_config_deleted.clone()));
            }
            crate::tasks::DeleteTaskPushNotificationConfigOutcome::VersionNotSupported { .. } => {}
            crate::tasks::DeleteTaskPushNotificationConfigOutcome::ExtensionRequired { .. } => {}
            crate::tasks::DeleteTaskPushNotificationConfigOutcome::PushNotSupported { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.FailTask`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn fail_task(&mut self, input: crate::tasks::FailTask) -> Result<crate::tasks::FailTaskOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.fail_task(input)?;
        match &outcome {
            crate::tasks::FailTaskOutcome::Moved { task_status_updated, .. } => {
                self.outbox.push(PublishedEvent::TaskStatusUpdated(task_status_updated.clone()));
            }
            crate::tasks::FailTaskOutcome::NotFound { .. } => {}
            crate::tasks::FailTaskOutcome::WrongState { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.GetTask`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn get_task(&mut self, input: crate::tasks::GetTask) -> Result<crate::tasks::GetTaskOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.get_task(input)?;
        match &outcome {
            crate::tasks::GetTaskOutcome::NotFound { .. } => {}
            crate::tasks::GetTaskOutcome::Found { .. } => {}
            crate::tasks::GetTaskOutcome::VersionNotSupported { .. } => {}
            crate::tasks::GetTaskOutcome::ExtensionRequired { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.GetTaskPushNotificationConfig`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn get_task_push_notification_config(&mut self, input: crate::tasks::GetTaskPushNotificationConfig) -> Result<crate::tasks::GetTaskPushNotificationConfigOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.get_task_push_notification_config(input)?;
        match &outcome {
            crate::tasks::GetTaskPushNotificationConfigOutcome::VersionNotSupported { .. } => {}
            crate::tasks::GetTaskPushNotificationConfigOutcome::ExtensionRequired { .. } => {}
            crate::tasks::GetTaskPushNotificationConfigOutcome::PushNotSupported { .. } => {}
            crate::tasks::GetTaskPushNotificationConfigOutcome::NotFound { .. } => {}
            crate::tasks::GetTaskPushNotificationConfigOutcome::ConfigNotFound { .. } => {}
            crate::tasks::GetTaskPushNotificationConfigOutcome::Found { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.ListTaskPushNotificationConfigs`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn list_task_push_notification_configs(&mut self, input: crate::tasks::ListTaskPushNotificationConfigs) -> Result<crate::tasks::ListTaskPushNotificationConfigsOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.list_task_push_notification_configs(input)?;
        match &outcome {
            crate::tasks::ListTaskPushNotificationConfigsOutcome::TaskNotFound { .. } => {}
            crate::tasks::ListTaskPushNotificationConfigsOutcome::Listed { .. } => {}
            crate::tasks::ListTaskPushNotificationConfigsOutcome::VersionNotSupported { .. } => {}
            crate::tasks::ListTaskPushNotificationConfigsOutcome::ExtensionRequired { .. } => {}
            crate::tasks::ListTaskPushNotificationConfigsOutcome::PushNotSupported { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.RejectTask`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn reject_task(&mut self, input: crate::tasks::RejectTask) -> Result<crate::tasks::RejectTaskOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.reject_task(input)?;
        match &outcome {
            crate::tasks::RejectTaskOutcome::Moved { task_status_updated, .. } => {
                self.outbox.push(PublishedEvent::TaskStatusUpdated(task_status_updated.clone()));
            }
            crate::tasks::RejectTaskOutcome::NotFound { .. } => {}
            crate::tasks::RejectTaskOutcome::WrongState { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.RequireAuth`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn require_auth(&mut self, input: crate::tasks::RequireAuth) -> Result<crate::tasks::RequireAuthOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.require_auth(input)?;
        match &outcome {
            crate::tasks::RequireAuthOutcome::Moved { task_status_updated, .. } => {
                self.outbox.push(PublishedEvent::TaskStatusUpdated(task_status_updated.clone()));
            }
            crate::tasks::RequireAuthOutcome::NotFound { .. } => {}
            crate::tasks::RequireAuthOutcome::WrongState { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.RequireInput`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn require_input(&mut self, input: crate::tasks::RequireInput) -> Result<crate::tasks::RequireInputOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.require_input(input)?;
        match &outcome {
            crate::tasks::RequireInputOutcome::Moved { task_status_updated, .. } => {
                self.outbox.push(PublishedEvent::TaskStatusUpdated(task_status_updated.clone()));
            }
            crate::tasks::RequireInputOutcome::NotFound { .. } => {}
            crate::tasks::RequireInputOutcome::WrongState { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.SendMessage`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn send_message(&mut self, input: crate::tasks::SendMessage) -> Result<crate::tasks::SendMessageOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.send_message(input)?;
        match &outcome {
            crate::tasks::SendMessageOutcome::TaskNotFound { .. } => {}
            crate::tasks::SendMessageOutcome::TaskTerminal { .. } => {}
            crate::tasks::SendMessageOutcome::ContextMismatch { .. } => {}
            crate::tasks::SendMessageOutcome::ContextNotAccepted { .. } => {}
            crate::tasks::SendMessageOutcome::ContentTypeNotSupported { .. } => {}
            crate::tasks::SendMessageOutcome::Continued { .. } => {}
            crate::tasks::SendMessageOutcome::DirectMessage { .. } => {}
            crate::tasks::SendMessageOutcome::TaskCreated { task_created, .. } => {
                self.outbox.push(PublishedEvent::TaskCreated(task_created.clone()));
            }
            crate::tasks::SendMessageOutcome::VersionNotSupported { .. } => {}
            crate::tasks::SendMessageOutcome::ExtensionRequired { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.SendStreamingMessage`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn send_streaming_message(&mut self, input: crate::tasks::SendStreamingMessage) -> Result<crate::tasks::SendStreamingMessageOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.send_streaming_message(input)?;
        match &outcome {
            crate::tasks::SendStreamingMessageOutcome::TaskNotFound { .. } => {}
            crate::tasks::SendStreamingMessageOutcome::TaskTerminal { .. } => {}
            crate::tasks::SendStreamingMessageOutcome::ContextMismatch { .. } => {}
            crate::tasks::SendStreamingMessageOutcome::ContextNotAccepted { .. } => {}
            crate::tasks::SendStreamingMessageOutcome::ContentTypeNotSupported { .. } => {}
            crate::tasks::SendStreamingMessageOutcome::Continued { message_stream_response, .. } => {
                self.outbox.push(PublishedEvent::MessageStreamResponse(message_stream_response.clone()));
            }
            crate::tasks::SendStreamingMessageOutcome::DirectMessage { message_stream_response, .. } => {
                self.outbox.push(PublishedEvent::MessageStreamResponse(message_stream_response.clone()));
            }
            crate::tasks::SendStreamingMessageOutcome::TaskCreated { task_created, message_stream_response, .. } => {
                self.outbox.push(PublishedEvent::TaskCreated(task_created.clone()));
                self.outbox.push(PublishedEvent::MessageStreamResponse(message_stream_response.clone()));
            }
            crate::tasks::SendStreamingMessageOutcome::VersionNotSupported { .. } => {}
            crate::tasks::SendStreamingMessageOutcome::ExtensionRequired { .. } => {}
            crate::tasks::SendStreamingMessageOutcome::StreamingNotSupported { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.StartWorking`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn start_working(&mut self, input: crate::tasks::StartWorking) -> Result<crate::tasks::StartWorkingOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.start_working(input)?;
        match &outcome {
            crate::tasks::StartWorkingOutcome::Moved { task_status_updated, .. } => {
                self.outbox.push(PublishedEvent::TaskStatusUpdated(task_status_updated.clone()));
            }
            crate::tasks::StartWorkingOutcome::NotFound { .. } => {}
            crate::tasks::StartWorkingOutcome::WrongState { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.SubscribeToTask`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn subscribe_to_task(&mut self, input: crate::tasks::SubscribeToTask) -> Result<crate::tasks::SubscribeToTaskOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.subscribe_to_task(input)?;
        match &outcome {
            crate::tasks::SubscribeToTaskOutcome::NotFound { .. } => {}
            crate::tasks::SubscribeToTaskOutcome::TaskTerminal { .. } => {}
            crate::tasks::SubscribeToTaskOutcome::Subscribed { .. } => {}
            crate::tasks::SubscribeToTaskOutcome::VersionNotSupported { .. } => {}
            crate::tasks::SubscribeToTaskOutcome::ExtensionRequired { .. } => {}
            crate::tasks::SubscribeToTaskOutcome::StreamingNotSupported { .. } => {}
        }
        Ok(outcome)
    }

    /// Accepts `a2a.tasks.UpdateArtifact`: runs the behaviour obligation, then publishes the declared events
    /// the outcome carries.
    ///
    /// `Err` is the typed refusal of an unmet obligation — never a domain outcome, which always
    /// arrives as a variant of the outcome type, refusals included.
    pub fn update_artifact(&mut self, input: crate::tasks::UpdateArtifact) -> Result<crate::tasks::UpdateArtifactOutcome, crate::obligation::UnmetObligation> {
        let outcome = self.behaviors.update_artifact(input)?;
        match &outcome {
            crate::tasks::UpdateArtifactOutcome::NotFound { .. } => {}
            crate::tasks::UpdateArtifactOutcome::Updated { task_artifact_updated, .. } => {
                self.outbox.push(PublishedEvent::TaskArtifactUpdated(task_artifact_updated.clone()));
            }
        }
        Ok(outcome)
    }

    /// Serves `a2a.tasks.AllTasks` at `read_your_writes` consistency, from the owed projection.
    pub fn all_tasks(&self) -> Result<Vec<crate::tasks::AllTasks>, crate::obligation::UnmetObligation> {
        self.behaviors.all_tasks()
    }

    /// Serves `a2a.tasks.PushNotificationConfigs` at `read_your_writes` consistency, from the owed projection.
    pub fn push_notification_configs(&self) -> Result<Vec<crate::tasks::PushNotificationConfigs>, crate::obligation::UnmetObligation> {
        self.behaviors.push_notification_configs()
    }

    /// Serves `a2a.tasks.TaskPushNotificationConfigs` at `read_your_writes` consistency, from the owed projection.
    pub fn task_push_notification_configs(&self, task_id: crate::tasks::TaskId) -> Result<Vec<crate::tasks::TaskPushNotificationConfigs>, crate::obligation::UnmetObligation> {
        self.behaviors.task_push_notification_configs(task_id)
    }

    /// Serves `a2a.tasks.Tasks` at `read_your_writes` consistency, from the owed projection.
    pub fn tasks(&self, context_id: Option<crate::tasks::ContextId>, status: Option<crate::tasks::TaskState>, status_timestamp_after: Option<crate::primitives::Timestamp>) -> Result<Vec<crate::tasks::Tasks>, crate::obligation::UnmetObligation> {
        self.behaviors.tasks(context_id, status, status_timestamp_after)
    }
}
