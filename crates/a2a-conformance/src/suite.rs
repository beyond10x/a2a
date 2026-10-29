//! An `ess-conformance/52` suite as read: its scenarios, each a list of typed steps, with the
//! compact spellings (`absent_events`, `unpublished_events`) expanded against the suite's event
//! catalog as `ess-conformance` 0.75.0 expands them (`compact.rs`).
//!
//! A scenario that cannot be read (a step kind or a field this interpreter does not know, a value
//! of the wrong kind) is kept with the reason, and the run reports it `error`: nothing in a suite
//! is skipped.

use a2a::json::{self, Value};

use crate::shape::{self, Declarations, Field, Shape, SubjectShape};
use crate::value::quote;

/// The suite version this interpreter runs.
pub const SUITE_VERSION: &str = "ess-conformance/52";

/// A synthesized conformance suite.
#[derive(Debug, Clone)]
pub struct Suite {
    pub(crate) scenarios: Vec<(String, Result<Scenario, String>)>,
}

impl Suite {
    /// Reads a suite document.
    ///
    /// # Errors
    ///
    /// The document is not JSON, is not an `ess-conformance/52` suite, or its scenarios are not
    /// an object. A scenario that cannot be read is kept, and runs as `error`.
    pub fn parse(text: &str) -> Result<Self, String> {
        let document =
            json::parse_written(text).map_err(|error| format!("the suite is not JSON: {error}"))?;
        let mut root = Fields::of(&document, "a suite")?;
        let provenance = root.required("provenance")?;
        match provenance.member("suite_version") {
            Some(Value::Text(version)) if version == SUITE_VERSION => {}
            Some(other) => {
                return Err(format!(
                    "the suite is {}, and a2a-conformance runs {SUITE_VERSION}",
                    quote(other)
                ));
            }
            None => return Err("the suite's provenance names no suite_version".to_owned()),
        }
        let events = root
            .optional("events")
            .map(|value| texts(&value, "events"))
            .transpose()?;
        let Value::Object(scenarios) = root.required("scenarios")? else {
            return Err("the suite's scenarios are an object keyed by scenario id".to_owned());
        };
        root.done()?;
        let scenarios = scenarios
            .into_iter()
            .map(|(id, body)| {
                let scenario = Scenario::read(&body, events.as_deref());
                (id, scenario)
            })
            .collect();
        Ok(Self { scenarios })
    }

    /// Every scenario id, in suite order.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.scenarios.iter().map(|(id, _)| id.as_str())
    }

    /// How many scenarios the suite holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.scenarios.len()
    }

    /// Whether the suite holds no scenario.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.scenarios.is_empty()
    }
}

/// One scenario's steps, expanded.
#[derive(Debug, Clone)]
pub(crate) struct Scenario {
    pub(crate) steps: Vec<Step>,
}

impl Scenario {
    fn read(body: &Value, events: Option<&[String]>) -> Result<Self, String> {
        let mut fields = Fields::of(body, "a scenario")?;
        fields.optional("purpose");
        fields.optional("source");
        let written = match fields.required("steps")? {
            Value::Array(steps) => steps
                .iter()
                .enumerate()
                .map(|(index, step)| {
                    Step::read(step).map_err(|error| format!("step {index}: {error}"))
                })
                .collect::<Result<Vec<_>, _>>()?,
            other => return Err(format!("steps are a list, not {}", other.describes())),
        };
        let unpublished = fields.optional("unpublished_events");
        let absent = fields.optional("absent_events");
        fields.done()?;
        let mut steps = written;
        if let Some(patches) = unpublished {
            let events =
                events.ok_or("an unpublished-event list requires the suite event catalog")?;
            expand_unpublished(events, &mut steps, &patches)?;
        }
        if let Some(runs) = absent {
            let events = events.ok_or("an absent-event run requires the suite event catalog")?;
            steps = expand_absent(events, steps, &runs)?;
        }
        Ok(Self { steps })
    }
}

