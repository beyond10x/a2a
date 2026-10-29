//! Runs one scenario's steps against a [`Target`], as `ess-conformance` 0.75.0's runner runs them
//! (`runner.rs`, `await_outcome.rs`, `event_multiplicity.rs`, `direct_response.rs`,
//! `response.rs`): the same checks, the same order, and the same rule for which failure stops a
//! scenario and which lets it go on.
//!
//! Every step records checks. A scenario's status is the worst of them, `failed` over `error` over
//! `unsupported` over `passed`; a failed check lets the scenario go on, an error or an unsupported
//! step stops it. A failed or error scenario gives the message of its first such check; an
//! unsupported one gives every distinct reason its unsupported checks gave, in order, so a reason
//! that only a later check gives still reaches the result.
//!
//! One departure, for an ESS 0.75.0 gap: an authored claim whose struct literal leaves out members
//! the target carries is `unsupported` and lets the scenario go on (`Run::expect_payload`).

use std::collections::BTreeMap;

use a2a::json::Value;

use crate::results::{ScenarioResult, Status};
use crate::shape::{self, Profile, Shape, SubjectShape};
use crate::suite::{DirectResponse, Expectation, ResponsePayload, Scenario, ScenarioValue, Step};
use crate::target::{Answer, Awaited, Event, Reader, Target, TargetError};
use crate::value::{
    Reached, carried_at, fields_match, left_out_leaves, quote, reach, same, same_value,
};

/// Whether a scenario goes on after a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Continue,
    Stop,
}

/// What separates the distinct reasons of an unsupported scenario in its message.
const UNSUPPORTED_REASONS: &str = " | ";

/// One check a step recorded.
struct Check {
    status: Status,
    message: String,
}

/// The last command a step sent, and what it answered.
struct Executed {
    command: String,
    /// `None` where it was refused as not granted.
    answer: Option<Answer>,
    /// The events it published, in publication order.
    events: Vec<Event>,
    /// The actor the refusal named, where it was refused as not granted.
    not_granted: Option<Option<String>>,
}

impl Executed {
    fn published(&self) -> String {
        if self.events.is_empty() {
            format!("{} published no events", self.command)
        } else {
            let names: Vec<&str> = self
                .events
                .iter()
                .map(|event| event.name.as_str())
                .collect();
            format!("{} published {}", self.command, names.join(", "))
        }
    }

    fn outcome(&self) -> Option<&str> {
        self.answer.as_ref().map(|answer| answer.outcome.as_str())
    }

    fn error(&self) -> Option<&str> {
        self.answer
            .as_ref()
            .and_then(|answer| answer.error.as_deref())
    }
}

/// What a read the next step requires refused came to.
enum ReadAnswer {
    Served,
    Refused(Option<String>),
}

/// What an event claim requires of the fields it names.
#[derive(Default)]
struct EventValues {
    values: Vec<(String, Value)>,
    /// Fields relaying an awaited input the answer left out: carried absent or null.
    unissued: Vec<String>,
}

/// One scenario's state.
#[derive(Default)]
struct Run {
    checks: Vec<Check>,
    instances: BTreeMap<String, Value>,
    /// Every instance an entity setup established: its entity and identity.
    established: Vec<(String, Value)>,
    /// Every event observed with a command, once each.
    seen: Vec<Event>,
    /// Every event the target published in this scenario, in order.
    log: Vec<Event>,
    log_before: BTreeMap<String, usize>,
    last_command: Option<Executed>,
    awaited: Option<Value>,
    /// `None`: the harness reads; `Some(None)`: no actor; `Some(Some(actor))`: that actor.
    reader: Option<Option<String>>,
    read_refusal_expected: bool,
    read_answer: Option<(String, ReadAnswer)>,
    last_view: Option<(String, Vec<Value>)>,
    snapshots: BTreeMap<String, (Vec<(String, Value)>, Value)>,
    complete_shapes: BTreeMap<String, SubjectShape>,
    claimed: BTreeMap<String, usize>,
    assigned: BTreeMap<String, Vec<Option<usize>>>,
    act: Vec<Step>,
    /// Whether the scenario is an authored one (`<domain>/authored/<name>`), not synthesized.
    authored: bool,
}

/// Runs the scenario `id` against `target`.
pub(crate) fn scenario<T: Target + ?Sized>(
    id: &str,
    scenario: &Result<Scenario, String>,
    target: &mut T,
) -> ScenarioResult {
    let steps = match scenario {
        Ok(scenario) => &scenario.steps,
        Err(reason) => {
            return ScenarioResult {
                scenario_id: id.to_owned(),
                status: Status::Error,
                message: Some(format!("the scenario cannot be read: {reason}")),
            };
        }
    };
    let mut run = Run {
        authored: id.contains("/authored/"),
        ..Run::default()
    };
    if let Err(error) = target.begin(id) {
        run.target_failure("opening an isolated execution context", &error);
    } else {
        for (index, step) in steps.iter().enumerate() {
            let next = steps.get(index + 1);
            if let (Step::ExecuteCommand { .. }, Some(Step::ExpectNotGranted { unpublished, .. })) =
                (step, next)
                && run.count_before(unpublished, target) == Flow::Stop
            {
                break;
            }
            run.open_act(step, &steps[index + 1..]);
            run.read_refusal_expected = matches!(
                (step, next),
                (Step::QueryView { .. }, Some(Step::ExpectNotGranted { .. }))
            );
            if run.step(step, target) == Flow::Stop {
                break;
            }
        }
        if let Err(error) = target.end() {
            run.target_failure("closing the execution context", &error);
        }
    }
    let status = run
        .checks
        .iter()
        .map(|check| check.status)
        .max()
        .unwrap_or(Status::Passed);
    let message = match status {
        Status::Passed => None,
        Status::Unsupported => {
            let mut reasons: Vec<&str> = Vec::new();
            for check in run.checks.iter().filter(|check| check.status == status) {
                if !reasons.contains(&check.message.as_str()) {
                    reasons.push(&check.message);
                }
            }
            Some(reasons.join(UNSUPPORTED_REASONS))
        }
        _ => run
            .checks
            .iter()
            .find(|check| check.status == status)
            .map(|check| check.message.clone()),
    };
    ScenarioResult {
        scenario_id: id.to_owned(),
        status,
        message,
    }
}

impl Run {
    fn record(&mut self, status: Status, message: impl Into<String>) {
        self.checks.push(Check {
            status,
            message: message.into(),
        });
    }

    fn passed(&mut self) {
        self.record(Status::Passed, String::new());
    }

