//! The names a value carries at the wire and in the specification, read from the contract the
//! generated server publishes (`generated/rust/a2a/src/server/a2a-agent.openapi.json`).
//!
//! A suite spells every field by its ESS name, at every depth (`task_id`, `status_message`,
//! `message_id`). The generated server reads and writes the wire names its binding declares:
//! nested structs and route parameters in camel case (`messageId`, `historyLength`), top-level
//! command inputs, event payloads and some view rows under their ESS names. The contract names
//! both for every property: the property's key is the wire name, and `x-ess-field` the ESS name
//! where it differs. [`Naming`] renames along the schema a value is declared by, so a target
//! over the wire can hand the interpreter what it expects.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use a2a::json::{self, Value};

/// The contract of the `a2a-agent` component, as the pinned `ess` generated it.
const CONTRACT: &str =
    include_str!("../../../generated/rust/a2a/src/server/a2a-agent.openapi.json");

/// How one schema spells its values.
#[derive(Debug, Clone)]
enum Form {
    /// As it is: a primitive, an enum, or `Json`, whose members are the caller's.
    Verbatim,
    /// An integer, written without a fraction toward the wire.
    Integer,
    /// The schema of this name.
    Named(String),
    /// An object of these fields.
    Object(Vec<Field>),
    /// A list of values of this form.
    List(Box<Form>),
    /// A map, its keys as they are and its values of this form.
    Map(Box<Form>),
}

#[derive(Debug, Clone)]
struct Field {
    wire: String,
    ess: String,
    form: Form,
}

/// Renames values between their ESS and wire spellings.
#[derive(Debug)]
pub struct Naming {
    schemas: BTreeMap<String, Form>,
    /// The parameters of each operation, by operation id: a command's path and query
    /// parameters, or a view's.
    parameters: BTreeMap<String, Vec<Field>>,
}

/// Which way a value is renamed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Toward {
    Wire,
    Ess,
}