/// Whether `except` names catalog events in catalog order and leaves at least one out.
fn leaves_out(events: &[String], except: &[String]) -> bool {
    let mut last = None;
    for event in except {
        let Some(at) = events.iter().position(|declared| declared == event) else {
            return false;
        };
        if last.is_some_and(|last| at <= last) {
            return false;
        }
        last = Some(at);
    }
    except.len() < events.len()
}

fn expand_unpublished(
    events: &[String],
    steps: &mut [Step],
    patches: &Value,
) -> Result<(), String> {
    let Value::Array(patches) = patches else {
        return Err("unpublished_events is a list".to_owned());
    };
    if patches.is_empty() {
        return Err("an empty unpublished_events is written as none".to_owned());
    }
    let mut previous = None;
    for patch in patches {
        let mut fields = Fields::of(patch, "an unpublished-event list")?;
        let step = fields.index("step")?;
        let except = fields.texts("except")?;
        fields.done()?;
        if previous.is_some_and(|previous| step <= previous) {
            return Err("unpublished-event lists name written steps in ascending order".to_owned());
        }
        previous = Some(step);
        let Some(Step::ExpectNotGranted { unpublished, .. }) = steps.get_mut(step) else {
            return Err("an unpublished-event list names an expect_not_granted step".to_owned());
        };
        if !unpublished.is_empty() || !leaves_out(events, &except) {
            return Err(
                "an unpublished-event list completes a step written without one, leaving out \
                 catalog events in catalog order and keeping one"
                    .to_owned(),
            );
        }
        *unpublished = events
            .iter()
            .filter(|event| !except.contains(event))
            .cloned()
            .collect();
    }
    Ok(())
}

fn expand_absent(events: &[String], written: Vec<Step>, runs: &Value) -> Result<Vec<Step>, String> {
    let Value::Array(runs) = runs else {
        return Err("absent_events is a list".to_owned());
    };
    if runs.is_empty() {
        return Err("an empty absent_events is written as none".to_owned());
    }
    let mut read = Vec::new();
    for run in runs {
        let mut fields = Fields::of(run, "an absent-event run")?;
        let before = fields.index("before")?;
        let except = fields.texts("except")?;
        fields.done()?;
        read.push((before, except));
    }
    if read.windows(2).any(|pair| pair[1].0 < pair[0].0)
        || read
            .last()
            .is_some_and(|(before, _)| *before > written.len())
    {
        return Err("absent-event runs stand at ascending positions within the steps".to_owned());
    }
    let mut steps = Vec::with_capacity(written.len() + read.len() * events.len());
    let mut written = written.into_iter().enumerate().peekable();
    for (before, except) in read {
        while written.peek().is_some_and(|(index, _)| *index < before) {
            steps.push(written.next().expect("peeked").1);
        }
        if !leaves_out(events, &except) {
            return Err(
                "an absent-event run leaves out catalog events in catalog order and keeps one"
                    .to_owned(),
            );
        }
        steps.extend(
            events
                .iter()
                .filter(|event| !except.contains(event))
                .map(|event| Step::ExpectNoEvent {
                    event: event.clone(),
                }),
        );
    }
    steps.extend(written.map(|(_, step)| step));
    Ok(steps)
}

/// A reference a step makes to a value: written out, or bound by an earlier step.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ScenarioValue {
    Literal(Value),
    Instance(String),
    Observed { event: String, field: String },
    Awaited(String),
    List(Vec<ScenarioValue>),
    Members(Vec<(String, ScenarioValue)>),
}

impl ScenarioValue {
    fn read(value: &Value) -> Result<Self, String> {
        let mut fields = Fields::of(value, "a scenario value")?;
        let read = match fields.text("kind")?.as_str() {
            "literal" => Self::Literal(fields.required("value")?),
            "instance" => Self::Instance(fields.text("instance")?),
            "observed" => Self::Observed {
                event: fields.text("event")?,
                field: fields.text("field")?,
            },
            "awaited" => Self::Awaited(fields.text("field")?),
            "list" => match fields.required("items")? {
                Value::Array(items) => {
                    Self::List(items.iter().map(Self::read).collect::<Result<_, _>>()?)
                }
                other => return Err(format!("list items are a list, not {}", other.describes())),
            },
            "members" => Self::Members(read_values(Some(fields.required("members")?))?),
            other => {
                return Err(format!(
                    "a scenario value of kind `{other}` is not interpreted by a2a-conformance"
                ));
            }
        };
        fields.done()?;
        Ok(read)
    }
}

