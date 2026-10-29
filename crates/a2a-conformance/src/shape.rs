//! The value shapes a suite carries and how a value is held to them, as `ess-conformance` 0.75.0
//! holds them: leaf shapes of payloads (`scenario.rs`, `LeafShape`, `Holds`) and the declarations
//! a response or a complete view row is validated against (`selection.rs`,
//! `validate_value_inner`, in its `Response` and `DirectResponse` profiles).

use std::collections::BTreeMap;

use a2a::json::Value;

use crate::suite::Fields;

/// A primitive type of the specification language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Primitive {
    String,
    Boolean,
    Integer,
    Decimal,
    Binary64,
    Timestamp,
    Duration,
    Uuid,
    Bytes,
    Json,
}

impl Primitive {
    fn named(name: &str) -> Option<Self> {
        Some(match name.to_ascii_lowercase().as_str() {
            "string" => Self::String,
            "boolean" => Self::Boolean,
            "integer" => Self::Integer,
            "decimal" => Self::Decimal,
            "binary64" => Self::Binary64,
            "timestamp" => Self::Timestamp,
            "duration" => Self::Duration,
            "uuid" => Self::Uuid,
            "bytes" => Self::Bytes,
            "json" => Self::Json,
            _ => return None,
        })
    }

    /// Whether `value` is a value of this primitive (`ess-domain` `Primitive::admits`).
    pub(crate) fn admits(self, value: &Value) -> bool {
        match (self, value) {
            (Self::Json, _) => true,
            (Self::Boolean, Value::Bool(_)) => true,
            (Self::Decimal | Self::Binary64, Value::Number(_)) => true,
            (Self::Integer, Value::Number(spelling)) => integral(spelling),
            (Self::Uuid, Value::Text(text)) => canonical_uuid(text),
            (Self::Bytes, Value::Text(text)) => padded_base64(text),
            (Self::String | Self::Timestamp | Self::Duration, Value::Text(_)) => true,
            _ => false,
        }
    }
}

fn integral(spelling: &str) -> bool {
    spelling
        .parse::<f64>()
        .is_ok_and(|number| number.is_finite() && number.fract() == 0.0)
}

fn canonical_uuid(text: &str) -> bool {
    let mut parts = text.split('-');
    for width in [8_usize, 4, 4, 4, 12] {
        let Some(part) = parts.next() else {
            return false;
        };
        if part.len() != width || !part.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return false;
        }
    }
    parts.next().is_none()
}

fn padded_base64(text: &str) -> bool {
    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return false;
    }
    let padding = bytes.iter().rev().take_while(|byte| **byte == b'=').count();
    padding <= 2
        && bytes[..bytes.len() - padding]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'+' || *byte == b'/')
}

/// How a field may be left empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Presence {
    /// Sent as `null` when it has no value, never left out.
    NullWhenAbsent,
    /// Left out when it has no value, never sent as `null`.
    OmittedWhenAbsent,
}

fn presence(value: &Value) -> Result<Presence, String> {
    match value {
        Value::Text(text) if text == "null_when_absent" => Ok(Presence::NullWhenAbsent),
        Value::Text(text) if text == "omitted_when_absent" => Ok(Presence::OmittedWhenAbsent),
        other => Err(format!(
            "presence is `null_when_absent` or `omitted_when_absent`, not {}",
            crate::value::quote(other)
        )),
    }
}

/// What a leaf of a payload shape holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Holds {
    Primitive(Primitive),
    Enum(Vec<String>),
    List,
    Map,
    Union,
}

impl Holds {
    fn admits(&self, value: &Value) -> bool {
        match self {
            Self::Primitive(kind) => kind.admits(value),
            Self::Enum(variants) => {
                matches!(value, Value::Text(text) if variants.iter().any(|variant| variant == text))
            }
            Self::List => matches!(value, Value::Array(_)),
            Self::Map | Self::Union => matches!(value, Value::Object(_)),
        }
    }

