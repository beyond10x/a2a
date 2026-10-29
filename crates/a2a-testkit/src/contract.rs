//! What the contract the generated server publishes declares of each command's outcomes, and
//! which actors may run each command or read each view
//! (`generated/rust/a2a/src/server/a2a-agent.openapi.json`).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use a2a::json::{self, Value};

/// The contract of the `a2a-agent` component, as the pinned `ess` generated it.
const CONTRACT: &str =
    include_str!("../../../generated/rust/a2a/src/server/a2a-agent.openapi.json");

/// How the contract describes an outcome the specification leaves to circumstances outside the
/// request (an `external:` outcome of `ess/domains/`).
///
/// ESS 0.75.0: the contract says so only in the outcome schema's `description`, so this reads the
/// prose; no `x-ess-*` member marks an outcome external or says which rule selects it.
const EXTERNAL: &str = "Decided outside the request:";

/// How the contract describes the outcome a command takes for an identity nothing holds.
///
/// ESS 0.75.0: prose as well, as for [`EXTERNAL`].
const UNKNOWN_IDENTITY: &str = "Taken when the request names an identity no record carries.";

/// One declared outcome of a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Outcome {
    /// Its name.
    pub(crate) name: String,
    /// The qualified name of the declared error it carries, where it refuses.
    pub(crate) error: Option<String>,
    /// Whether it may publish an event.
    pub(crate) publishes: bool,
    /// The qualified names of the events it publishes, in publication order: the `event` of each
    /// `published.prefixItems` entry of its response schema.
    pub(crate) events: Vec<String>,
    /// Whether the specification leaves it to circumstances outside the request.
    pub(crate) external: bool,
    /// Whether it is taken for an identity no record carries.
    pub(crate) unknown_identity: bool,
}

/// The HTTP status and the `google.rpc.Status` status an error answer of the binding carries
/// (A2A v1.0.1 section 5.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Answered {
    /// The HTTP status, `404`.
    pub(crate) http: u16,
    /// The `google.rpc.Code` name, `NOT_FOUND`.
    pub(crate) status: String,
}

impl std::fmt::Display for Answered {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} {}", self.http, self.status)
    }
}

/// The outcomes and grants of the contract.
#[derive(Debug)]
pub(crate) struct Contract {
    outcomes: BTreeMap<String, Vec<Outcome>>,
    /// The actors that may run a command, or read a view, by its qualified name.
    grants: BTreeMap<String, Vec<String>>,
    /// How each declared error is answered, by its qualified name.
    errors: BTreeMap<String, Answered>,
}

/// How the description of a declared error names its answer: `(JSON-RPC -32001, gRPC NOT_FOUND,
/// HTTP 404)`, or `(gRPC RESOURCE_EXHAUSTED, HTTP 429)` for one that is not an A2A error.
///
/// ESS 0.75.0: the mapping is prose `ess/domains/` writes into the error's description; no
/// `x-ess-*` member carries it.
fn answered(description: &str) -> Option<Answered> {
    let (_, after) = description.split_once("gRPC ")?;
    let status: String = after
        .chars()
        .take_while(|character| character.is_ascii_uppercase() || *character == '_')
        .collect();
    let (_, after) = after.split_once("HTTP ")?;
    let digits: String = after.chars().take_while(char::is_ascii_digit).collect();
    let http = digits.parse().ok()?;
    (!status.is_empty()).then_some(Answered { http, status })
}

/// The short name of the declared error an A2A error reason is (A2A v1.0.1 section 5.4).
pub(crate) fn error_of(reason: &a2a_client::Reason) -> Option<&'static str> {
    use a2a_client::Reason;
    Some(match reason {
        Reason::TaskNotFound => "TaskNotFound",
        Reason::TaskNotCancelable => "TaskNotCancelable",
        Reason::PushNotificationNotSupported => "PushNotificationNotSupported",
        Reason::UnsupportedOperation => "UnsupportedOperation",
        Reason::ContentTypeNotSupported => "ContentTypeNotSupported",
        Reason::ExtendedAgentCardNotConfigured => "ExtendedAgentCardNotConfigured",
        Reason::ExtensionSupportRequired => "ExtensionSupportRequired",
        Reason::VersionNotSupported => "VersionNotSupported",
        Reason::ContextMismatch => "ContextMismatch",
        Reason::ContextNotAccepted => "ContextNotAccepted",
        Reason::InvalidAgentResponse | Reason::Other(_) => return None,
    })
}

