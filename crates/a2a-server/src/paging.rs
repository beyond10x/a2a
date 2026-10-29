//! Page tokens. A2A v1.0.1 section 3.1.4: "MUST use cursor-based pagination". A token names the
//! last item of its page, so the next page starts after that item wherever it now stands, and an
//! item that arrives between two calls does not push an item of the first page onto the second.
//!
//! A token is the letter of the list it pages through, then the key of its item in lowercase hex.
//! A token of one list names nothing in another, and a text this server would not write (another
//! letter, an uppercase or signed digit, an odd length) names nothing at all.

/// A list this server pages through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum List {
    /// `ListTasks`.
    Tasks,
    /// `ListTaskPushNotificationConfigs`.
    Configs,
}

impl List {
    fn letter(self) -> char {
        match self {
            Self::Tasks => 't',
            Self::Configs => 'c',
        }
    }
}

/// The token of `list` naming an item by `key`, opaque to the client.
pub(crate) fn token(list: List, key: &str) -> String {
    let mut token = String::with_capacity(1 + 2 * key.len());
    token.push(list.letter());
    for byte in key.bytes() {
        token.push_str(&format!("{byte:02x}"));
    }
    token
}

/// The key a token of `list` names; `None` where the text is no token this server issued for it.
pub(crate) fn key(list: List, token: &str) -> Option<String> {
    let (pairs, odd) = token
        .strip_prefix(list.letter())?
        .as_bytes()
        .as_chunks::<2>();
    if !odd.is_empty() {
        return None;
    }
    let bytes = pairs
        .iter()
        .map(|[high, low]| Some(digit(*high)? * 16 + digit(*low)?))
        .collect::<Option<Vec<u8>>>()?;
    String::from_utf8(bytes).ok()
}

/// The value of a lowercase hex digit, the only kind [`token`] writes.
fn digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{List, key, token};

    #[test]
    fn a_token_names_its_key() {
        for list in [List::Tasks, List::Configs] {
            for named in ["", "t-1", "2026-10-22T10:00:00.000Z\n9b1f", "ü/€"] {
                assert_eq!(key(list, &token(list, named)).as_deref(), Some(named));
            }
        }
    }

    #[test]
    fn a_token_of_one_list_names_nothing_in_another() {
        for named in ["", "t-1", "2026-10-22T10:00:00.000Z\n9b1f"] {
            assert_eq!(key(List::Configs, &token(List::Tasks, named)), None);
            assert_eq!(key(List::Tasks, &token(List::Configs, named)), None);
        }
    }

    #[test]
    fn a_text_this_server_did_not_issue_names_nothing() {
        for list in [List::Tasks, List::Configs] {
            for text in [
                "", "2", "abc", "zz", "ff", "00", "0a", "+0", "t+0", "c+0", "t-1", "c-1", "tAB",
                "cAB", "t0", "c0", "t 0a", "c 0a", "tzz", "czz", "t0x", "c0x", "T0a", "C0a",
            ] {
                assert_eq!(key(list, text), None, "{list:?} {text:?}");
            }
        }
    }
}