    fn describe(&self) -> String {
        match self {
            Self::Primitive(kind) => format!("a {kind:?}"),
            Self::Enum(variants) => format!("one of {}", variants.join(", ")),
            Self::List => "a list".to_owned(),
            Self::Map => "a mapping".to_owned(),
            Self::Union => "a tagged union, as a mapping".to_owned(),
        }
    }
}

/// The shape one leaf of a payload is held to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Leaf {
    pub(crate) holds: Holds,
    pub(crate) optional: bool,
    pub(crate) presence: Option<Presence>,
}

impl Leaf {
    /// Whether the leaf admits what a payload carries there (`None`: nothing).
    pub(crate) fn admits(&self, value: Option<&Value>) -> bool {
        match value {
            None => self.optional && self.presence != Some(Presence::NullWhenAbsent),
            Some(Value::Null) => {
                self.optional && self.presence != Some(Presence::OmittedWhenAbsent)
            }
            Some(value) => self.holds.admits(value),
        }
    }

    /// What it holds, for a message.
    pub(crate) fn describe(&self) -> String {
        self.holds.describe()
    }

    fn read(value: &Value) -> Result<Self, String> {
        let mut fields = Fields::of(value, "a leaf shape")?;
        let holds = match fields.text("holds")?.as_str() {
            "primitive" => {
                let kind = fields.text("kind")?;
                Holds::Primitive(
                    Primitive::named(&kind).ok_or_else(|| format!("no primitive `{kind}`"))?,
                )
            }
            "enum" => Holds::Enum(fields.texts("variants")?),
            "list" => Holds::List,
            "map" => Holds::Map,
            "union" => Holds::Union,
            other => return Err(format!("no leaf shape holds `{other}`")),
        };
        let optional = fields.optional_bool("optional")?.unwrap_or(false);
        let presence = fields
            .optional("presence")
            .map(|value| presence(&value))
            .transpose()?;
        fields.done()?;
        Ok(Self {
            holds,
            optional,
            presence,
        })
    }
}

/// Each leaf path of a payload and its shape, in path order.
pub(crate) type Shape = Vec<(String, Leaf)>;

/// A payload shape from its suite form, an object of leaf paths.
pub(crate) fn read_shape(value: Option<Value>) -> Result<Shape, String> {
    match value {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Object(members)) => members
            .into_iter()
            .map(|(path, leaf)| Leaf::read(&leaf).map(|leaf| (path, leaf)))
            .collect(),
        Some(other) => Err(format!(
            "a shape is an object of leaf paths, not {}",
            other.describes()
        )),
    }
}

/// A type reference of a declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TypeRef {
    Primitive(Primitive),
    Named(String),
    Optional(Box<TypeRef>),
    List(Box<TypeRef>),
    Map(Primitive, Box<TypeRef>),
}

impl TypeRef {
    fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim();
        let inner = |prefix: &str| {
            text.strip_prefix(prefix)
                .and_then(|rest| rest.strip_suffix('>'))
        };
        if let Some(of) = inner("Optional<") {
            return Ok(Self::Optional(Box::new(Self::parse(of)?)));
        }
        if let Some(of) = inner("List<") {
            return Ok(Self::List(Box::new(Self::parse(of)?)));
        }
        if let Some(pair) = inner("Map<") {
            let (key, of) = split_top(pair).ok_or_else(|| format!("no key type in `{text}`"))?;
            let key = Primitive::named(key.trim())
                .ok_or_else(|| format!("a map key is a primitive, not `{key}` in `{text}`"))?;
            return Ok(Self::Map(key, Box::new(Self::parse(of)?)));
        }
        if text.contains('<') || text.is_empty() {
            return Err(format!("no type `{text}`"));
        }
        Ok(match text {
            "String" | "Boolean" | "Integer" | "Decimal" | "Binary64" | "Timestamp"
            | "Duration" | "Uuid" | "Bytes" | "Json" => {
                Self::Primitive(Primitive::named(text).expect("a primitive name"))
            }
            named => Self::Named(named.to_owned()),
        })
    }

    fn optional(&self) -> bool {
        matches!(self, Self::Optional(_))
    }
}