impl Naming {
    /// The naming of the generated `a2a-agent` contract.
    ///
    /// # Panics
    ///
    /// The contract compiled in is not the JSON `ess` generates; a build of this crate holds it.
    pub fn contract() -> &'static Self {
        static NAMING: OnceLock<Naming> = OnceLock::new();
        NAMING.get_or_init(|| {
            Self::read(CONTRACT).expect("the generated contract names every schema it declares")
        })
    }

    /// Reads the naming of an OpenAPI contract `ess` generated.
    ///
    /// # Errors
    ///
    /// The contract is not JSON, or declares no schemas.
    pub fn read(contract: &str) -> Result<Self, String> {
        let document =
            json::parse_written(contract).map_err(|error| format!("not JSON: {error}"))?;
        let Some(Value::Object(schemas)) = document
            .member("components")
            .and_then(|components| components.member("schemas"))
        else {
            return Err("the contract declares no components.schemas".to_owned());
        };
        let schemas = schemas
            .iter()
            .map(|(name, schema)| (name.clone(), form(schema)))
            .collect();
        let mut parameters = BTreeMap::new();
        if let Some(Value::Object(paths)) = document.member("paths") {
            for (_, operations) in paths {
                let Value::Object(operations) = operations else {
                    continue;
                };
                for (_, operation) in operations {
                    let Some(Value::Text(id)) = operation.member("operationId") else {
                        continue;
                    };
                    let Some(Value::Array(declared)) = operation.member("parameters") else {
                        continue;
                    };
                    let fields: &mut Vec<Field> = parameters.entry(id.clone()).or_default();
                    for parameter in declared {
                        let Some(Value::Text(wire)) = parameter.member("name") else {
                            continue;
                        };
                        let schema = parameter.member("schema").unwrap_or(&Value::Null);
                        fields.push(Field {
                            wire: wire.clone(),
                            ess: ess_name(schema).unwrap_or(wire).clone(),
                            form: form(schema),
                        });
                    }
                }
            }
        }
        Ok(Self {
            schemas,
            parameters,
        })
    }

    /// The input of the command or view `name`, ESS-named, as its route reads it: the command's
    /// declared input and its route's parameters, or the view's parameters.
    #[must_use]
    pub fn input_to_wire(&self, name: &str, input: &Value) -> Value {
        self.rename(&self.input(name), input, Toward::Wire)
    }

    /// The input of the command `name`, wire-named, as the specification names it.
    #[must_use]
    pub fn input_to_ess(&self, name: &str, input: &Value) -> Value {
        self.rename(&self.input(name), input, Toward::Ess)
    }

    /// The response of the command `name`, wire-named, as the specification names it.
    #[must_use]
    pub fn response_to_ess(&self, name: &str, response: &Value) -> Value {
        self.rename(
            &Form::Named(format!("{name}.Result")),
            response,
            Toward::Ess,
        )
    }

    /// The payload of the event `name`, wire-named, as the specification names it: every field it
    /// declares, one the wire left out as `null`.
    #[must_use]
    pub fn event_to_ess(&self, name: &str, payload: &Value) -> Value {
        self.complete(&format!("{name}.Event"), payload)
    }

    /// A row of the view `name`, wire-named, as the specification names it: every field it
    /// declares, one the wire left out as `null`.
    #[must_use]
    pub fn row_to_ess(&self, name: &str, row: &Value) -> Value {
        self.complete(&format!("{name}.Row"), row)
    }

    /// A row of the view `name`, ESS-named, as the wire spells it.
    #[must_use]
    pub fn row_to_wire(&self, name: &str, row: &Value) -> Value {
        self.rename(&Form::Named(format!("{name}.Row")), row, Toward::Wire)
    }

    /// `value` of the schema `name` renamed to ESS names, with every top-level field the schema
    /// declares and `value` leaves out as `null`.
    ///
    /// A field declared `omitted_when_absent` is left out on the wire where it holds nothing,
    /// which is what the specification spells `null`. The runner compares a top-level field
    /// exactly, and absent with `null` alike only below it, so the two spellings are made one
    /// here.
    fn complete(&self, name: &str, value: &Value) -> Value {
        let renamed = self.rename(&Form::Named(name.to_owned()), value, Toward::Ess);
        let (Some(Form::Object(fields)), Value::Object(mut members)) =
            (self.schemas.get(name), renamed.clone())
        else {
            return renamed;
        };
        for field in fields {
            if !members.iter().any(|(key, _)| *key == field.ess) {
                members.push((field.ess.clone(), Value::Null));
            }
        }
        Value::Object(members)
    }

    fn input(&self, name: &str) -> Form {
        let mut fields = match self.schemas.get(&format!("{name}.Input")) {
            Some(Form::Object(fields)) => fields.clone(),
            _ => Vec::new(),
        };
        if let Some(parameters) = self.parameters.get(name) {
            for parameter in parameters {
                if !fields.iter().any(|field| field.ess == parameter.ess) {
                    fields.push(parameter.clone());
                }
            }
        }
        Form::Object(fields)
    }

    fn rename(&self, form: &Form, value: &Value, toward: Toward) -> Value {
        match (form, value) {
            (Form::Named(name), _) => match self.schemas.get(name) {
                Some(form) => self.rename(form, value, toward),
                None => value.clone(),
            },
            (Form::Object(fields), Value::Object(members)) => Value::Object(
                members
                    .iter()
                    .map(|(key, member)| {
                        let field = fields.iter().find(|field| match toward {
                            Toward::Wire => field.ess == *key,
                            Toward::Ess => field.wire == *key,
                        });
                        match field {
                            Some(field) => (
                                match toward {
                                    Toward::Wire => field.wire.clone(),
                                    Toward::Ess => field.ess.clone(),
                                },
                                self.rename(&field.form, member, toward),
                            ),
                            None => (key.clone(), member.clone()),
                        }
                    })
                    .collect(),
            ),
            (Form::List(item), Value::Array(items)) => Value::Array(
                items
                    .iter()
                    .map(|each| self.rename(item, each, toward))
                    .collect(),
            ),
            (Form::Map(item), Value::Object(members)) => Value::Object(
                members
                    .iter()
                    .map(|(key, each)| (key.clone(), self.rename(item, each, toward)))
                    .collect(),
            ),
            (Form::Integer, Value::Number(number)) if toward == Toward::Wire => {
                Value::Number(integral(number).unwrap_or_else(|| number.clone()))
            }
            _ => value.clone(),
        }
    }
}

/// `number` without its fraction, where it is a whole number written with one (`1.0`).
// ESS gap (ess 0.75.0): authoring writes an authored integer literal (`history_length: 1`) into
// the suite as `1.0`, and the decoder ess generates for an `Integer` input refuses `1.0` ("expected
// an integer, found the number 1.0"), so a suite's whole number is written as the wire's integer.
fn integral(number: &str) -> Option<String> {
    let (whole, fraction) = number.split_once('.')?;
    let digits = whole.strip_prefix('-').unwrap_or(whole);
    let whole_number = !digits.is_empty()
        && digits.bytes().all(|byte| byte.is_ascii_digit())
        && !fraction.is_empty()
        && fraction.bytes().all(|byte| byte == b'0');
    whole_number.then(|| whole.to_owned())
}

