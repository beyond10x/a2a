// generated from a2a v1
// model digest 911794dff39b27311ddc6db22aa294d7c70c823fd2310684ad996b6eab4c5cdf
// contract digest f3a9f588bb74b749958999da8b7430eaf336cfa0d5e58a72b23699edc898a4dd
// do not edit: regenerate with `ess synthesize --layout crate`

//! Every actor the specification declares, and the commands each may invoke — as data.
//!
//! A grant is checked against a caller identity, which these types do not read from anywhere:
//! whatever authenticates a request builds a [`Caller`], and a served surface checks it against
//! [`may`] before the command runs. The `PLAN.md` beside this workspace says, per actor,
//! whether a generated surface enforces the grant or the caller does.

/// An actor the specification declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Actor {
    /// `a2a.discovery.Client`.
    DiscoveryClient,
    /// `a2a.tasks.Agent`.
    TasksAgent,
    /// `a2a.tasks.Client`.
    TasksClient,
}

impl Actor {
    /// Every declared actor, ordered by qualified name.
    pub const ALL: &'static [Actor] = &[
        Actor::DiscoveryClient,
        Actor::TasksAgent,
        Actor::TasksClient,
    ];

    /// The actor's qualified name, as the specification spells it.
    pub const fn name(self) -> &'static str {
        match self {
            Actor::DiscoveryClient => "a2a.discovery.Client",
            Actor::TasksAgent => "a2a.tasks.Agent",
            Actor::TasksClient => "a2a.tasks.Client",
        }
    }
}

/// The qualified names of the commands `actor` may invoke, ordered by name; empty for an
/// actor that only observes.
pub fn may(actor: Actor) -> &'static [&'static str] {
    match actor {
        Actor::DiscoveryClient => &[
            "a2a.discovery.GetExtendedAgentCard",
        ],
        Actor::TasksAgent => &[
            "a2a.tasks.CompleteTask",
            "a2a.tasks.FailTask",
            "a2a.tasks.RejectTask",
            "a2a.tasks.RequireAuth",
            "a2a.tasks.RequireInput",
            "a2a.tasks.StartWorking",
            "a2a.tasks.UpdateArtifact",
        ],
        Actor::TasksClient => &[
            "a2a.tasks.CancelTask",
            "a2a.tasks.CreateTaskPushNotificationConfig",
            "a2a.tasks.DeleteTaskPushNotificationConfig",
            "a2a.tasks.GetTask",
            "a2a.tasks.GetTaskPushNotificationConfig",
            "a2a.tasks.ListTaskPushNotificationConfigs",
            "a2a.tasks.SendMessage",
            "a2a.tasks.SendStreamingMessage",
            "a2a.tasks.SubscribeToTask",
        ],
    }
}

/// Who a request was authenticated as.
///
/// Built by whatever authenticates the request — a session, a token, a certificate — and
/// handed to the served surface's `dispatch` and `handle`, which check its grant
/// before the command runs. Never derived from the request itself: a client can write
/// anything into a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Caller {
    /// The declared actor.
    pub actor: Actor,
}

impl Caller {
    /// `true` when this caller may invoke `command`, named by its qualified name.
    pub fn may(&self, command: &str) -> bool {
        may(self.actor).contains(&command)
    }
}

/// The qualified names of the views `actor` may read, ordered by name: the views its grant
/// names. A view no actor's grant names is open to every caller and listed for none.
pub fn may_read(actor: Actor) -> &'static [&'static str] {
    match actor {
        Actor::DiscoveryClient => &[],
        Actor::TasksAgent => &[
            "a2a.tasks.AllTasks",
            "a2a.tasks.PushNotificationConfigs",
        ],
        Actor::TasksClient => &[
            "a2a.tasks.TaskPushNotificationConfigs",
            "a2a.tasks.Tasks",
        ],
    }
}

impl Caller {
    /// `true` when this caller may read `view`, a view some actor's grant names, named by
    /// its qualified name.
    pub fn may_read(&self, view: &str) -> bool {
        may_read(self.actor).contains(&view)
    }
}