/// `left, right` split at the first comma outside angle brackets.
fn split_top(pair: &str) -> Option<(&str, &str)> {
    let mut depth = 0_i32;
    for (at, character) in pair.char_indices() {
        match character {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => return Some((&pair[..at], &pair[at + 1..])),
            _ => {}
        }
    }
    None
}

/// One declared field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) type_ref: TypeRef,
    pub(crate) presence: Option<Presence>,
}

impl Field {
    fn read(value: &Value) -> Result<Self, String> {
        let mut fields = Fields::of(value, "a field")?;
        let name = fields.text("name")?;
        let type_ref = TypeRef::parse(&fields.text("type")?)?;
        let presence = fields
            .optional("presence")
            .map(|value| presence(&value))
            .transpose()?;
        fields.done()?;
        Ok(Self {
            name,
            type_ref,
            presence,
        })
    }

    /// Whether `actual`, what a response carries for this field, keeps its declared presence.
    pub(crate) fn present(&self, actual: Option<&Value>) -> Result<(), String> {
        match (self.presence, actual) {
            (Some(Presence::NullWhenAbsent), None) => Err(format!(
                "response field {} was left out, and it is declared null_when_absent",
                self.name
            )),
            (Some(Presence::OmittedWhenAbsent), Some(Value::Null)) => Err(format!(
                "response field {} was sent as null, and it is declared omitted_when_absent",
                self.name
            )),
            _ => Ok(()),
        }
    }
}

/// The declared fields of a response or row.
pub(crate) fn read_fields(value: Value) -> Result<Vec<Field>, String> {
    match value {
        Value::Array(items) => items.iter().map(Field::read).collect(),
        other => Err(format!(
            "fields are a list of fields, not {}",
            other.describes()
        )),
    }
}

/// One named type a value is validated against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Declaration {
    Struct(Vec<Field>),
    Newtype(TypeRef),
    Enum(Vec<String>),
}

/// Every named type a response or row refers to, by qualified name.
pub(crate) type Declarations = BTreeMap<String, Declaration>;

/// The declarations from their suite form, an object of qualified names.
pub(crate) fn read_declarations(value: Value) -> Result<Declarations, String> {
    let Value::Object(members) = value else {
        return Err(format!(
            "declarations are an object of type names, not {}",
            value.describes()
        ));
    };
    let mut declarations = BTreeMap::new();
    for (name, body) in members {
        let mut fields = Fields::of(&body, "a declaration")?;
        let declaration = match fields.text("kind")?.as_str() {
            "struct" => Declaration::Struct(read_fields(fields.required("fields")?)?),
            "newtype" => Declaration::Newtype(TypeRef::parse(&fields.text("of")?)?),
            "enum" => Declaration::Enum(fields.texts("variants")?),
            other => {
                return Err(format!(
                    "a declaration of kind `{other}` is not interpreted by a2a-conformance"
                ));
            }
        };
        fields.done()?;
        declarations.insert(name, declaration);
    }
    Ok(declarations)
}

/// How strictly a value is validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Profile {
    /// A view row or an event-mapped response: closed structs, short texts and collections.
    Response,
    /// A command's direct response: closed structs, every nested field's presence held.
    DirectResponse,
}

impl Profile {
    fn text_limit(self) -> usize {
        match self {
            Self::Response => 4096,
            Self::DirectResponse => 1_048_576,
        }
    }

    fn collection_limit(self) -> usize {
        match self {
            Self::Response => 64,
            Self::DirectResponse => 65_536,
        }
    }
}

const BYTE_LIMIT: usize = 1_048_576;
const DEPTH_LIMIT: usize = 128;

