//! Adversarial cases, pass 2, against the cursor paging that fix pass 1 added to `a2a-server`
//! (story:server, wave 3). router.rs:950-952 says "a page token is one this server issued", and
//! `precheck` answers 400 "not a token this server issued" otherwise.

mod common;

use a2a::json::Value;
use common::{at, echo, text};

fn config_ids(value: &Value) -> Vec<String> {
    match at(value, &["configs"]) {
        Some(Value::Array(configs)) => configs.iter().map(|config| text(config, &["id"])).collect(),
        _ => Vec::new(),
    }
}

/// paging.rs encodes both lists' cursors the same way (hex of the key), and router.rs:968-972
/// refuses only text that does not decode. A ListTasks cursor decodes, so ListConfigs takes it
/// as the id to start after.
#[test]
fn adv2_a_list_tasks_page_token_is_not_taken_as_a_list_configs_page_token() {
    let server = echo();
    let task = server.completed_task();
    server.completed_task();
    for n in 1..=6 {
        let reply = server.call(
            "POST",
            &format!("/tasks/{task}/pushNotificationConfigs"),
            Some(&format!(
                "{{\"id\":\"{n}-cfg\",\"url\":\"https://example.invalid/{n}\"}}"
            )),
        );
        assert_eq!(reply.status, 200, "{}", reply.body);
    }
    let all = config_ids(
        &server
            .call(
                "GET",
                &format!("/tasks/{task}/pushNotificationConfigs"),
                None,
            )
            .json(),
    );
    assert_eq!(all.len(), 6, "{all:?}");
    let token = text(
        &server.call("GET", "/tasks?pageSize=1", None).json(),
        &["nextPageToken"],
    );
    assert!(!token.is_empty(), "ListTasks issued no page token");

    let reply = server.call(
        "GET",
        &format!("/tasks/{task}/pushNotificationConfigs?pageToken={token}"),
        None,
    );
    let listed = if reply.status == 200 {
        config_ids(&reply.json())
    } else {
        Vec::new()
    };
    assert!(
        reply.status == 400 || listed == all,
        "ListConfigs took ListTasks' page token and answered {} with {listed:?} of {all:?}",
        reply.status
    );
}

/// router.rs:925-934 `cursor` takes a token whose key has an empty timestamp, which no task's
/// cursor has: `0a` is the hex of a lone newline. ListTasks refuses `00` and `zz` as tokens it
/// never issued.
#[test]
fn adv2_a_list_tasks_page_token_the_server_never_issued_is_refused() {
    let server = echo();
    server.completed_task();
    server.completed_task();
    let refused = server.call("GET", "/tasks?pageToken=00", None);
    assert_eq!(refused.status, 400, "{}", refused.body);

    let reply = server.call("GET", "/tasks?pageToken=0a", None);
    assert_eq!(
        reply.status, 400,
        "ListTasks took the page token `0a` (a lone newline), which it never issues: {}",
        reply.body
    );
}

/// paging.rs `key` decodes each pair with `u8::from_str_radix`, which takes a leading `+`; the
/// server only ever writes lowercase hex digits (paging.rs `token`).
#[test]
fn adv2_a_list_configs_page_token_the_server_never_issued_is_refused() {
    let server = echo();
    let task = server.completed_task();
    let refused = server.call(
        "GET",
        &format!("/tasks/{task}/pushNotificationConfigs?pageToken=zz"),
        None,
    );
    assert_eq!(refused.status, 400, "{}", refused.body);

    let reply = server.call(
        "GET",
        &format!("/tasks/{task}/pushNotificationConfigs?pageToken=%2B0"),
        None,
    );
    assert_eq!(
        reply.status, 400,
        "ListConfigs took the page token `+0`, which the server never writes: {}",
        reply.body
    );
}
