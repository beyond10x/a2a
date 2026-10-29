//! What the interpreter asks of an implementation: the [`Target`] trait and the values that cross
//! it.
//!
//! Every value crossing the trait is spelled as the specification spells it: field names are the
//! ESS names (`task_id`, `status_message`), at every depth, and enums carry their declared
//! variants. A target over a wire translates both ways; the interpreter never sees a wire name.

use std::fmt;

use a2a::json::Value;

/// An implementation the interpreter drives, one scenario at a time.
///
/// Each method answers `Err(TargetError::Unsupported(reason))` for what this target cannot carry
/// out; the scenario then ends `unsupported` with that reason. Any other error ends it `error`,
/// except [`TargetError::NotGranted`], which is the implementation's answer to a caller no grant
/// admits and is checked like any other answer.
pub trait Target {
    /// Opens a fresh, isolated execution context for the scenario `scenario`: nothing an earlier
    /// scenario did is visible in it.
    ///
    /// # Errors
    ///
    /// The context cannot be opened.
    fn begin(&mut self, scenario: &str) -> Result<(), TargetError>;

    /// Makes the implementation hold an instance of `entity` (an authored `setup:`): the identity
    /// `identity`, the fields of the object `fields`, ESS-named, and the lifecycle state `state`,
    /// with no history of how it got there. The scenario names the instance by `identity` once
    /// this answers.
    ///
    /// The default answers that the target cannot hold state it is handed.
    ///
    /// # Errors
    ///
    /// The target cannot hold the instance.
    fn establish(
        &mut self,
        entity: &str,
        identity: &Value,
        fields: &Value,
        state: &str,
    ) -> Result<(), TargetError> {
        let _ = (identity, fields, state);
        Err(TargetError::Unsupported(format!(
            "the target cannot be handed an instance of {entity} to hold: it is set up only through the commands it serves"
        )))
    }

    /// Runs `command` as `actor` (`None`: as no actor) with `input`, an object of the command's
    /// input fields.
    ///
    /// # Errors
    ///
    /// [`TargetError::NotGranted`] where the implementation refused the caller before the command
    /// ran; any other error where it could not be run.
    fn execute(
        &mut self,
        command: &str,
        actor: Option<&str>,
        input: &Value,
    ) -> Result<Answer, TargetError>;

    /// Reads the rows of `view` as `reader`, with `params`, an object of the view's parameters.
    ///
    /// # Errors
    ///
    /// [`TargetError::NotGranted`] where the implementation refused the reader; any other error
    /// where the view could not be read.
    fn query(
        &mut self,
        view: &str,
        reader: Reader<'_>,
        params: &Value,
    ) -> Result<Vec<Value>, TargetError>;

    /// Makes the next run of `command` take the externally decided branch `outcome`. A force
    /// applies to one run, and a run that is refused before the command ran does not use it.
    ///
    /// # Errors
    ///
    /// The target cannot bring that branch about.
    fn force(&mut self, command: &str, outcome: &str) -> Result<(), TargetError>;

    /// Every event the implementation published since the last call, in publication order.
    ///
    /// # Errors
    ///
    /// The events cannot be read.
    fn published(&mut self) -> Result<Vec<Event>, TargetError>;

    /// Lets the implementation issue `command`, a command its system issues itself, on the
    /// instance `instance`, and answers what it did.
    ///
    /// # Errors
    ///
    /// The target cannot observe what the system issues.
    fn await_outcome(&mut self, command: &str, instance: &Value) -> Result<Awaited, TargetError>;

    /// Closes the context [`Target::begin`] opened.
    ///
    /// # Errors
    ///
    /// The context cannot be closed.
    fn end(&mut self) -> Result<(), TargetError>;
}

/// Why a target did not answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetError {
    /// The target cannot carry this out; the scenario is `unsupported` with this reason.
    Unsupported(String),
    /// The implementation refused the caller before the command or read ran, naming the actor it
    /// refused (`None`: no actor).
    NotGranted(Option<String>),
    /// The target failed; the scenario is `error` with this reason.
    Failed(String),
}

impl fmt::Display for TargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(reason) => write!(formatter, "unsupported: {reason}"),
            Self::NotGranted(Some(actor)) => write!(formatter, "not granted to `{actor}`"),
            Self::NotGranted(None) => formatter.write_str("not granted to no actor"),
            Self::Failed(reason) => formatter.write_str(reason),
        }
    }
}

impl std::error::Error for TargetError {}

/// What one run of a command answered.
#[derive(Debug, Clone, PartialEq)]
pub struct Answer {
    /// The declared outcome it took.
    pub outcome: String,
    /// The qualified name of the declared error a refusing outcome carries.
    pub error: Option<String>,
    /// The response object an accepting outcome returns.
    pub response: Option<Value>,
}

/// Who a view is read as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reader<'a> {
    /// The harness, before a scenario names a reader: a read the scenario relies on to observe
    /// state, not one it holds to a grant.
    Harness,
    /// The declared actor of this qualified name.
    Actor(&'a str),
    /// No actor: an unauthenticated read.
    Anonymous,
}

/// One published event.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// The event's qualified name.
    pub name: String,
    /// Its payload object.
    pub payload: Value,
}

/// What a system-issued command came to.
#[derive(Debug, Clone, PartialEq)]
pub enum Awaited {
    /// The system issued it with `input` and it took an accepting branch.
    Reached {
        /// What it answered.
        answer: Answer,
        /// The input object the system issued it with.
        input: Value,
    },
    /// The system did not issue it, or issued it and it was refused.
    NotIssued,
}