/// Holds `value` (`None`: absent) to `type_ref`.
///
/// # Errors
///
/// The ESS code of the first violation (`invalid_input`, `invalid_presence`, `resource`) with
/// where it is.
pub(crate) fn validate(
    type_ref: &TypeRef,
    value: Option<&Value>,
    declarations: &Declarations,
    profile: Profile,
) -> Result<(), String> {
    let mut bytes = 0;
    Validation {
        declarations,
        profile,
        bytes: &mut bytes,
    }
    .value(type_ref, value, 0)
    .map_err(|(code, path)| {
        if path.is_empty() {
            code.to_owned()
        } else {
            format!("{code} at `{path}`")
        }
    })
}

struct Validation<'a> {
    declarations: &'a Declarations,
    profile: Profile,
    bytes: &'a mut usize,
}

type Invalid = (&'static str, String);

fn within(segment: &str, (code, path): Invalid) -> Invalid {
    if path.is_empty() {
        (code, segment.to_owned())
    } else if path.starts_with('[') {
        (code, format!("{segment}{path}"))
    } else {
        (code, format!("{segment}.{path}"))
    }
}

fn invalid(code: &'static str) -> Invalid {
    (code, String::new())
}

impl Validation<'_> {
    fn value(
        &mut self,
        type_ref: &TypeRef,
        value: Option<&Value>,
        depth: usize,
    ) -> Result<(), Invalid> {
        if depth > DEPTH_LIMIT || *self.bytes > BYTE_LIMIT {
            return Err(invalid("resource"));
        }
        if let TypeRef::Optional(of) = type_ref {
            return match value {
                None | Some(Value::Null) => Ok(()),
                Some(_) => self.value(of, value, depth + 1),
            };
        }
        let value = value.ok_or_else(|| invalid("invalid_input"))?;
        match type_ref {
            TypeRef::Named(name) => {
                let declaration = self
                    .declarations
                    .get(name)
                    .ok_or_else(|| invalid("invalid_input"))?;
                match declaration {
                    Declaration::Newtype(of) => self.value(of, Some(value), depth + 1),
                    Declaration::Enum(variants) => match value {
                        Value::Text(text) if variants.contains(text) => {
                            *self.bytes = self.bytes.saturating_add(text.len());
                            Ok(())
                        }
                        _ => Err(invalid("invalid_input")),
                    },
                    Declaration::Struct(fields) => self.fields(fields, value, depth),
                }
            }
            TypeRef::Primitive(kind) => self.primitive(*kind, value, depth),
            TypeRef::List(of) => {
                let Value::Array(items) = value else {
                    return Err(invalid("invalid_input"));
                };
                if items.len() > self.profile.collection_limit() {
                    return Err(invalid("resource"));
                }
                for (index, item) in items.iter().enumerate() {
                    self.value(of, Some(item), depth + 1)
                        .map_err(|error| within(&format!("[{index}]"), error))?;
                }
                Ok(())
            }
            TypeRef::Map(Primitive::String, of) => {
                let Value::Object(members) = value else {
                    return Err(invalid("invalid_input"));
                };
                if members.len() > self.profile.collection_limit() {
                    return Err(invalid("resource"));
                }
                for (name, child) in members {
                    *self.bytes = self.bytes.saturating_add(name.len());
                    self.value(of, Some(child), depth + 1)
                        .map_err(|error| within(&format!("[{name:?}]"), error))?;
                }
                Ok(())
            }
            TypeRef::Map(..) => Err(invalid("unsupported")),
            TypeRef::Optional(_) => unreachable!("handled above"),
        }
    }

    fn fields(&mut self, fields: &[Field], value: &Value, depth: usize) -> Result<(), Invalid> {
        let Value::Object(members) = value else {
            return Err(invalid("invalid_input"));
        };
        if let Some((unknown, _)) = members
            .iter()
            .find(|(name, _)| !fields.iter().any(|field| &field.name == name))
        {
            return Err(within(unknown, invalid("invalid_input")));
        }
        for field in fields {
            *self.bytes = self.bytes.saturating_add(field.name.len());
            let carried = value.member(&field.name);
            if self.profile == Profile::DirectResponse && field.present(carried).is_err() {
                return Err(within(&field.name, invalid("invalid_presence")));
            }
            self.value(&field.type_ref, carried, depth + 1)
                .map_err(|error| within(&field.name, error))?;
        }
        Ok(())
    }

    fn primitive(&mut self, kind: Primitive, value: &Value, depth: usize) -> Result<(), Invalid> {
        if !kind.admits(value) {
            return Err(invalid("invalid_input"));
        }
        if kind == Primitive::Json && self.profile == Profile::DirectResponse {
            return self.json(value, depth);
        }
        if let Value::Text(text) = value {
            *self.bytes = self.bytes.saturating_add(text.len());
            if text.len() > self.profile.text_limit() {
                return Err(invalid("resource"));
            }
        }
        Ok(())
    }

    fn json(&mut self, value: &Value, depth: usize) -> Result<(), Invalid> {
        *self.bytes = self.bytes.saturating_add(1);
        if depth > DEPTH_LIMIT || *self.bytes > BYTE_LIMIT {
            return Err(invalid("resource"));
        }
        let limit = Profile::DirectResponse.collection_limit();
        match value {
            Value::Text(text) => *self.bytes = self.bytes.saturating_add(text.len()),
            Value::Array(items) => {
                if items.len() > limit {
                    return Err(invalid("resource"));
                }
                for item in items {
                    self.json(item, depth + 1)?;
                }
            }
            Value::Object(members) => {
                if members.len() > limit {
                    return Err(invalid("resource"));
                }
                for (name, child) in members {
                    *self.bytes = self.bytes.saturating_add(name.len());
                    self.json(child, depth + 1)?;
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
        if *self.bytes > BYTE_LIMIT {
            return Err(invalid("resource"));
        }
        Ok(())
    }
}

/// The shape of a complete view row: its fields, their declarations and the identity field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SubjectShape {
    pub(crate) fields: Vec<Field>,
    pub(crate) declarations: Declarations,
}

impl SubjectShape {
    pub(crate) fn read(value: &Value) -> Result<Self, String> {
        let mut fields = Fields::of(value, "a complete subject shape")?;
        let declared = read_fields(fields.required("fields")?)?;
        let declarations = read_declarations(fields.required("declarations")?)?;
        let identity = fields.text("identity_field")?;
        fields.done()?;
        let field = declared
            .iter()
            .find(|field| field.name == identity)
            .ok_or("complete subject identity is not a declared field")?;
        if field.type_ref.optional() {
            return Err("complete subject identity cannot be optional".to_owned());
        }
        Ok(Self {
            fields: declared,
            declarations,
        })
    }

    /// Holds a row to every declared field (`SubjectShape::admit_row`).
    pub(crate) fn admit_row(&self, row: &Value) -> Result<(), String> {
        for field in &self.fields {
            validate(
                &field.type_ref,
                row.member(&field.name),
                &self.declarations,
                Profile::Response,
            )
            .map_err(|code| format!("row field {}: {code}", field.name))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_nested_type_references() {
        assert_eq!(
            TypeRef::parse("Optional<Map<String, a2a.discovery.StringList>>"),
            Ok(TypeRef::Optional(Box::new(TypeRef::Map(
                Primitive::String,
                Box::new(TypeRef::Named("a2a.discovery.StringList".to_owned()))
            ))))
        );
        assert!(TypeRef::parse("Optional<String").is_err());
    }

    #[test]
    fn holds_primitives_to_the_ess_grammar() {
        assert!(Primitive::Integer.admits(&Value::Number("2".to_owned())));
        assert!(!Primitive::Integer.admits(&Value::Number("2.5".to_owned())));
        assert!(Primitive::Bytes.admits(&Value::Text("aGk=".to_owned())));
        assert!(!Primitive::Bytes.admits(&Value::Text("aGk".to_owned())));
        assert!(Primitive::Uuid.admits(&Value::Text(
            "123e4567-e89b-12d3-a456-426614174000".to_owned()
        )));
    }
}
