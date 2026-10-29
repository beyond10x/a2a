//! Reading and comparing JSON values as the ESS runner does (`ess-conformance` 0.75.0,
//! `runner.rs`: `reach_into`, `carried_at`, `same_value`).

use a2a::json::{self, Value};

/// What a dotted leaf path finds in an object.
pub(crate) enum Reached<'a> {
    /// A value sits there.
    Value(&'a Value),
    /// Nothing was carried under the path; a `null` on the way down is absence too.
    Absent,
    /// A prefix of the path holds something no field can be read out of.
    Blocked {
        /// The prefix.
        at: String,
        /// What it holds instead.
        found: &'static str,
    },
}

/// Walks the dotted `path` into `object`, segment by segment.
pub(crate) fn reach<'a>(object: &'a Value, path: &str) -> Reached<'a> {
    let mut walked = String::new();
    let mut at: Option<&Value> = None;
    for segment in path.split('.') {
        let here = match at {
            None => object.member(segment),
            Some(Value::Object(_)) => at.and_then(|value| value.member(segment)),
            Some(Value::Null) => return Reached::Absent,
            Some(value) => {
                return Reached::Blocked {
                    at: walked,
                    found: value.describes(),
                };
            }
        };
        if !walked.is_empty() {
            walked.push('.');
        }
        walked.push_str(segment);
        match here {
            Some(value) => at = Some(value),
            None => return Reached::Absent,
        }
    }
    at.map_or(Reached::Absent, Reached::Value)
}

/// The value `object` carries under `key`, a field name or the dotted path of a leaf.
pub(crate) fn carried_at<'a>(object: &'a Value, key: &str) -> Option<&'a Value> {
    match reach(object, key) {
        Reached::Value(value) => Some(value),
        Reached::Absent | Reached::Blocked { .. } => None,
    }
}

/// Whether two values are one value: objects by their member sets, arrays item by item, numbers by
/// their value and not their spelling.
pub(crate) fn same(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Object(left_members), Value::Object(right_members)) => {
            left_members.len() == right_members.len()
                && left_members
                    .iter()
                    .all(|(name, value)| right.member(name).is_some_and(|other| same(value, other)))
        }
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| same(left, right))
        }
        (Value::Number(left), Value::Number(right)) => same_number(left, right),
        _ => left == right,
    }
}

/// [`same`], where an object member that is absent and one that is `null` are one value at any
/// depth below the top (`same_value`): a view row compared field by field.
pub(crate) fn same_value(carried: &Value, wanted: &Value) -> bool {
    match (carried, wanted) {
        (Value::Object(carried_members), Value::Object(wanted_members)) => carried_members
            .iter()
            .map(|(name, _)| name)
            .chain(wanted_members.iter().map(|(name, _)| name))
            .all(|name| match (carried.member(name), wanted.member(name)) {
                (Some(carried), Some(wanted)) => same_value(carried, wanted),
                (None | Some(Value::Null), None | Some(Value::Null)) => true,
                _ => false,
            }),
        (Value::Array(carried), Value::Array(wanted)) => {
            carried.len() == wanted.len()
                && carried
                    .iter()
                    .zip(wanted)
                    .all(|(carried, wanted)| same_value(carried, wanted))
        }
        _ => same(carried, wanted),
    }
}

fn same_number(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    match (left.parse::<f64>(), right.parse::<f64>()) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

/// Whether `object` carries every field `wanted` names with that value, compared by `equal`.
/// A dotted path that finds nothing matches only a wanted `null`.
pub(crate) fn fields_match(
    object: &Value,
    wanted: &[(String, Value)],
    equal: impl Fn(&Value, &Value) -> bool,
) -> bool {
    wanted
        .iter()
        .all(|(field, value)| match carried_at(object, field) {
            Some(carried) => equal(carried, value),
            None => field.contains('.') && matches!(value, Value::Null),
        })
}

/// A value as JSON text, for a message.
pub(crate) fn quote(value: &Value) -> String {
    let mut text = String::new();
    json::push_value(&mut text, value);
    text
}

/// The member `name` of an object, removed; `None` where there is none.
pub(crate) fn take(members: &mut Vec<(String, Value)>, name: &str) -> Option<Value> {
    let at = members.iter().position(|(key, _)| key == name)?;
    Some(members.remove(at).1)
}

/// Reads the object literal `wanted` as a claim on each leaf it writes, not on the whole value: the
/// dotted paths, under `at`, of the members `carried` holds (not `null`) where `wanted` writes
/// none, or `None` where a leaf `wanted` writes is not held ([`same_value`]).
///
/// ESS 0.75.0 workaround, for an authored scenario only: see `interpret.rs`, `expect_payload`.
pub(crate) fn left_out_leaves(carried: &Value, wanted: &Value, at: &str) -> Option<Vec<String>> {
    let (Value::Object(carried_members), Value::Object(wanted_members)) = (carried, wanted) else {
        return same_value(carried, wanted).then(Vec::new);
    };
    let mut left_out = Vec::new();
    for (name, written) in wanted_members {
        match carried.member(name) {
            Some(held) => left_out.extend(left_out_leaves(held, written, &format!("{at}.{name}"))?),
            None if *written == Value::Null => {}
            None => return None,
        }
    }
    for (name, held) in carried_members {
        if *held != Value::Null && wanted.member(name).is_none() {
            left_out.push(format!("{at}.{name}"));
        }
    }
    Some(left_out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(text: &str) -> Value {
        json::parse(text).expect("the test value is JSON")
    }

    #[test]
    fn numbers_are_one_value_whatever_their_spelling() {
        assert!(same(&parsed("1"), &parsed("1.0")));
        assert!(!same(&parsed("1"), &parsed("2")));
    }

    #[test]
    fn objects_compare_as_member_sets() {
        assert!(same(
            &parsed(r#"{"a":1,"b":2}"#),
            &parsed(r#"{"b":2,"a":1}"#)
        ));
        assert!(!same(&parsed(r#"{"a":1}"#), &parsed(r#"{"a":1,"b":null}"#)));
        assert!(same_value(
            &parsed(r#"{"a":{"x":1}}"#),
            &parsed(r#"{"a":{"x":1,"y":null}}"#)
        ));
    }

    #[test]
    fn a_literal_read_leaf_by_leaf_names_what_it_leaves_out() {
        let carried = parsed(r#"{"state":"Open","at":"t","note":null,"by":{"id":1,"via":"x"}}"#);
        assert_eq!(
            left_out_leaves(&carried, &parsed(r#"{"state":"Open","by":{"id":1}}"#), "s"),
            Some(vec!["s.by.via".to_owned(), "s.at".to_owned()])
        );
        assert_eq!(
            left_out_leaves(&carried, &parsed(r#"{"state":"Closed"}"#), "s"),
            None
        );
        assert_eq!(
            left_out_leaves(&carried, &parsed(r#"{"gone":1}"#), "s"),
            None
        );
    }

    #[test]
    fn a_null_on_the_way_down_is_absence() {
        let object = parsed(r#"{"status":null,"count":3}"#);
        assert!(matches!(reach(&object, "status.state"), Reached::Absent));
        assert!(matches!(
            reach(&object, "count.value"),
            Reached::Blocked { .. }
        ));
    }
}