impl Contract {
    /// The contract compiled in.
    ///
    /// # Panics
    ///
    /// The contract compiled in is not the JSON `ess` generates; a build of this crate holds it.
    pub(crate) fn get() -> &'static Self {
        static CONTRACT_READ: OnceLock<Contract> = OnceLock::new();
        CONTRACT_READ.get_or_init(|| {
            Self::read(CONTRACT).expect("the generated contract declares its outcomes")
        })
    }

    fn read(text: &str) -> Result<Self, String> {
        let document = json::parse_written(text).map_err(|error| format!("not JSON: {error}"))?;
        let Some(Value::Object(schemas)) = document
            .member("components")
            .and_then(|components| components.member("schemas"))
        else {
            return Err("the contract declares no components.schemas".to_owned());
        };
        let mut outcomes: BTreeMap<String, Vec<Outcome>> = BTreeMap::new();
        let mut errors = BTreeMap::new();
        for (key, schema) in schemas {
            let Some(properties) = schema.member("properties") else {
                continue;
            };
            let Some(Value::Text(name)) = properties
                .member("outcome")
                .and_then(|outcome| outcome.member("const"))
            else {
                continue;
            };
            let Some(command) = key
                .strip_suffix(".Response")
                .and_then(|rest| rest.strip_suffix(name.as_str()))
                .and_then(|rest| rest.strip_suffix('.'))
            else {
                continue;
            };
            let error = match properties
                .member("error")
                .and_then(|error| error.member("const"))
            {
                Some(Value::Text(error)) => Some(error.clone()),
                _ => None,
            };
            if let (Some(error), Some(Value::Text(description))) = (
                &error,
                properties
                    .member("error")
                    .and_then(|error| error.member("description")),
            ) && let Some(answer) = answered(description)
            {
                errors.insert(error.clone(), answer);
            }
            let publishes = !matches!(
                properties
                    .member("published")
                    .and_then(|published| published.member("maxItems")),
                Some(Value::Number(most)) if most == "0"
            );
            let events = match properties
                .member("published")
                .and_then(|published| published.member("prefixItems"))
            {
                Some(Value::Array(items)) => items
                    .iter()
                    .filter_map(|item| {
                        match item
                            .member("properties")
                            .and_then(|properties| properties.member("event"))
                            .and_then(|event| event.member("const"))
                        {
                            Some(Value::Text(event)) => Some(event.clone()),
                            _ => None,
                        }
                    })
                    .collect(),
                _ => Vec::new(),
            };
            let description = match schema.member("description") {
                Some(Value::Text(description)) => description.as_str(),
                _ => "",
            };
            outcomes
                .entry(command.to_owned())
                .or_default()
                .push(Outcome {
                    name: name.clone(),
                    error,
                    publishes,
                    events,
                    external: description.contains(EXTERNAL),
                    unknown_identity: description.contains(UNKNOWN_IDENTITY),
                });
        }
        let mut grants = BTreeMap::new();
        if let Some(Value::Object(paths)) = document.member("paths") {
            for (_, operations) in paths {
                let Value::Object(operations) = operations else {
                    continue;
                };
                for (_, operation) in operations {
                    let Some(Value::Text(id)) = operation.member("operationId") else {
                        continue;
                    };
                    let granted = operation
                        .member("x-ess-may-invoke")
                        .or_else(|| operation.member("x-ess-may-read"));
                    if let Some(Value::Array(actors)) = granted {
                        let actors = actors
                            .iter()
                            .filter_map(|actor| match actor {
                                Value::Text(actor) => Some(actor.clone()),
                                _ => None,
                            })
                            .collect();
                        grants.insert(id.clone(), actors);
                    }
                }
            }
        }
        if outcomes.is_empty() {
            return Err("the contract declares no outcome".to_owned());
        }
        Ok(Self {
            outcomes,
            grants,
            errors,
        })
    }

    /// The declared outcomes of `command`.
    pub(crate) fn outcomes(&self, command: &str) -> &[Outcome] {
        self.outcomes.get(command).map_or(&[], Vec::as_slice)
    }

    /// The actors that may run the command, or read the view, `name`.
    pub(crate) fn granted(&self, name: &str) -> &[String] {
        self.grants.get(name).map_or(&[], Vec::as_slice)
    }

    /// How the declared error `name` is answered: by its qualified name, `a2a.tasks.TaskNotFound`,
    /// or by its short name, `TaskNotFound`, which every domain declaring it answers alike.
    pub(crate) fn answered_as(&self, name: &str) -> Option<&Answered> {
        self.errors.get(name).or_else(|| {
            self.errors
                .iter()
                .find(|(qualified, _)| {
                    qualified
                        .rsplit_once('.')
                        .is_some_and(|(_, short)| short == name)
                })
                .map(|(_, answered)| answered)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Contract;

    fn named(command: &str, pick: impl Fn(&super::Outcome) -> bool) -> Vec<String> {
        Contract::get()
            .outcomes(command)
            .iter()
            .filter(|outcome| pick(outcome))
            .map(|outcome| outcome.name.clone())
            .collect()
    }

    #[test]
    fn outcomes_carry_their_error_and_whether_they_publish() {
        let contract = Contract::get();
        let not_found = contract
            .outcomes("a2a.tasks.GetTask")
            .iter()
            .find(|outcome| outcome.name == "not-found")
            .expect("GetTask not-found");
        assert_eq!(not_found.error.as_deref(), Some("a2a.tasks.TaskNotFound"));
        assert!(!not_found.publishes);
        assert!(not_found.unknown_identity);
        assert!(!not_found.external);
        assert_eq!(
            named("a2a.tasks.SendMessage", |outcome| outcome.publishes),
            ["task-created"]
        );
    }

    #[test]
    fn outcomes_name_the_events_they_publish_in_order() {
        let events = |command: &str, outcome: &str| {
            Contract::get()
                .outcomes(command)
                .iter()
                .find(|held| held.name == outcome)
                .map(|held| held.events.clone())
                .unwrap_or_else(|| panic!("{command} {outcome}"))
        };
        assert_eq!(
            events("a2a.tasks.SendStreamingMessage", "task-created"),
            ["a2a.tasks.TaskCreated", "a2a.tasks.MessageStreamResponse"]
        );
        assert_eq!(
            events("a2a.tasks.SendMessage", "task-created"),
            ["a2a.tasks.TaskCreated"]
        );
        assert_eq!(
            events("a2a.tasks.CancelTask", "canceled"),
            ["a2a.tasks.TaskStatusUpdated"]
        );
        assert!(events("a2a.tasks.GetTask", "found").is_empty());
        assert!(events("a2a.tasks.SubscribeToTask", "subscribed").is_empty());
    }

    #[test]
    fn the_contract_tells_which_outcomes_are_decided_outside_the_request() {
        let mut external = named("a2a.discovery.GetExtendedAgentCard", |outcome| {
            outcome.external
        });
        external.sort();
        assert_eq!(
            external,
            [
                "extension-required",
                "not-configured",
                "not-supported",
                "version-not-supported"
            ]
        );
    }

    #[test]
    fn one_outcome_of_each_task_lookup_is_taken_for_an_unknown_identity() {
        for (command, outcome) in [
            ("a2a.tasks.GetTask", "not-found"),
            ("a2a.tasks.CancelTask", "not-found"),
            ("a2a.tasks.SubscribeToTask", "not-found"),
            ("a2a.tasks.GetTaskPushNotificationConfig", "not-found"),
            (
                "a2a.tasks.ListTaskPushNotificationConfigs",
                "task-not-found",
            ),
            (
                "a2a.tasks.DeleteTaskPushNotificationConfig",
                "task-not-found",
            ),
        ] {
            assert_eq!(
                named(command, |held| held.unknown_identity),
                [outcome],
                "{command}"
            );
        }
    }

    #[test]
    fn each_declared_error_is_answered_as_a2a_maps_it() {
        let contract = Contract::get();
        let answered = |name: &str| {
            contract
                .answered_as(name)
                .map(ToString::to_string)
                .unwrap_or_default()
        };
        assert_eq!(answered("a2a.tasks.TaskNotFound"), "404 NOT_FOUND");
        assert_eq!(answered("TaskNotFound"), "404 NOT_FOUND");
        assert_eq!(answered("TaskNotCancelable"), "400 FAILED_PRECONDITION");
        assert_eq!(answered("ContextMismatch"), "400 INVALID_ARGUMENT");
        assert_eq!(answered("ResourceExhausted"), "429 RESOURCE_EXHAUSTED");
        assert_eq!(
            answered("a2a.discovery.UnsupportedOperation"),
            "400 FAILED_PRECONDITION"
        );
        assert_eq!(contract.answered_as("TaskStateConflict"), None);
    }

    #[test]
    fn grants_name_who_may_run_or_read() {
        let contract = Contract::get();
        assert_eq!(contract.granted("a2a.tasks.GetTask"), ["a2a.tasks.Client"]);
        assert_eq!(contract.granted("a2a.tasks.Tasks"), ["a2a.tasks.Client"]);
        assert_eq!(
            contract.granted("a2a.discovery.GetExtendedAgentCard"),
            ["a2a.discovery.Client"]
        );
        assert_eq!(
            contract.granted("a2a.tasks.StartWorking"),
            ["a2a.tasks.Agent"]
        );
    }
}