/// An object of scenario values by field name, in written order; none where `value` is absent.
fn read_values(value: Option<Value>) -> Result<Vec<(String, ScenarioValue)>, String> {
    match value {
        None => Ok(Vec::new()),
        Some(Value::Object(members)) => members
            .into_iter()
            .map(|(name, value)| ScenarioValue::read(&value).map(|read| (name, read)))
            .collect(),
        Some(other) => Err(format!(
            "values are an object by field name, not {}",
            other.describes()
        )),
    }
}

/// What a view expectation requires of the rows read.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expectation {
    Contains(Vec<(String, ScenarioValue)>),
    Excludes(Vec<(String, ScenarioValue)>),
    Counts {
        at_least: Option<usize>,
        at_most: Option<usize>,
    },
}

impl Expectation {
    fn read(value: &Value) -> Result<Self, String> {
        let mut fields = Fields::of(value, "a view expectation")?;
        let read = match fields.text("expect")?.as_str() {
            "contains" => Self::Contains(read_values(Some(fields.required("fields")?))?),
            "excludes" => Self::Excludes(read_values(Some(fields.required("fields")?))?),
            "counts" => Self::Counts {
                at_least: fields.optional_index("at_least")?,
                at_most: fields.optional_index("at_most")?,
            },
            other => {
                return Err(format!(
                    "a view expectation `{other}` is not interpreted by a2a-conformance"
                ));
            }
        };
        fields.done()?;
        Ok(read)
    }
}

/// A command's direct response as the suite declares it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DirectResponse {
    pub(crate) command: String,
    pub(crate) outcome: Option<String>,
    pub(crate) fields: Vec<Field>,
    pub(crate) declarations: Declarations,
    pub(crate) expected: Vec<(String, Value)>,
}

impl DirectResponse {
    fn read(value: &Value) -> Result<Self, String> {
        let mut fields = Fields::of(value, "a direct response")?;
        let command = fields.text("command")?;
        let outcome = fields
            .optional("outcome")
            .map(|outcome| outcome_of(&outcome, &command))
            .transpose()?;
        let declared = shape::read_fields(fields.required("fields")?)?;
        let declarations = shape::read_declarations(fields.required("declarations")?)?;
        let expected = match fields.optional("expected") {
            None => Vec::new(),
            Some(Value::Object(members)) => members,
            Some(other) => {
                return Err(format!(
                    "expected literals are an object, not {}",
                    other.describes()
                ));
            }
        };
        fields.done()?;
        Ok(Self {
            command,
            outcome,
            fields: declared,
            declarations,
            expected,
        })
    }
}

/// An event that maps a command's response, as the suite declares it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResponsePayload {
    pub(crate) command: String,
    pub(crate) outcome: String,
    pub(crate) event: String,
    pub(crate) fields: Vec<Field>,
    pub(crate) declarations: Declarations,
    pub(crate) mappings: Vec<(String, String)>,
}

