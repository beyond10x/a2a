//! Security review: a refused base URL's error text, which callers log, carries neither the URL's
//! credentials nor its raw control bytes; and only `http` is spoken, whatever the case or form
//! of the scheme.

use a2a_client::{Client, Error};

#[test]
fn a_refused_base_url_s_error_text_holds_no_credentials() {
    for url in [
        "http://alice:hunter2-secret@127.0.0.1:8080",
        "http://alice:hunter2-secret@127.0.0.1:8080/a2a?x=1",
    ] {
        let error = Client::new(url).expect_err("user information is refused");
        let text = format!("{error} {error:?}");
        assert!(
            !text.contains("hunter2-secret"),
            "the refusal of a URL with user information repeats its password: {text}"
        );
    }
}

#[test]
fn a_refused_base_url_s_error_text_holds_no_raw_control_bytes() {
    let error = Client::new("http://agent.example/\r\nX-Forged-Log-Line: 1")
        .expect_err("a control byte is refused");
    let text = error.to_string();
    assert!(
        !text.bytes().any(|byte| byte.is_ascii_control()),
        "the refusal writes the URL's raw control bytes into the error text: {text:?}"
    );
}

#[test]
fn only_http_is_spoken_whatever_the_scheme_s_form() {
    for url in [
        "https://127.0.0.1:1",
        "hTTpS://127.0.0.1:1",
        "http+unix://127.0.0.1:1",
        "ws://127.0.0.1:1",
        " http://127.0.0.1:1",
        "http\t://127.0.0.1:1",
    ] {
        match Client::new(url) {
            Err(Error::UnsupportedScheme(_) | Error::InvalidRequest(_)) => {}
            other => panic!("{url:?} was taken: {other:?}"),
        }
    }
    for url in ["http://127.0.0.1:1", "HTTP://127.0.0.1:1"] {
        assert!(Client::new(url).is_ok(), "{url}");
    }
}