/// The form a schema declares.
fn form(schema: &Value) -> Form {
    if let Some(Value::Text(reference)) = schema.member("$ref") {
        return match reference.strip_prefix("#/components/schemas/") {
            Some(name) => Form::Named(name.to_owned()),
            None => Form::Verbatim,
        };
    }
    if let Some(Value::Object(properties)) = schema.member("properties") {
        return Form::Object(
            properties
                .iter()
                .map(|(wire, property)| Field {
                    wire: wire.clone(),
                    ess: ess_name(property).unwrap_or(wire).clone(),
                    form: form(property),
                })
                .collect(),
        );
    }
    if let Some(items) = schema.member("items") {
        return Form::List(Box::new(form(items)));
    }
    match schema.member("additionalProperties") {
        Some(values @ Value::Object(_)) => Form::Map(Box::new(form(values))),
        _ if schema.member("type") == Some(&Value::Text("integer".to_owned())) => Form::Integer,
        _ => Form::Verbatim,
    }
}

fn ess_name(schema: &Value) -> Option<&String> {
    match schema.member("x-ess-field") {
        Some(Value::Text(name)) => Some(name),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(text: &str) -> Value {
        json::parse(text).expect("JSON")
    }

    #[test]
    fn renames_nested_structs_and_route_parameters() {
        let naming = Naming::contract();
        let input = value(
            r#"{"tenant":"t","message":{"message_id":"m","context_id":"c","role":"ROLE_USER","parts":[{"media_type":"text/plain","text":"x"}],"metadata":{"message_id":"kept"}}}"#,
        );
        let wire = naming.input_to_wire("a2a.tasks.SendMessage", &input);
        assert_eq!(
            wire,
            value(
                r#"{"tenant":"t","message":{"messageId":"m","contextId":"c","role":"ROLE_USER","parts":[{"mediaType":"text/plain","text":"x"}],"metadata":{"message_id":"kept"}}}"#
            )
        );
        assert_eq!(naming.input_to_ess("a2a.tasks.SendMessage", &wire), input);
        assert_eq!(
            naming.input_to_wire(
                "a2a.tasks.GetTask",
                &value(r#"{"id":"t","history_length":2}"#)
            ),
            value(r#"{"id":"t","historyLength":2}"#)
        );
        assert_eq!(
            naming.input_to_wire("a2a.tasks.Tasks", &value(r#"{"context_id":"c"}"#)),
            value(r#"{"contextId":"c"}"#)
        );
    }

    #[test]
    fn writes_a_whole_number_of_an_integer_input_without_its_fraction() {
        let naming = Naming::contract();
        let wire = |input: &str| naming.input_to_wire("a2a.tasks.GetTask", &value(input));
        assert_eq!(
            wire(r#"{"id":"t","history_length":1.0}"#),
            value(r#"{"id":"t","historyLength":1}"#)
        );
        assert_eq!(
            wire(r#"{"id":"t","history_length":-2.00}"#),
            value(r#"{"id":"t","historyLength":-2}"#)
        );
        assert_eq!(
            wire(r#"{"id":"t","history_length":1.5}"#),
            value(r#"{"id":"t","historyLength":1.5}"#)
        );
        assert_eq!(
            naming.input_to_ess(
                "a2a.tasks.GetTask",
                &value(r#"{"id":"t","historyLength":1}"#)
            ),
            value(r#"{"id":"t","history_length":1}"#)
        );
    }

    #[test]
    fn renames_responses_events_and_rows() {
        let naming = Naming::contract();
        assert_eq!(
            naming.response_to_ess(
                "a2a.tasks.SendMessage",
                &value(r#"{"message":{"messageId":"m","role":"ROLE_AGENT","parts":[]}}"#)
            ),
            value(r#"{"reply":{"message_id":"m","role":"ROLE_AGENT","parts":[]}}"#)
        );
        assert_eq!(
            naming.event_to_ess(
                "a2a.tasks.TaskStatusUpdated",
                &value(
                    r#"{"task_id":"t","context_id":"c","status":{"state":"TASK_STATE_WORKING","message":{"messageId":"m","role":"ROLE_AGENT","parts":[]}}}"#
                )
            ),
            value(
                r#"{"task_id":"t","context_id":"c","status":{"state":"TASK_STATE_WORKING","message":{"message_id":"m","role":"ROLE_AGENT","parts":[]}}}"#
            )
        );
        assert_eq!(
            naming.row_to_ess(
                "a2a.tasks.Tasks",
                &value(r#"{"id":"t","contextId":"c","state":"Working"}"#)
            ),
            value(
                r#"{"id":"t","context_id":"c","state":"Working","status_message":null,"status_timestamp":null,"artifacts":null,"history":null,"metadata":null}"#
            )
        );
    }
}
