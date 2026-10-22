// generated from a2a v1
// model digest d675090b4779d71e3671099ac4d5efbaa17d7a58ae4e0ddaa6cafd8d5a23f002
// contract digest 3dea9905e0cd94c1255be15d5619996d104b54b00e2b8097da86f0bf80643b7f
// do not edit: regenerate with `ess synthesize --layout crate`

//! The `a2a` system, v1: its components assembled, its bindings wired, and its one transport.
//!
//! The transport is derived from the specification, not chosen: `at_least_once` is the only
//! delivery guarantee the model declares, so published events land on an append-only log and a
//! pump delivers each to every binding that reacts to it. The log is the system's observable
//! record, and so is the record of what each binding invoked. What no specification determines
//! — how an escalation event is filled, behaviour behind the ports — stays an obligation; see
//! the `PLAN.md` beside this workspace.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// An event on the system's log: everything any component publishes, and everything a binding
/// escalates into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemEvent {
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

impl SystemEvent {
    /// The qualified name the specification declares this event under.
    pub fn name(&self) -> &'static str {
        match self {
            Self::MessageStreamResponse(_) => "a2a.tasks.MessageStreamResponse",
            Self::PushNotificationConfigCreated(_) => "a2a.tasks.PushNotificationConfigCreated",
            Self::PushNotificationConfigDeleted(_) => "a2a.tasks.PushNotificationConfigDeleted",
            Self::TaskArtifactUpdated(_) => "a2a.tasks.TaskArtifactUpdated",
            Self::TaskCreated(_) => "a2a.tasks.TaskCreated",
            Self::TaskStatusUpdated(_) => "a2a.tasks.TaskStatusUpdated",
            Self::TaskSubscriptionResponse(_) => "a2a.tasks.TaskSubscriptionResponse",
        }
    }
}

impl From<crate::ports::a2a_agent::PublishedEvent> for SystemEvent {
    fn from(event: crate::ports::a2a_agent::PublishedEvent) -> Self {
        match event {
            crate::ports::a2a_agent::PublishedEvent::MessageStreamResponse(event) => Self::MessageStreamResponse(event),
            crate::ports::a2a_agent::PublishedEvent::PushNotificationConfigCreated(event) => Self::PushNotificationConfigCreated(event),
            crate::ports::a2a_agent::PublishedEvent::PushNotificationConfigDeleted(event) => Self::PushNotificationConfigDeleted(event),
            crate::ports::a2a_agent::PublishedEvent::TaskArtifactUpdated(event) => Self::TaskArtifactUpdated(event),
            crate::ports::a2a_agent::PublishedEvent::TaskCreated(event) => Self::TaskCreated(event),
            crate::ports::a2a_agent::PublishedEvent::TaskStatusUpdated(event) => Self::TaskStatusUpdated(event),
            crate::ports::a2a_agent::PublishedEvent::TaskSubscriptionResponse(event) => Self::TaskSubscriptionResponse(event),
        }
    }
}

/// The `a2a` system: every component behind its port, and the transport between them.
///
/// The component fields are public because commands enter the system through a component's own
/// port; the log and its delivery cursor are not, because publishing happens by pumping, not by
/// writing history directly.
pub struct System<A2aAgentBehaviors> {
    /// The `a2a-agent` component.
    pub a2a_agent: crate::ports::a2a_agent::A2aAgent<A2aAgentBehaviors>,
    published: Vec<SystemEvent>,
    cursor: usize,
}

impl<A2aAgentBehaviors> System<A2aAgentBehaviors> {
    /// Assembles the system from its components.
    pub fn new(a2a_agent: crate::ports::a2a_agent::A2aAgent<A2aAgentBehaviors>) -> Self {
        Self {
            a2a_agent,
            published: Vec::new(),
            cursor: 0,
        }
    }

    /// Everything published so far, in publication order — the system's observable record.
    pub fn published(&self) -> &[SystemEvent] {
        &self.published
    }

    /// Takes every event the pump has already delivered off the log, in publication order.
    ///
    /// A long-running shell calls this after each `pump`, or the log holds every event the
    /// process ever published. A `pump` returns with every logged event delivered: each
    /// reacting binding has had its attempt, and a binding whose attempt stopped holds the event in
    /// its own held-back list, not on the log. Events published since the last `pump` stay on the
    /// log, so the next `pump` still delivers them; taking never skips a binding.
    pub fn take_published(&mut self) -> Vec<SystemEvent> {
        let delivered: Vec<SystemEvent> = self.published.drain(..self.cursor).collect();
        self.cursor = 0;
        delivered
    }
}

impl<A2aAgentBehaviors> System<A2aAgentBehaviors>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    /// Delivers until quiescent: collects every component's outbox onto the log. No binding
    /// reacts to anything this specification publishes, so collecting is the whole delivery.
    pub fn pump(&mut self) -> Result<(), crate::obligation::UnmetObligation> {
        loop {
            self.collect();
            if self.cursor == self.published.len() {
                return Ok(());
            }
            self.cursor += 1;
        }
    }

    /// Moves every component's outbox onto the log, in component order.
    fn collect(&mut self) {
        for event in self.a2a_agent.drain_outbox() {
            self.published.push(SystemEvent::from(event));
        }
    }
}