impl ResponsePayload {
    fn read(value: &Value) -> Result<Self, String> {
        let mut fields = Fields::of(value, "a response payload")?;
        let command = fields.text("command")?;
        let outcome = outcome_of(&fields.required("outcome")?, &command)?;
        let event = fields.text("event")?;
        let declared = shape::read_fields(fields.required("fields")?)?;
        let declarations = shape::read_declarations(fields.required("declarations")?)?;
        let mappings = match fields.required("mappings")? {
            Value::Object(members) => members
                .into_iter()
                .map(|(target, source)| match source {
                    Value::Text(source) => Ok((target, source)),
                    other => Err(format!(
                        "a mapping names its source field, not {}",
                        other.describes()
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?,
            other => return Err(format!("mappings are an object, not {}", other.describes())),
        };
        let targets = shape::read_fields(fields.required("targets")?)?;
        fields.done()?;
        if mappings.is_empty() || mappings.len() != targets.len() {
            return Err("response contract field/declaration bound".to_owned());
        }
        if targets
            .iter()
            .any(|target| !mappings.iter().any(|(name, _)| *name == target.name))
            || mappings
                .iter()
                .any(|(_, source)| !declared.iter().any(|field| &field.name == source))
        {
            return Err("response mapping has no declared source/target".to_owned());
        }
        Ok(Self {
            command,
            outcome,
            event,
            fields: declared,
            declarations,
            mappings,
        })
    }
}

/// One step of a scenario.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Step {
    ExecuteCommand {
        command: String,
        actor: Option<String>,
        input: Vec<(String, ScenarioValue)>,
    },
    AwaitOutcome {
        command: String,
        identity: String,
        instance: ScenarioValue,
        input: Vec<(String, ScenarioValue)>,
        shape: Shape,
    },
    ExpectNotIssued {
        command: String,
        instance: ScenarioValue,
        unpublished: Vec<String>,
    },
    ConfigureExternalOutcome {
        command: String,
        outcome: String,
    },
    CaptureInstance {
        instance: String,
        entity: String,
        event: String,
        field: String,
    },
    ExpectOutcome {
        command: String,
        outcome: String,
    },
    ExpectNotGranted {
        actor: Option<String>,
        unpublished: Vec<String>,
    },
    ExpectNoEvents,
    ExpectNoError,
    ExpectError {
        error: String,
    },
    ExpectNoEvent {
        event: String,
    },
    ExpectEvent {
        event: String,
        payload: Vec<(String, Value)>,
        shape: Shape,
    },
    ExpectEventValues {
        event: String,
        payload: Vec<(String, ScenarioValue)>,
        shape: Shape,
    },
    ReadAs {
        actor: Option<String>,
    },
    QueryView {
        view: String,
        params: Vec<(String, ScenarioValue)>,
    },
    ExpectView {
        view: String,
        expectation: Expectation,
    },
    SnapshotSubject {
        view: String,
        subject: Vec<(String, ScenarioValue)>,
    },
    SnapshotCompleteSubject {
        view: String,
        subject: Vec<(String, ScenarioValue)>,
        shape: SubjectShape,
    },
    ExpectSubjectUnchanged {
        view: String,
    },
    ExpectCompleteSubjectUnchanged {
        view: String,
    },
    ExpectSubjectAbsent {
        view: String,
        subject: Vec<(String, ScenarioValue)>,
    },
    ExpectDirectResponse(DirectResponse),
    ExpectResponsePayload(ResponsePayload),
}

/// The outcome an `{command, outcome}` reference names, which must be of `command`.
fn outcome_of(value: &Value, command: &str) -> Result<String, String> {
    let mut fields = Fields::of(value, "an outcome reference")?;
    let of = fields.text("command")?;
    let outcome = fields.text("outcome")?;
    fields.done()?;
    if of != command {
        return Err("response outcome belongs to a different command".to_owned());
    }
    Ok(outcome)
}

/// An optional text field that may be written `null`.
fn optional_name(fields: &mut Fields, name: &str) -> Result<Option<String>, String> {
    match fields.optional(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Text(text)) => Ok(Some(text)),
        Some(other) => Err(format!("`{name}` is a name, not {}", other.describes())),
    }
}

/// A field this interpreter does not carry out, refused where it is written with a value.
fn not_interpreted(fields: &mut Fields, name: &str, step: &str) -> Result<(), String> {
    match fields.optional(name) {
        None | Some(Value::Null) => Ok(()),
        Some(Value::Object(members)) if members.is_empty() => Ok(()),
        Some(_) => Err(format!(
            "`{name}` of {step} is not interpreted by a2a-conformance"
        )),
    }
}

impl Step {
    fn read(value: &Value) -> Result<Self, String> {
        let mut fields = Fields::of(value, "a step")?;
        let kind = fields.text("step")?;
        let step = match kind.as_str() {
            "execute_command" => {
                not_interpreted(&mut fields, "caller", &kind)?;
                Self::ExecuteCommand {
                    command: fields.text("command")?,
                    actor: optional_name(&mut fields, "actor")?,
                    input: read_values(fields.optional("input"))?,
                }
            }
            "await_outcome" => {
                not_interpreted(&mut fields, "within", &kind)?;
                Self::AwaitOutcome {
                    command: fields.text("command")?,
                    identity: fields.text("identity")?,
                    instance: ScenarioValue::read(&fields.required("instance")?)?,
                    input: read_values(fields.optional("input"))?,
                    shape: shape::read_shape(fields.optional("shape"))?,
                }
            }
            "expect_not_issued" => {
                not_interpreted(&mut fields, "within", &kind)?;
                Self::ExpectNotIssued {
                    command: fields.text("command")?,
                    instance: ScenarioValue::read(&fields.required("instance")?)?,
                    unpublished: fields.optional_texts("unpublished")?,
                }
            }
            "configure_external_outcome" => {
                not_interpreted(&mut fields, "times", &kind)?;
                let mut force = Fields::of(&fields.required("force")?, "a forced outcome")?;
                let command = force.text("command")?;
                let outcome = force.text("outcome")?;
                force.done()?;
                Self::ConfigureExternalOutcome { command, outcome }
            }
            "capture_instance" => Self::CaptureInstance {
                instance: fields.text("instance")?,
                entity: fields.text("entity")?,
                event: fields.text("event")?,
                field: fields.text("field")?,
            },
            "expect_outcome" => {
                let mut outcome = Fields::of(&fields.required("outcome")?, "an outcome reference")?;
                let command = outcome.text("command")?;
                let name = outcome.text("outcome")?;
                outcome.done()?;
                Self::ExpectOutcome {
                    command,
                    outcome: name,
                }
            }
            "expect_not_granted" => Self::ExpectNotGranted {
                actor: optional_name(&mut fields, "actor")?,
                unpublished: fields.optional_texts("unpublished")?,
            },
            "expect_no_events" => Self::ExpectNoEvents,
            "expect_no_error" => Self::ExpectNoError,
            "expect_error" => {
                not_interpreted(&mut fields, "fields", &kind)?;
                Self::ExpectError {
                    error: fields.text("error")?,
                }
            }
            "expect_no_event" => Self::ExpectNoEvent {
                event: fields.text("event")?,
            },
            "expect_event" => Self::ExpectEvent {
                event: fields.text("event")?,
                payload: match fields.optional("payload") {
                    None => Vec::new(),
                    Some(Value::Object(members)) => members,
                    Some(other) => {
                        return Err(format!(
                            "an event payload is an object, not {}",
                            other.describes()
                        ));
                    }
                },
                shape: shape::read_shape(fields.optional("shape"))?,
            },
            "expect_event_values" => Self::ExpectEventValues {
                event: fields.text("event")?,
                payload: read_values(fields.optional("payload"))?,
                shape: shape::read_shape(fields.optional("shape"))?,
            },
            "read_as" => Self::ReadAs {
                actor: optional_name(&mut fields, "actor")?,
            },
            "query_view" => Self::QueryView {
                view: fields.text("view")?,
                params: read_values(fields.optional("params"))?,
            },
            "expect_view" => Self::ExpectView {
                view: fields.text("view")?,
                expectation: Expectation::read(&fields.required("expectation")?)?,
            },
            "snapshot_subject" => Self::SnapshotSubject {
                view: fields.text("view")?,
                subject: read_values(Some(fields.required("subject")?))?,
            },
            "snapshot_complete_subject" => Self::SnapshotCompleteSubject {
                view: fields.text("view")?,
                subject: read_values(Some(fields.required("subject")?))?,
                shape: SubjectShape::read(&fields.required("shape")?)?,
            },
            "expect_subject_unchanged" => Self::ExpectSubjectUnchanged {
                view: fields.text("view")?,
            },
            "expect_complete_subject_unchanged" => Self::ExpectCompleteSubjectUnchanged {
                view: fields.text("view")?,
            },
            "expect_subject_absent" => Self::ExpectSubjectAbsent {
                view: fields.text("view")?,
                subject: read_values(Some(fields.required("subject")?))?,
            },
            "expect_direct_response" => {
                Self::ExpectDirectResponse(DirectResponse::read(&fields.required("response")?)?)
            }
            "expect_response_payload" => {
                Self::ExpectResponsePayload(ResponsePayload::read(&fields.required("response")?)?)
            }
            other => {
                return Err(format!(
                    "the step kind `{other}` is not interpreted by a2a-conformance"
                ));
            }
        };
        fields.done()?;
        Ok(step)
    }
}

/// A list of names.
fn texts(value: &Value, what: &str) -> Result<Vec<String>, String> {
    match value {
        Value::Array(items) => items
            .iter()
            .map(|item| match item {
                Value::Text(text) => Ok(text.clone()),
                other => Err(format!("{what} lists names, not {}", other.describes())),
            })
            .collect(),
        other => Err(format!(
            "{what} is a list of names, not {}",
            other.describes()
        )),
    }
}

/// The members of one suite object, taken one by one; [`Fields::done`] refuses any left over, so
/// a field this interpreter does not know makes the object unreadable rather than ignored.
pub(crate) struct Fields {
    what: &'static str,
    members: Vec<(String, Value)>,
}

impl Fields {
    pub(crate) fn of(value: &Value, what: &'static str) -> Result<Self, String> {
        match value {
            Value::Object(members) => Ok(Self {
                what,
                members: members.clone(),
            }),
            other => Err(format!("{what} is an object, not {}", other.describes())),
        }
    }

    pub(crate) fn optional(&mut self, name: &str) -> Option<Value> {
        crate::value::take(&mut self.members, name)
    }

    pub(crate) fn required(&mut self, name: &str) -> Result<Value, String> {
        self.optional(name)
            .ok_or_else(|| format!("{} has no `{name}`", self.what))
    }

    pub(crate) fn text(&mut self, name: &str) -> Result<String, String> {
        match self.required(name)? {
            Value::Text(text) => Ok(text),
            other => Err(format!(
                "`{name}` of {} is a text, not {}",
                self.what,
                other.describes()
            )),
        }
    }

    pub(crate) fn texts(&mut self, name: &str) -> Result<Vec<String>, String> {
        texts(&self.required(name)?, name)
    }

    pub(crate) fn optional_texts(&mut self, name: &str) -> Result<Vec<String>, String> {
        match self.optional(name) {
            None | Some(Value::Null) => Ok(Vec::new()),
            Some(value) => texts(&value, name),
        }
    }

    pub(crate) fn optional_bool(&mut self, name: &str) -> Result<Option<bool>, String> {
        match self.optional(name) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Bool(flag)) => Ok(Some(flag)),
            Some(other) => Err(format!("`{name}` is a boolean, not {}", other.describes())),
        }
    }

    fn index(&mut self, name: &str) -> Result<usize, String> {
        match self.required(name)? {
            Value::Number(spelling) => spelling
                .parse()
                .map_err(|_| format!("`{name}` is a count, not {spelling}")),
            other => Err(format!("`{name}` is a count, not {}", other.describes())),
        }
    }

    fn optional_index(&mut self, name: &str) -> Result<Option<usize>, String> {
        match self.optional(name) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Number(spelling)) => spelling
                .parse()
                .map(Some)
                .map_err(|_| format!("`{name}` is a count, not {spelling}")),
            Some(other) => Err(format!("`{name}` is a count, not {}", other.describes())),
        }
    }

    pub(crate) fn done(self) -> Result<(), String> {
        match self.members.first() {
            None => Ok(()),
            Some((name, _)) => Err(format!(
                "`{name}` of {} is not interpreted by a2a-conformance",
                self.what
            )),
        }
    }
}
