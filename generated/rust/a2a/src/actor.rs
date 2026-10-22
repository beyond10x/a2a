// generated from a2a v1
// model digest 340c397069089debcfcb2cf241e311c06d40ac443ecea2a196508e49c5fb6859
// contract digest f01a7ba8cee02bc352a5a026f681d10fa8fa5237c90807e3a38f323f46cb67ea
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
        ],
        Actor::TasksClient => &[
            "a2a.tasks.CancelTask",
            "a2a.tasks.CreateTaskPushNotificationConfig",
            "a2a.tasks.DeleteTaskPushNotificationConfig",
            "a2a.tasks.GetTask",
            "a2a.tasks.GetTaskPushNotificationConfig",
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
