//! Reads the generated layer's own answers back: the generated JSON reader with a deeper limit.
//!
//! Every request body is read by the generated `a2a::json::parse`, which refuses a document
//! nested 64 deep or more. What the server takes, it answers later inside the generated layer's
//! envelope (`{"response": {"task": {"history": [{"metadata": …}]}}}` and the `published`
//! events), a few levels deeper than the request carried it. Reading that envelope with the same
//! limit refuses an answer about a request the server admitted, so the envelope is read here with
//! a limit of 128: the intake's 64, and room for any envelope the generated layer writes. The
//! intake keeps its 64 and its 400; this reader is used on the generated layer's answers only.
// ESS gap (ess 0.67.0): the generated reader's depth (`json.rs:19`, `DEPTH`) is one constant for
// requests and for the envelope the layer reads back, and no construct sets either, so the
// envelope is read by this copy of the reader with a deeper limit.

use a2a::json::{ParseError, Value};

/// How deep an answer of the generated layer may nest: the intake's limit of 64, and 64 more
/// for the envelope, which adds fewer than ten levels.
const DEPTH: usize = 128;

/// Reads one JSON document the generated layer wrote, nested less than [`DEPTH`] deep.
///
/// # Errors
///
/// [`ParseError`] naming the offset and what was expected there, as the generated reader does.
pub(crate) fn parse(text: &str) -> Result<Value, ParseError> {
    let bytes = text.as_bytes();
    let mut at = 0;
    skip(bytes, &mut at);
    let value = read(bytes, &mut at, 0)?;
    skip(bytes, &mut at);
    if at != bytes.len() {
        return Err(refused(at, "the end of the document"));
    }
    Ok(value)
}

fn refused(at: usize, expected: &str) -> ParseError {
    ParseError {
        at,
        expected: expected.to_owned(),
    }
}

fn skip(bytes: &[u8], at: &mut usize) {
    while bytes
        .get(*at)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r'))
    {
        *at += 1;
    }
}

fn read(bytes: &[u8], at: &mut usize, depth: usize) -> Result<Value, ParseError> {
    if depth > DEPTH {
        return Err(refused(*at, "a document nested less than 128 deep"));
    }
    match bytes.get(*at) {
        Some(b'{') => object(bytes, at, depth),
        Some(b'[') => array(bytes, at, depth),
        Some(b'"') => text(bytes, at).map(Value::Text),
        Some(b't') => literal(bytes, at, b"true", Value::Bool(true)),
        Some(b'f') => literal(bytes, at, b"false", Value::Bool(false)),
        Some(b'n') => literal(bytes, at, b"null", Value::Null),
        Some(byte) if *byte == b'-' || byte.is_ascii_digit() => number(bytes, at),
        _ => Err(refused(*at, "a value")),
    }
}

fn literal(bytes: &[u8], at: &mut usize, word: &[u8], value: Value) -> Result<Value, ParseError> {
    if bytes.get(*at..*at + word.len()) != Some(word) {
        return Err(refused(
            *at,
            &format!("`{}`", String::from_utf8_lossy(word)),
        ));
    }
    *at += word.len();
    Ok(value)
}

fn digits(bytes: &[u8], at: &mut usize) -> usize {
    let start = *at;
    while bytes.get(*at).is_some_and(u8::is_ascii_digit) {
        *at += 1;
    }
    *at - start
}

fn number(bytes: &[u8], at: &mut usize) -> Result<Value, ParseError> {
    let start = *at;
    if bytes.get(*at) == Some(&b'-') {
        *at += 1;
    }
    if digits(bytes, at) == 0 {
        return Err(refused(*at, "a digit"));
    }
    if bytes.get(*at) == Some(&b'.') {
        *at += 1;
        if digits(bytes, at) == 0 {
            return Err(refused(*at, "a digit after the decimal point"));
        }
    }
    if matches!(bytes.get(*at), Some(b'e' | b'E')) {
        *at += 1;
        if matches!(bytes.get(*at), Some(b'+' | b'-')) {
            *at += 1;
        }
        if digits(bytes, at) == 0 {
            return Err(refused(*at, "a digit in the exponent"));
        }
    }
    Ok(Value::Number(
        String::from_utf8_lossy(&bytes[start..*at]).into_owned(),
    ))
}

fn text(bytes: &[u8], at: &mut usize) -> Result<String, ParseError> {
    *at += 1;
    let mut out = String::new();
    loop {
        match bytes.get(*at).copied() {
            None => return Err(refused(*at, "a closing quote")),
            Some(b'"') => {
                *at += 1;
                return Ok(out);
            }
            Some(b'\\') => {
                *at += 1;
                out.push(escape(bytes, at)?);
            }
            Some(_) => {
                let start = *at;
                while bytes
                    .get(*at)
                    .is_some_and(|byte| *byte != b'"' && *byte != b'\\')
                {
                    *at += 1;
                }
                out.push_str(&String::from_utf8_lossy(&bytes[start..*at]));
            }
        }
    }
}