    fn failed(&mut self, message: impl Into<String>) {
        self.record(Status::Failed, message);
    }

    fn errored(&mut self, message: impl Into<String>) {
        self.record(Status::Error, message);
    }

    /// A target that could not carry out `operation`: unsupported with the target's reason, or an
    /// error.
    fn target_failure(&mut self, operation: &str, error: &TargetError) {
        match error {
            TargetError::Unsupported(reason) => self.record(Status::Unsupported, reason.clone()),
            other => self.errored(format!(
                "the target could not carry out {operation}: {other}"
            )),
        }
    }

    fn no_command(&mut self, subject: &str) -> Flow {
        self.errored(format!(
            "{subject} of the last command: no command had been executed in this scenario"
        ));
        Flow::Stop
    }

    fn unresolvable(&mut self, view: &str, reason: &str) -> Flow {
        self.errored(format!(
            "view {view}: every field it is matched on refers to something an earlier step \
             established, and {reason}"
        ));
        Flow::Stop
    }

    /// Takes what the target published since the last call onto the log, and answers it.
    fn drain<T: Target + ?Sized>(&mut self, target: &mut T) -> Result<Vec<Event>, TargetError> {
        let fresh = target.published()?;
        self.log.extend(fresh.iter().cloned());
        Ok(fresh)
    }

    fn remember(&mut self, events: &[Event]) {
        for event in events {
            if !self.seen.contains(event) {
                self.seen.push(event.clone());
            }
        }
    }

    fn open_act(&mut self, step: &Step, after: &[Step]) {
        if matches!(
            step,
            Step::ExecuteCommand { .. } | Step::AwaitOutcome { .. }
        ) {
            self.claimed.clear();
            self.assigned.clear();
            self.act = after
                .iter()
                .take_while(|step| !matches!(step, Step::ExecuteCommand { .. }))
                .filter(|step| {
                    matches!(
                        step,
                        Step::ExpectEvent { .. } | Step::ExpectEventValues { .. }
                    )
                })
                .cloned()
                .collect();
        }
    }

    fn resolve(&self, value: &ScenarioValue) -> Result<Value, String> {
        match value {
            ScenarioValue::Literal(value) => Ok(value.clone()),
            ScenarioValue::Instance(name) => self
                .instances
                .get(name)
                .cloned()
                .ok_or_else(|| format!("no earlier step bound the instance `{name}`")),
            ScenarioValue::Observed { event, field } => self
                .seen
                .iter()
                .find(|seen| &seen.name == event)
                .ok_or_else(|| format!("`{event}` had not been observed"))
                .and_then(|seen| {
                    seen.payload
                        .member(field)
                        .cloned()
                        .ok_or_else(|| format!("`{event}` carried no field `{field}`"))
                }),
            ScenarioValue::Awaited(field) => self
                .awaited
                .as_ref()
                .ok_or_else(|| {
                    format!("`{{kind: awaited, field: {field}}}` is read before any await_outcome")
                })
                .and_then(|input| {
                    input
                        .member(field)
                        .cloned()
                        .ok_or_else(|| format!("the awaited answer carried no `{field}`"))
                }),
            ScenarioValue::List(items) => items
                .iter()
                .map(|item| self.resolve(item))
                .collect::<Result<_, _>>()
                .map(Value::Array),
            ScenarioValue::Members(members) => members
                .iter()
                .map(|(name, member)| self.resolve(member).map(|value| (name.clone(), value)))
                .collect::<Result<_, _>>()
                .map(Value::Object),
        }
    }

    /// Whether `value` reads an input the last await's answer left out, by omitting it or
    /// spelling it `null`.
    fn left_out(&self, value: &ScenarioValue) -> bool {
        match (value, &self.awaited) {
            (ScenarioValue::Awaited(field), Some(input)) => {
                matches!(input.member(field), None | Some(Value::Null))
            }
            _ => false,
        }
    }

    fn resolve_payload(&self, payload: &[(String, ScenarioValue)]) -> Result<EventValues, String> {
        let mut values = EventValues::default();
        for (field, value) in payload {
            if self.left_out(value) {
                values.unissued.push(field.clone());
            } else {
                values.values.push((field.clone(), self.resolve(value)?));
            }
        }
        Ok(values)
    }

    fn resolve_all(
        &self,
        values: &[(String, ScenarioValue)],
    ) -> Result<Vec<(String, Value)>, String> {
        values
            .iter()
            .map(|(field, value)| {
                self.resolve(value)
                    .map(|resolved| (field.clone(), resolved))
                    .map_err(|reason| format!("`{field}`: {reason}"))
            })
            .collect()
    }

