//! Adversary, story:client: credentials never appear in `Debug`, `Display` or an error.

mod common;

use a2a::tasks::{GetTask, TaskId};
use a2a_client::Client;
use common::{answer, answering, client_of};

const SECRET: &str = "s3cr3t-Pa55";

fn shown<T: std::fmt::Debug, E: std::fmt::Debug + std::fmt::Display>(
    result: &Result<T, E>,
) -> String {
    match result {
        Ok(value) => format!("{value:?}"),
        Err(error) => format!("{error} {error:?}"),
    }
}

/// A base URL with user information is refused, and the refusal quotes the whole URL, password
/// included (`transport.rs` `origin`: "`{url}` names no host, or user information"). An error
/// message is what gets logged.
#[test]
fn a_base_url_s_user_information_is_not_in_the_error() {
    for url in [
        format!("http://agent:{SECRET}@127.0.0.1:9"),
        format!("http://{SECRET}@127.0.0.1:9/a2a"),
    ] {
        let made = Client::new(&url);
        assert!(made.is_err(), "user information is refused");
        let text = shown(&made);
        assert!(
            !text.contains(SECRET),
            "the refusal shows the password: {}",
            text.replace(SECRET, "<SECRET>")
        );
    }
}

/// Guards `Client`'s hand-written `Debug`: the bearer token is not shown, by the client or a
/// clone of it.
#[test]
fn the_bearer_token_is_not_in_debug() {
    let client = Client::new("http://127.0.0.1:9")
        .expect("a base URL")
        .with_bearer(SECRET);
    for text in [format!("{client:?}"), format!("{:#?}", client.clone())] {
        assert!(!text.contains(SECRET), "Debug shows the token");
        assert!(text.contains("bearer"), "Debug names the field: {text}");
    }
}

/// Guards the header check: a token refused for a control character is not quoted in the
/// refusal, and nothing is sent.
#[test]
fn a_refused_bearer_token_is_not_in_the_error() {
    let (address, seen) = answering(answer(200, "application/a2a+json", "{}"));
    let answered = client_of(address)
        .with_bearer(&format!("{SECRET}\nX-Injected: 1"))
        .get_task(&GetTask {
            tenant: None,
            id: TaskId("t-1".to_owned()),
            history_length: None,
        });
    let text = shown(&answered);
    assert!(answered.is_err(), "a control character is refused");
    assert!(!text.contains(SECRET), "the refusal shows the token");
    assert!(seen.lock().expect("the list").is_empty());
}