fn escape(bytes: &[u8], at: &mut usize) -> Result<char, ParseError> {
    let Some(byte) = bytes.get(*at).copied() else {
        return Err(refused(*at, "an escape"));
    };
    *at += 1;
    Ok(match byte {
        b'"' => '"',
        b'\\' => '\\',
        b'/' => '/',
        b'b' => '\u{8}',
        b'f' => '\u{c}',
        b'n' => '\n',
        b'r' => '\r',
        b't' => '\t',
        b'u' => return unicode(bytes, at),
        _ => return Err(refused(*at, "one of \" \\ / b f n r t u")),
    })
}

fn unicode(bytes: &[u8], at: &mut usize) -> Result<char, ParseError> {
    let first = hex(bytes, at)?;
    if (0xD800..0xDC00).contains(&first) {
        if bytes.get(*at) != Some(&b'\\') || bytes.get(*at + 1) != Some(&b'u') {
            return Err(refused(*at, "the low half of a surrogate pair"));
        }
        *at += 2;
        let second = hex(bytes, at)?;
        if !(0xDC00..0xE000).contains(&second) {
            return Err(refused(*at, "the low half of a surrogate pair"));
        }
        let joined = 0x1_0000 + ((first - 0xD800) << 10) + (second - 0xDC00);
        return char::from_u32(joined).ok_or_else(|| refused(*at, "a character"));
    }
    char::from_u32(first).ok_or_else(|| refused(*at, "a character"))
}

fn hex(bytes: &[u8], at: &mut usize) -> Result<u32, ParseError> {
    let mut value = 0;
    for _ in 0..4 {
        let digit = bytes
            .get(*at)
            .and_then(|byte| char::from(*byte).to_digit(16))
            .ok_or_else(|| refused(*at, "four hexadecimal digits"))?;
        value = value * 16 + digit;
        *at += 1;
    }
    Ok(value)
}

fn array(bytes: &[u8], at: &mut usize, depth: usize) -> Result<Value, ParseError> {
    *at += 1;
    let mut items = Vec::new();
    skip(bytes, at);
    if bytes.get(*at) == Some(&b']') {
        *at += 1;
        return Ok(Value::Array(items));
    }
    loop {
        skip(bytes, at);
        items.push(read(bytes, at, depth + 1)?);
        skip(bytes, at);
        match bytes.get(*at) {
            Some(b',') => *at += 1,
            Some(b']') => {
                *at += 1;
                return Ok(Value::Array(items));
            }
            _ => return Err(refused(*at, "`,` or `]`")),
        }
    }
}

fn object(bytes: &[u8], at: &mut usize, depth: usize) -> Result<Value, ParseError> {
    *at += 1;
    let mut members = Vec::new();
    skip(bytes, at);
    if bytes.get(*at) == Some(&b'}') {
        *at += 1;
        return Ok(Value::Object(members));
    }
    loop {
        skip(bytes, at);
        if bytes.get(*at) != Some(&b'"') {
            return Err(refused(*at, "a member name"));
        }
        let name = text(bytes, at)?;
        skip(bytes, at);
        if bytes.get(*at) != Some(&b':') {
            return Err(refused(*at, "`:`"));
        }
        *at += 1;
        skip(bytes, at);
        members.push((name, read(bytes, at, depth + 1)?));
        skip(bytes, at);
        match bytes.get(*at) {
            Some(b',') => *at += 1,
            Some(b'}') => {
                *at += 1;
                return Ok(Value::Object(members));
            }
            _ => return Err(refused(*at, "`,` or `}`")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nested(depth: usize) -> String {
        format!("{}1{}", "[".repeat(depth), "]".repeat(depth))
    }

    #[test]
    fn reads_what_the_generated_reader_reads_as_it_does() {
        let samples = [
            "null",
            " true ",
            "false",
            "-0.5e+3",
            "12",
            "\"a\\u00e9\\ud83d\\ude00\\n\\\"\"",
            "[]",
            "{}",
            "{\"a\":[1,{\"b\":null}],\"a\":\"twice\"}",
            "[1,]",
            "{\"a\" 1}",
            "\"\\ud800\"",
            "\"\\x\"",
            "01",
            "1.",
            "1e",
            "tru",
            "[1 2]",
            "{\"a\":1,}",
            "\"open",
            "[] []",
            "",
        ];
        for sample in samples {
            assert_eq!(parse(sample), a2a::json::parse(sample), "{sample:?}");
        }
    }

    #[test]
    fn reads_an_envelope_nested_past_the_intake_limit() {
        let deep = nested(66);
        assert!(a2a::json::parse(&deep).is_err());
        assert!(parse(&deep).is_ok());
    }

    #[test]
    fn refuses_a_document_nested_past_its_own_limit() {
        assert!(parse(&nested(128)).is_ok());
        let refused = parse(&nested(129)).expect_err("nested 129 deep");
        assert_eq!(refused.expected, "a document nested less than 128 deep");
    }
}