    fn reader(&self) -> Reader<'_> {
        match &self.reader {
            None => Reader::Harness,
            Some(None) => Reader::Anonymous,
            Some(Some(actor)) => Reader::Actor(actor),
        }
    }

    fn step<T: Target + ?Sized>(&mut self, step: &Step, target: &mut T) -> Flow {
        match step {
            Step::EstablishEntity {
                instance,
                entity,
                identity,
                fields,
                state,
            } => self.establish_entity(instance, entity, identity, fields, state, target),
            Step::ExecuteCommand {
                command,
                actor,
                input,
            } => self.execute_command(command, actor.as_deref(), input, target),
            Step::AwaitOutcome {
                command,
                identity,
                instance,
                input,
                shape,
            } => self.await_outcome(command, identity, instance, input, shape, target),
            Step::ExpectNotIssued {
                command,
                instance,
                unpublished,
            } => self.expect_not_issued(command, instance, unpublished, target),
            Step::ConfigureExternalOutcome { command, outcome } => {
                match target.force(command, outcome) {
                    Ok(()) => Flow::Continue,
                    Err(error) => {
                        self.target_failure(
                            &format!("forcing the external outcome `{command}` `{outcome}`"),
                            &error,
                        );
                        Flow::Stop
                    }
                }
            }
            Step::CaptureInstance {
                instance,
                entity,
                event,
                field,
            } => self.capture_instance(instance, entity, event, field),
            Step::ExpectOutcome { command, outcome } => self.expect_outcome(command, outcome),
            Step::ExpectNotGranted { actor, unpublished } => {
                if let Some((view, answer)) = self.read_answer.take() {
                    return self.expect_read_not_granted(actor.as_deref(), &view, &answer);
                }
                self.expect_not_granted(actor.as_deref(), unpublished, target)
            }
            Step::ExpectNoEvents => {
                if self
                    .last_command
                    .as_ref()
                    .is_some_and(|executed| executed.events.is_empty())
                {
                    self.passed();
                } else {
                    let observed = self
                        .last_command
                        .as_ref()
                        .map_or_else(|| "no command".to_owned(), Executed::published);
                    self.failed(format!(
                        "no direct events: expected a preceding command with zero direct events; \
                         {observed}"
                    ));
                }
                Flow::Continue
            }
            Step::ExpectNoError => {
                let Some(executed) = self.last_command.as_ref() else {
                    return self.no_command("absence of an error");
                };
                match executed.error() {
                    Some(error) => {
                        let message =
                            format!("no error: {} carried error = {error}", executed.command);
                        self.failed(message);
                    }
                    None => self.passed(),
                }
                Flow::Continue
            }
            Step::ExpectError { error } => {
                let Some(executed) = self.last_command.as_ref() else {
                    return self.no_command("a declared error");
                };
                match executed.error() {
                    Some(carried) if carried == error => self.passed(),
                    Some(carried) => {
                        let message = format!(
                            "error {error}: {} carried error = {carried}",
                            executed.command
                        );
                        self.failed(message);
                    }
                    None => {
                        let message = format!(
                            "error {error}: {} carried no declared error ({})",
                            executed.command,
                            outcome_seen(executed)
                        );
                        self.failed(message);
                    }
                }
                Flow::Continue
            }
            Step::ExpectNoEvent { event } => {
                let Some(executed) = self.last_command.as_ref() else {
                    return self.no_command("the absence of an event");
                };
                if executed.events.iter().any(|seen| &seen.name == event) {
                    let message = format!("no event {event}: {}", executed.published());
                    self.failed(message);
                } else {
                    self.passed();
                }
                Flow::Continue
            }
            Step::ExpectEvent {
                event,
                payload,
                shape,
            } => {
                let values = EventValues {
                    values: payload.clone(),
                    unissued: Vec::new(),
                };
                self.expect_event(event, &values, shape)
            }
            Step::ExpectEventValues {
                event,
                payload,
                shape,
            } => match self.resolve_payload(payload) {
                Ok(values) => self.expect_event(event, &values, shape),
                Err(reason) => {
                    self.errored(format!(
                        "event value equality: values resolved before this event assertion, and \
                         {reason}"
                    ));
                    Flow::Stop
                }
            },
            Step::ReadAs { actor } => {
                self.reader = Some(actor.clone());
                Flow::Continue
            }
            Step::QueryView { view, params } => self.query_view(view, params, target),
            Step::ExpectView { view, expectation } => self.expect_view(view, expectation),
            Step::SnapshotSubject { view, subject } => {
                self.snapshot_subject(view, Some(subject), None)
            }
            Step::SnapshotCompleteSubject {
                view,
                subject,
                shape,
            } => {
                self.complete_shapes.insert(view.clone(), shape.clone());
                self.snapshot_subject(view, Some(subject), Some(shape))
            }
            Step::ExpectSubjectUnchanged { view } => self.snapshot_subject(view, None, None),
            Step::ExpectCompleteSubjectUnchanged { view } => {
                let Some(shape) = self.complete_shapes.get(view).cloned() else {
                    return self.unresolvable(view, "no complete subject snapshot preceded it");
                };
                self.snapshot_subject(view, None, Some(&shape))
            }
            Step::ExpectSubjectAbsent { view, subject } => self.expect_absent(view, subject),
            Step::ExpectDirectResponse(response) => {
                let checked = self.direct_response(response);
                self.payload_check(&format!("direct response {}", response.command), checked)
            }
            Step::ExpectResponsePayload(response) => {
                let checked = self.response_payload(response);
                self.payload_check(&format!("response payload {}", response.event), checked)
            }
        }
    }

    fn payload_check(&mut self, about: &str, checked: Result<(), String>) -> Flow {
        match checked {
            Ok(()) => self.passed(),
            Err(reason) => self.failed(format!("{about}: {reason}")),
        }
        Flow::Continue
    }

    fn execute_command<T: Target + ?Sized>(
        &mut self,
        command: &str,
        actor: Option<&str>,
        input: &[(String, ScenarioValue)],
        target: &mut T,
    ) -> Flow {
        let mut resolved = Vec::new();
        for (field, value) in input {
            match self.resolve(value) {
                Ok(value) => resolved.push((field.clone(), value)),
                Err(reason) => {
                    self.errored(format!(
                        "the input `{field}` of `{command}` refers to something an earlier step \
                         established, and {reason}"
                    ));
                    return Flow::Stop;
                }
            }
        }
        let input = Value::Object(resolved);
        match target.execute(command, actor, &input) {
            Ok(answer) => {
                let events = match self.drain(target) {
                    Ok(events) => events,
                    Err(error) => {
                        self.target_failure(
                            &format!("observing what `{command}` published"),
                            &error,
                        );
                        return Flow::Stop;
                    }
                };
                self.remember(&events);
                self.last_command = Some(Executed {
                    command: command.to_owned(),
                    answer: Some(answer),
                    events,
                    not_granted: None,
                });
                Flow::Continue
            }
            Err(TargetError::NotGranted(named)) => {
                if let Err(error) = self.drain(target) {
                    self.target_failure(&format!("observing what `{command}` published"), &error);
                    return Flow::Stop;
                }
                self.last_command = Some(Executed {
                    command: command.to_owned(),
                    answer: None,
                    events: Vec::new(),
                    not_granted: Some(named),
                });
                Flow::Continue
            }
            Err(error) => {
                self.target_failure(&format!("invoking `{command}`"), &error);
                Flow::Stop
            }
        }
    }

    fn expect_outcome(&mut self, command: &str, outcome: &str) -> Flow {
        let Some(executed) = self.last_command.as_ref() else {
            return self.no_command("an outcome");
        };
        if executed.command == command && executed.outcome() == Some(outcome) {
            self.passed();
        } else {
            let message = format!(
                "outcome {command} {outcome}: expected outcome = {outcome}; {}",
                outcome_seen(executed)
            );
            self.failed(message);
        }
        Flow::Continue
    }

    fn count_before<T: Target + ?Sized>(&mut self, unpublished: &[String], target: &mut T) -> Flow {
        self.log_before.clear();
        if unpublished.is_empty() {
            return Flow::Continue;
        }
        if let Err(error) = self.drain(target) {
            self.target_failure("observing the log before the refused send", &error);
            return Flow::Stop;
        }
        for event in unpublished {
            let count = self.logged(event);
            self.log_before.insert(event.clone(), count);
        }
        Flow::Continue
    }

    fn logged(&self, event: &str) -> usize {
        self.log.iter().filter(|seen| seen.name == event).count()
    }

    fn nothing_published_after<T: Target + ?Sized>(
        &mut self,
        unpublished: &[String],
        target: &mut T,
    ) -> Flow {
        if unpublished.is_empty() {
            return Flow::Continue;
        }
        if let Err(error) = self.drain(target) {
            self.target_failure("observing the log after the refused send", &error);
            return Flow::Stop;
        }
        for event in unpublished {
            let after = self.logged(event);
            let before = self.log_before.get(event).copied().unwrap_or(0);
            if after <= before {
                self.passed();
            } else {
                self.failed(format!(
                    "no event {event} after the refused send: a refused command sets nothing in \
                     motion, and the log holds {after} occurrence(s) of {event} after the send, \
                     {before} before it"
                ));
            }
        }
        Flow::Continue
    }

    fn expect_not_granted<T: Target + ?Sized>(
        &mut self,
        actor: Option<&str>,
        unpublished: &[String],
        target: &mut T,
    ) -> Flow {
        let Some(executed) = self.last_command.as_ref() else {
            return self.no_command("the refusal an ungranted actor gets");
        };
        if executed.not_granted.as_ref().map(Option::as_deref) == Some(actor) {
            self.passed();
            return self.nothing_published_after(unpublished, target);
        }
        let message = format!(
            "not granted to {}: expected {}; {}",
            actor.unwrap_or("no actor"),
            not_granted_seen(actor),
            outcome_seen(executed)
        );
        self.failed(message);
        Flow::Continue
    }

    fn expect_read_not_granted(
        &mut self,
        actor: Option<&str>,
        view: &str,
        answer: &ReadAnswer,
    ) -> Flow {
        if let ReadAnswer::Refused(named) = answer
            && named.as_deref() == actor
        {
            self.passed();
            return Flow::Continue;
        }
        let seen = match answer {
            ReadAnswer::Refused(named) => not_granted_seen(named.as_deref()),
            ReadAnswer::Served => format!("`{view}` was served; the target checked no read grant"),
        };
        self.failed(format!(
            "reading {view} not granted to {}: expected {}; {seen}",
            actor.unwrap_or("no actor"),
            not_granted_seen(actor)
        ));
        Flow::Continue
    }

    /// An authored `setup:`, as the runner's `establish_entity`: the target is handed the
    /// instance, and its identity is bound to `instance` only once the target holds it. An
    /// instance named, or an identity of `entity` established, earlier in the scenario is not
    /// established twice: the scenario is in error, as the runner records it, not unsupported.
    fn establish_entity<T: Target + ?Sized>(
        &mut self,
        instance: &str,
        entity: &str,
        identity: &Value,
        fields: &Value,
        state: &str,
        target: &mut T,
    ) -> Flow {
        let established = (entity.to_owned(), identity.clone());
        if self.instances.contains_key(instance) || self.established.contains(&established) {
            self.target_failure(
                "establishing entity state",
                &TargetError::Failed(
                    "entity setup: duplicate instance or qualified entity identity".to_owned(),
                ),
            );
            return Flow::Stop;
        }
        match target.establish(entity, identity, fields, state) {
            Ok(()) => {
                self.established.push(established);
                self.instances.insert(instance.to_owned(), identity.clone());
                self.last_view = None;
                Flow::Continue
            }
            Err(error) => {
                self.target_failure(&format!("establishing `{entity}` as `{instance}`"), &error);
                Flow::Stop
            }
        }
    }

    fn capture_instance(&mut self, instance: &str, entity: &str, event: &str, field: &str) -> Flow {
        let Some(executed) = self.last_command.as_ref() else {
            return self.no_command("an instance identity");
        };
        let identity = executed
            .events
            .iter()
            .find(|seen| seen.name == event)
            .and_then(|seen| seen.payload.member(field))
            .cloned();
        match identity {
            Some(identity) => {
                self.instances.insert(instance.to_owned(), identity);
                Flow::Continue
            }
            None => {
                let message = format!(
                    "the identity `{instance}` of a {entity}: expected {event}.{field} to carry \
                     the new identity; {}",
                    executed.published()
                );
                self.errored(message);
                Flow::Stop
            }
        }
    }

    fn expect_event(&mut self, event: &str, values: &EventValues, shape: &Shape) -> Flow {
        let Some(executed) = self.last_command.as_ref() else {
            return self.no_command("an event");
        };
        let occurrences: Vec<&Value> = executed
            .events
            .iter()
            .filter(|seen| seen.name == event)
            .map(|seen| &seen.payload)
            .collect();
        let ordinal = self.claimed.get(event).copied().unwrap_or_default();
        if ordinal == 0 || !self.assigned.contains_key(event) {
            let carried: Vec<Vec<bool>> = self
                .act
                .iter()
                .filter_map(|step| match step {
                    Step::ExpectEvent {
                        event: claimed,
                        payload,
                        shape,
                    } if claimed == event => Some(Ok((
                        EventValues {
                            values: payload.clone(),
                            unissued: Vec::new(),
                        },
                        shape,
                    ))),
                    Step::ExpectEventValues {
                        event: claimed,
                        payload,
                        shape,
                    } if claimed == event => {
                        Some(self.resolve_payload(payload).map(|values| (values, shape)))
                    }
                    _ => None,
                })
                .map(|claim| {
                    occurrences
                        .iter()
                        .map(|held| {
                            claim
                                .as_ref()
                                .is_ok_and(|(values, shape)| carries(held, values, shape))
                        })
                        .collect()
                })
                .collect();
            let assigned = assignment(&carried, occurrences.len());
            self.assigned.insert(event.to_owned(), assigned);
        }
        self.claimed.insert(event.to_owned(), ordinal + 1);
        let taken = self
            .assigned
            .get(event)
            .and_then(|assigned| assigned.get(ordinal).copied().flatten())
            .and_then(|index| occurrences.get(index))
            .map(|carried| (*carried).clone());
        let Some(carried) = taken else {
            let observed = if occurrences.is_empty() {
                executed.published()
            } else {
                let claims = self
                    .act
                    .iter()
                    .filter(|step| {
                        matches!(step, Step::ExpectEvent { event: claimed, .. }
                            | Step::ExpectEventValues { event: claimed, .. } if claimed == event)
                    })
                    .count();
                format!(
                    "{} occurrence(s) of {event} published, {claims} claimed",
                    occurrences.len()
                )
            };
            self.failed(format!("event {event}: expected it published; {observed}"));
            return Flow::Continue;
        };
        self.passed();
        self.expect_payload(event, &carried, values, shape);
        Flow::Continue
    }

    /// Holds the payload an event claim took to the values and shape it requires.
    ///
    /// ESS 0.75.0 gap, worked around for an authored scenario only: authoring refuses a dotted
    /// leaf key in an event claim (`ESS-AUTHOR-013`), so a scenario can claim a struct-valued
    /// field only as a literal, and authoring reads a member the literal leaves out as unclaimed
    /// (`authored.rs`, `minted_leaves`). The runner compares the literal whole (`fields_match`,
    /// `same_value`), so a member the model leaves to the implementation (`{generated: true}`)
    /// and the literal leaves out reads as claimed absent. Synthesis writes the same claim leaf by
    /// leaf (`status.state`) and is not affected. Where every leaf the literal writes is held and
    /// the target carries members it leaves out, the claim is `unsupported`, naming those members;
    /// it is never `passed`, and a leaf written and not held is still `failed`.
    fn expect_payload(
        &mut self,
        event: &str,
        carried: &Value,
        values: &EventValues,
        shape: &Shape,
    ) {
        if shape.is_empty() && values.values.is_empty() && values.unissued.is_empty() {
            return;
        }
        let mut wrong = Vec::new();
        let mut left_out = Vec::new();
        for (field, expected) in &values.values {
            match carried_at(carried, field) {
                Some(observed) if same(observed, expected) => {}
                Some(observed) => match self
                    .authored
                    .then(|| left_out_leaves(observed, expected, field))
                    .flatten()
                {
                    Some(leaves) if !leaves.is_empty() => left_out.extend(leaves),
                    _ => wrong.push(format!(
                        "{field} = {}, expected {}",
                        quote(observed),
                        quote(expected)
                    )),
                },
                None => wrong.push(format!(
                    "{field} was not carried, expected {}",
                    quote(expected)
                )),
            }
        }
        for field in &values.unissued {
            if let Some(observed) = carried_at(carried, field).filter(|node| **node != Value::Null)
            {
                wrong.push(format!(
                    "{field} = {}, expected absent or null: the awaited answer left the input it \
                     relays out",
                    quote(observed)
                ));
            }
        }
        wrong.extend(shape_violations(carried, shape));
        if !wrong.is_empty() {
            self.failed(format!("payload of {event}: {}", wrong.join("; ")));
        } else if left_out.is_empty() {
            self.passed();
        } else {
            self.record(Status::Unsupported, authored_literal(event, &left_out));
        }
    }

    fn query_view<T: Target + ?Sized>(
        &mut self,
        view: &str,
        params: &[(String, ScenarioValue)],
        target: &mut T,
    ) -> Flow {
        let mut bound = Vec::new();
        for (name, value) in params {
            match self.resolve(value) {
                Ok(value) => bound.push((name.clone(), value)),
                Err(reason) => {
                    return self.unresolvable(view, &format!("the parameter `{name}`: {reason}"));
                }
            }
        }
        let expected = std::mem::take(&mut self.read_refusal_expected);
        let params = Value::Object(bound);
        let reader = self.reader();
        let answered = target.query(view, reader, &params);
        match answered {
            Ok(rows) => {
                if expected {
                    self.read_answer = Some((view.to_owned(), ReadAnswer::Served));
                }
                self.last_view = Some((view.to_owned(), rows));
                Flow::Continue
            }
            Err(TargetError::NotGranted(named)) if expected => {
                self.last_view = None;
                self.read_answer = Some((view.to_owned(), ReadAnswer::Refused(named)));
                Flow::Continue
            }
            Err(TargetError::NotGranted(named)) => {
                let sent = match &self.reader {
                    Some(Some(actor)) => format!("`{actor}`"),
                    Some(None) => "no actor".to_owned(),
                    None => "the harness".to_owned(),
                };
                self.failed(format!(
                    "reading {view}: expected `{view}` read as {sent} to answer its rows; {}",
                    not_granted_seen(named.as_deref())
                ));
                Flow::Stop
            }
            Err(error) => {
                self.target_failure(&format!("reading `{view}`"), &error);
                Flow::Stop
            }
        }
    }

    fn expect_view(&mut self, view: &str, expectation: &Expectation) -> Flow {
        let resolve = |run: &Self, fields: &[(String, ScenarioValue)]| {
            fields
                .iter()
                .map(|(field, value)| {
                    if run.left_out(value) {
                        return Ok((field.clone(), Value::Null));
                    }
                    run.resolve(value)
                        .map(|resolved| (field.clone(), resolved))
                        .map_err(|reason| format!("`{field}`: {reason}"))
                })
                .collect::<Result<Vec<_>, String>>()
        };
        let required = match expectation {
            Expectation::Contains(fields) => resolve(self, fields).map(Required::Contains),
            Expectation::Excludes(fields) => resolve(self, fields).map(Required::Excludes),
            Expectation::Counts {
                at_least: None,
                at_most: None,
            } => Err("a count expectation with neither bound asks nothing".to_owned()),
            Expectation::Counts { at_least, at_most } => Ok(Required::Counts {
                at_least: *at_least,
                at_most: *at_most,
            }),
        };
        let required = match required {
            Ok(required) => required,
            Err(reason) => return self.unresolvable(view, &reason),
        };
        let rows = match &self.last_view {
            Some((queried, rows)) if queried == view => rows,
            Some((queried, _)) => {
                let message = format!(
                    "view {view}: a query of `{view}` precedes an expectation of it; the view \
                     last read was `{queried}`"
                );
                self.errored(message);
                return Flow::Stop;
            }
            None => {
                self.errored(format!(
                    "view {view}: a query of `{view}` precedes an expectation of it; no view had \
                     been read"
                ));
                return Flow::Stop;
            }
        };
        let matching = |fields: &[(String, Value)]| {
            rows.iter()
                .filter(|row| fields_match(row, fields, same_value))
                .count()
        };
        let verdict = match &required {
            Required::Contains(fields) => (matching(fields) > 0)
                .then_some(())
                .ok_or_else(|| format!("no row carries {}", quote_fields(fields))),
            Required::Excludes(fields) => match matching(fields) {
                0 => Ok(()),
                count => Err(format!("{count} row(s) carry {}", quote_fields(fields))),
            },
            Required::Counts { at_least, at_most } => {
                let count = rows.len();
                if at_least.is_some_and(|least| count < least)
                    || at_most.is_some_and(|most| count > most)
                {
                    Err(format!(
                        "{count} row(s), expected at least {} and at most {}",
                        at_least.map_or_else(|| "0".to_owned(), |least| least.to_string()),
                        at_most.map_or_else(|| "any".to_owned(), |most| most.to_string())
                    ))
                } else {
                    Ok(())
                }
            }
        };
        match verdict {
            Ok(()) => self.passed(),
            Err(observed) => {
                let shown = quote(&Value::Array(rows.clone()));
                self.failed(format!("view {view}: {observed}; rows {shown}"));
            }
        }
        Flow::Continue
    }

    fn snapshot_subject(
        &mut self,
        view: &str,
        subject: Option<&Vec<(String, ScenarioValue)>>,
        shape: Option<&SubjectShape>,
    ) -> Flow {
        let selected = if let Some(subject) = subject {
            if subject.is_empty() {
                return self.unresolvable(view, "the subject identity is empty");
            }
            match self.resolve_all(subject) {
                Ok(selected) => selected,
                Err(reason) => return self.unresolvable(view, &reason),
            }
        } else {
            match self.snapshots.get(view) {
                Some((selected, _)) => selected.clone(),
                None => {
                    return self.unresolvable(view, "no subject snapshot preceded this assertion");
                }
            }
        };
        let rows: Vec<Value> = match &self.last_view {
            Some((queried, rows)) if queried == view => rows
                .iter()
                .filter(|row| {
                    selected.iter().all(|(field, value)| {
                        row.member(field).is_some_and(|held| same(held, value))
                    })
                })
                .cloned()
                .collect(),
            Some(_) => return self.unresolvable(view, "the last query names a different view"),
            None => {
                return self.unresolvable(
                    view,
                    "no consistent view query preceded the snapshot assertion",
                );
            }
        };
        let matches = rows.len() == 1;
        if let Some(shape) = shape.filter(|_| matches)
            && let Err(reason) = shape.admit_row(&rows[0])
        {
            self.failed(format!(
                "complete subject snapshot {view}: expected an actual row satisfying its \
                 complete declared shape; {reason}"
            ));
            return Flow::Stop;
        }
        let unchanged = matches
            && (subject.is_some()
                || self
                    .snapshots
                    .get(view)
                    .is_some_and(|(_, prior)| same(prior, &rows[0])));
        if unchanged {
            if subject.is_some() {
                self.snapshots
                    .insert(view.to_owned(), (selected, rows[0].clone()));
            }
            self.passed();
        } else {
            let prior = self
                .snapshots
                .get(view)
                .map(|(_, prior)| format!(", before {}", quote(prior)))
                .unwrap_or_default();
            self.failed(format!(
                "subject snapshot {view}: expected exactly one subject row, unchanged after a \
                 preserving command; {} matching row(s) {}{prior}",
                rows.len(),
                quote(&Value::Array(rows.clone()))
            ));
            if subject.is_some() {
                return Flow::Stop;
            }
        }
        Flow::Continue
    }

    fn expect_absent(&mut self, view: &str, subject: &[(String, ScenarioValue)]) -> Flow {
        if subject.is_empty() {
            return self.unresolvable(view, "the subject identity is empty");
        }
        let selected = match self.resolve_all(subject) {
            Ok(selected) => selected,
            Err(reason) => return self.unresolvable(view, &reason),
        };
        let remaining = match &self.last_view {
            Some((queried, rows)) if queried == view => rows
                .iter()
                .filter(|row| {
                    selected.iter().all(|(field, value)| {
                        row.member(field).is_some_and(|held| same(held, value))
                    })
                })
                .count(),
            Some(_) => {
                return self.unresolvable(
                    view,
                    "the last query before `expect_subject_absent` names a different view",
                );
            }
            None => {
                return self.unresolvable(
                    view,
                    "no consistent view query preceded `expect_subject_absent`",
                );
            }
        };
        if remaining == 0 {
            self.passed();
        } else {
            self.failed(format!(
                "the removed subject is absent from {view}: expected no row to carry {}; \
                 {remaining} row(s) still carry it",
                quote_fields(&selected)
            ));
        }
        Flow::Continue
    }

    fn direct_response(&self, response: &DirectResponse) -> Result<(), String> {
        let executed = self.last_command.as_ref().ok_or("no preceding command")?;
        if executed.command != response.command
            || response
                .outcome
                .as_ref()
                .is_some_and(|outcome| executed.outcome() != Some(outcome.as_str()))
        {
            return Err(format!(
                "direct response names a different command or outcome ({})",
                outcome_seen(executed)
            ));
        }
        if executed.error().is_some() {
            return Err(
                "command returned an error instead of the declared successful response".to_owned(),
            );
        }
        let actual = executed
            .answer
            .as_ref()
            .and_then(|answer| answer.response.as_ref())
            .ok_or("command returned no response")?;
        let Value::Object(members) = actual else {
            return Err(format!(
                "the response is {}, not an object",
                actual.describes()
            ));
        };
        if let Some((name, _)) = members
            .iter()
            .find(|(name, _)| !response.fields.iter().any(|field| &field.name == name))
        {
            return Err(format!("response has an undeclared field `{name}`"));
        }
        for field in &response.fields {
            shape::validate(
                &field.type_ref,
                actual.member(&field.name),
                &response.declarations,
                Profile::DirectResponse,
            )
            .map_err(|reason| format!("response field {}: {reason}", field.name))?;
            field.present(actual.member(&field.name))?;
        }
        if quote(actual).len() > 1_048_576 {
            return Err("direct response resource byte limit".to_owned());
        }
        for (name, expected) in &response.expected {
            if !actual.member(name).is_some_and(|held| same(held, expected)) {
                return Err(format!(
                    "response field {name} differs from its declared literal"
                ));
            }
        }
        Ok(())
    }

    fn response_payload(&self, response: &ResponsePayload) -> Result<(), String> {
        let executed = self.last_command.as_ref().ok_or("no preceding command")?;
        if executed.command != response.command
            || executed.outcome() != Some(response.outcome.as_str())
        {
            return Err(format!(
                "response observation names a different command or outcome ({})",
                outcome_seen(executed)
            ));
        }
        let payload = &executed
            .events
            .iter()
            .find(|event| event.name == response.event)
            .ok_or("same invocation did not emit the response-mapped event")?
            .payload;
        let actual = executed
            .answer
            .as_ref()
            .and_then(|answer| answer.response.as_ref())
            .ok_or("command returned no response")?;
        let Value::Object(members) = actual else {
            return Err(format!(
                "the response is {}, not an object",
                actual.describes()
            ));
        };
        if let Some((name, _)) = members
            .iter()
            .find(|(name, _)| !response.fields.iter().any(|field| &field.name == name))
        {
            return Err(format!("response has an undeclared field `{name}`"));
        }
        for field in &response.fields {
            shape::validate(
                &field.type_ref,
                actual.member(&field.name),
                &response.declarations,
                Profile::Response,
            )
            .map_err(|reason| format!("response field {}: {reason}", field.name))?;
        }
        if quote(actual).len() > 1_048_576 {
            return Err("response byte limit".to_owned());
        }
        for field in &response.fields {
            field.present(actual.member(&field.name))?;
        }
        for (target, source) in &response.mappings {
            let held = actual.member(source);
            let carried = payload.member(target);
            let optional = response.fields.iter().any(|field| {
                &field.name == source && matches!(field.type_ref, shape::TypeRef::Optional(_))
            });
            if optional
                && matches!(held, None | Some(Value::Null))
                && matches!(carried, None | Some(Value::Null))
            {
                continue;
            }
            let held = held.ok_or_else(|| format!("response source {source} is absent"))?;
            if !carried.is_some_and(|carried| same(carried, held)) {
                return Err(format!(
                    "event field {target} differs from actual response field {source}"
                ));
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn await_outcome<T: Target + ?Sized>(
        &mut self,
        command: &str,
        identity: &str,
        instance: &ScenarioValue,
        input: &[(String, ScenarioValue)],
        shape: &Shape,
        target: &mut T,
    ) -> Flow {
        let instance = match self.resolve(instance) {
            Ok(instance) => instance,
            Err(reason) => {
                self.errored(format!(
                    "the instance awaited of `{command}` refers to something an earlier step \
                     established, and {reason}"
                ));
                return Flow::Stop;
            }
        };
        let mut stated = Vec::new();
        for (field, value) in input {
            match self.resolve(value) {
                Ok(value) => stated.push((field.clone(), value)),
                Err(reason) => {
                    self.errored(format!(
                        "the input `{field}` awaited of `{command}` refers to something an \
                         earlier step established, and {reason}"
                    ));
                    return Flow::Stop;
                }
            }
        }
        let about = format!("{command} awaited on {}", quote(&instance));
        match target.await_outcome(command, &instance) {
            Err(error) => {
                self.target_failure(&format!("awaiting `{command}`"), &error);
                Flow::Stop
            }
            Ok(Awaited::NotIssued) => {
                self.failed(format!(
                    "{about}: expected an outcome of {command} on {}; nothing reached",
                    quote(&instance)
                ));
                Flow::Stop
            }
            Ok(Awaited::Reached { answer, input }) => {
                let events = match self.drain(target) {
                    Ok(events) => events,
                    Err(error) => {
                        self.target_failure(
                            &format!("observing what `{command}` published"),
                            &error,
                        );
                        return Flow::Stop;
                    }
                };
                let mut wrong = Vec::new();
                match input.member(identity) {
                    Some(carried) if same(carried, &instance) => {}
                    Some(carried) => wrong.push(format!(
                        "the answer is about {identity} = {}, not the awaited instance",
                        quote(carried)
                    )),
                    None => wrong.push(format!(
                        "the answer carries no `{identity}`, the input naming the instance"
                    )),
                }
                for (field, value) in &stated {
                    match input.member(field) {
                        Some(carried) if same(carried, value) => {}
                        Some(carried) => wrong.push(format!(
                            "{field} = {}, where the scenario states {}",
                            quote(carried),
                            quote(value)
                        )),
                        None => wrong.push(format!(
                            "{field} was not carried, where the scenario states {}",
                            quote(value)
                        )),
                    }
                }
                if wrong.is_empty() {
                    self.passed();
                } else {
                    self.failed(format!("{about}: {}", wrong.join("; ")));
                }
                if !shape.is_empty() {
                    let violations = shape_violations(&input, shape);
                    if violations.is_empty() {
                        self.passed();
                    } else {
                        self.failed(format!(
                            "input of {command}, as the system issued it: {}",
                            violations.join("; ")
                        ));
                    }
                }
                self.remember(&events);
                self.awaited = Some(input);
                self.last_command = Some(Executed {
                    command: command.to_owned(),
                    answer: Some(answer),
                    events,
                    not_granted: None,
                });
                Flow::Continue
            }
        }
    }

    fn expect_not_issued<T: Target + ?Sized>(
        &mut self,
        command: &str,
        instance: &ScenarioValue,
        unpublished: &[String],
        target: &mut T,
    ) -> Flow {
        let instance = match self.resolve(instance) {
            Ok(instance) => instance,
            Err(reason) => {
                self.errored(format!(
                    "the instance awaited of `{command}` refers to something an earlier step \
                     established, and {reason}"
                ));
                return Flow::Stop;
            }
        };
        if self.count_before(unpublished, target) == Flow::Stop {
            return Flow::Stop;
        }
        match target.await_outcome(command, &instance) {
            Err(error) => {
                self.target_failure(&format!("awaiting `{command}`"), &error);
                return Flow::Stop;
            }
            Ok(Awaited::Reached { answer, input }) => {
                let events = match self.drain(target) {
                    Ok(events) => events,
                    Err(error) => {
                        self.target_failure(
                            &format!("observing what `{command}` published"),
                            &error,
                        );
                        return Flow::Stop;
                    }
                };
                self.remember(&events);
                self.failed(format!(
                    "{command} not issued on {}: expected no outcome of {command}; the system \
                     issued it with {}: outcome = {}",
                    quote(&instance),
                    quote(&input),
                    answer.outcome
                ));
            }
            Ok(Awaited::NotIssued) => self.passed(),
        }
        self.nothing_published_after(unpublished, target)
    }
}

/// What a view expectation requires, resolved.
enum Required {
    Contains(Vec<(String, Value)>),
    Excludes(Vec<(String, Value)>),
    Counts {
        at_least: Option<usize>,
        at_most: Option<usize>,
    },
}

/// Why an authored claim on `event` is `unsupported`: the members its literal leaves out, by
/// dotted path, which the target carries. No value is quoted, so the reason is the same in every
/// run.
fn authored_literal(event: &str, left_out: &[String]) -> String {
    let mut leaves: Vec<String> = left_out.iter().map(|leaf| format!("`{leaf}`")).collect();
    leaves.sort();
    leaves.dedup();
    format!(
        "ESS 0.75.0: the authored claim on {event} writes a struct literal that leaves out {}, \
         which the target carries; the runner compares the literal whole and reads them as \
         claimed absent, while authoring reads them as unclaimed and refuses the dotted leaf key \
         synthesis writes for the same claim; every leaf the literal writes is held",
        leaves.join(", ")
    )
}

fn quote_fields(fields: &[(String, Value)]) -> String {
    quote(&Value::Object(fields.to_vec()))
}

/// What a report says a command answered when the target refused it as not granted.
fn not_granted_seen(named: Option<&str>) -> String {
    match named {
        Some(actor) => {
            format!("the standard refusal for an actor no grant admits, naming `{actor}`")
        }
        None => "the standard refusal for an actor no grant admits, naming no actor".to_owned(),
    }
}

/// What the last command came to, for a message.
fn outcome_seen(executed: &Executed) -> String {
    match (&executed.not_granted, executed.outcome()) {
        (Some(named), _) => not_granted_seen(named.as_deref()),
        (None, Some(outcome)) => format!("{} took outcome = {outcome}", executed.command),
        (None, None) => format!("{} reached no declared outcome", executed.command),
    }
}

/// Whether an occurrence carries every value a claim names and every leaf of its shape.
fn carries(carried: &Value, values: &EventValues, shape: &Shape) -> bool {
    values.values.iter().all(|(field, expected)| {
        carried_at(carried, field).is_some_and(|held| same(held, expected))
    }) && values
        .unissued
        .iter()
        .all(|field| carried_at(carried, field).is_none_or(|held| *held == Value::Null))
        && shape_violations(carried, shape).is_empty()
}

/// Every leaf of `shape` that `carried` does not hold as declared, described.
fn shape_violations(carried: &Value, shape: &Shape) -> Vec<String> {
    shape
        .iter()
        .filter_map(|(path, leaf)| {
            let reached = reach(carried, path);
            let admitted = match &reached {
                Reached::Value(value) => leaf.admits(Some(value)),
                Reached::Absent => leaf.admits(None),
                Reached::Blocked { .. } => false,
            };
            (!admitted).then(|| match reached {
                Reached::Value(value) => {
                    format!(
                        "{path} = {}, which is not {}",
                        quote(value),
                        leaf.describe()
                    )
                }
                Reached::Absent => {
                    format!("{path} was not carried, and it holds {}", leaf.describe())
                }
                Reached::Blocked { at, found } => {
                    format!("{at} holds {found}, so {path} is not there to read")
                }
            })
        })
        .collect()
}

/// The occurrence each claim of one event in one act takes, by claim, from `carries[claim]
/// [occurrence]` (`ess-conformance` 0.75.0, `event_multiplicity::assignment`).
///
/// A maximum matching of claims to distinct occurrences that carry them, so the order claims are
/// written in never decides how many are met. Where several maximum matchings exist, each claim in
/// turn takes the lowest occurrence some maximum matching gives it, given the choices before it.
/// Every claim left out then takes, in turn, the lowest occurrence nothing took, and `None` once
/// none is left.
pub(crate) fn assignment(carries: &[Vec<bool>], occurrences: usize) -> Vec<Option<usize>> {
    let claims = carries.len();
    let edge =
        |claim: usize, occurrence: usize| carries[claim].get(occurrence).copied().unwrap_or(false);
    let everyone: Vec<usize> = (0..claims).collect();
    let best = maximum(&edge, &everyone, &vec![false; occurrences]);
    let mut used = vec![false; occurrences];
    let mut taken = vec![None; claims];
    let mut fixed = 0;
    for (claim, slot) in taken.iter_mut().enumerate() {
        let rest: Vec<usize> = (claim + 1..claims).collect();
        for occurrence in 0..occurrences {
            if used[occurrence] || !edge(claim, occurrence) {
                continue;
            }
            used[occurrence] = true;
            if fixed + 1 + maximum(&edge, &rest, &used) == best {
                *slot = Some(occurrence);
                fixed += 1;
                break;
            }
            used[occurrence] = false;
        }
    }
    for slot in &mut taken {
        if slot.is_none()
            && let Some(free) = used.iter().position(|held| !held)
        {
            used[free] = true;
            *slot = Some(free);
        }
    }
    taken
}

/// The size of a maximum matching of `claims` to occurrences not `blocked` (augmenting paths).
fn maximum(edge: &impl Fn(usize, usize) -> bool, claims: &[usize], blocked: &[bool]) -> usize {
    fn augment(
        claim: usize,
        edge: &impl Fn(usize, usize) -> bool,
        blocked: &[bool],
        seen: &mut [bool],
        owner: &mut [Option<usize>],
    ) -> bool {
        for occurrence in 0..blocked.len() {
            if blocked[occurrence] || seen[occurrence] || !edge(claim, occurrence) {
                continue;
            }
            seen[occurrence] = true;
            if owner[occurrence].is_none_or(|held| augment(held, edge, blocked, seen, owner)) {
                owner[occurrence] = Some(claim);
                return true;
            }
        }
        false
    }
    let mut owner = vec![None; blocked.len()];
    claims
        .iter()
        .filter(|claim| {
            let mut seen = vec![false; blocked.len()];
            augment(**claim, edge, blocked, &mut seen, &mut owner)
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::assignment;

    #[test]
    fn the_order_claims_are_written_in_does_not_decide_the_matching() {
        assert_eq!(
            assignment(&[vec![true, true], vec![true, false]], 2),
            vec![Some(1), Some(0)]
        );
        assert_eq!(
            assignment(&[vec![true, false], vec![true, true]], 2),
            vec![Some(0), Some(1)]
        );
    }

    #[test]
    fn a_claim_left_out_takes_the_first_occurrence_nothing_took_then_none() {
        assert_eq!(
            assignment(&[vec![true, false], vec![true, false]], 2),
            vec![Some(0), Some(1)]
        );
        assert_eq!(
            assignment(&[vec![true, true], vec![true, true], vec![true, true]], 2),
            vec![Some(0), Some(1), None]
        );
        assert_eq!(assignment(&[vec![]], 0), vec![None]);
    }

    #[test]
    fn among_maximum_matchings_each_claim_takes_its_lowest_occurrence() {
        assert_eq!(
            assignment(&[vec![true, true, true], vec![true, true, false]], 3),
            vec![Some(0), Some(1)]
        );
        assert_eq!(
            assignment(&[vec![false, true, true], vec![false, true, false]], 3),
            vec![Some(2), Some(1)]
        );
    }
}
