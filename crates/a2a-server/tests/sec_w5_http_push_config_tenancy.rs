//! Security review of story:view-tenant-scope (wave 5): over HTTP, no route of the binary answers
//! a push notification configuration (its url or token) to a request under another tenant, or
//! under none, however the other tenant is spelled: a case variant, a space, a percent-encoded
//! byte that decodes to a different name, a `+` the query encoding could read as a space, a
//! `tenant` smuggled into the query or the body of a plain route under an encoded name. A
//! request under another tenant also changes no configuration of the tenant's.

mod common;

use common::{Server, echo, message, text};

/// A task under `prefix` (`""` or `/<tenant>`) with configuration `c-1` holding `token`.
fn configured(server: &Server, prefix: &str, token: &str) -> String {
    let created = server.call(
        "POST",
        &format!("{prefix}/message:send"),
        Some(&message("", None, None)),
    );
    assert_eq!(created.status, 200, "{prefix}: {}", created.body);
    let task = text(&created.json(), &["task", "id"]);
    let config = server.call(
        "POST",
        &format!("{prefix}/tasks/{task}/pushNotificationConfigs"),
        Some(&format!(
            "{{\"id\":\"c-1\",\"url\":\"https://example.invalid/{token}\",\"token\":\"{token}\"}}"
        )),
    );
    assert_eq!(config.status, 200, "{prefix}: {}", config.body);
    task
}

/// The token `c-1` of `task` holds, read under `prefix`.
fn token_under(server: &Server, prefix: &str, task: &str) -> String {
    let read = server.call(
        "GET",
        &format!("{prefix}/tasks/{task}/pushNotificationConfigs/c-1"),
        None,
    );
    assert_eq!(read.status, 200, "{prefix}: {}", read.body);
    text(&read.json(), &["token"])
}

#[test]
fn sec_w5_no_route_answers_a_configuration_under_another_spelling_of_its_tenant() {
    let server = echo();
    // The owners: `acme`, `a b` (spelt `/a%20b`), and the plain route.
    let owners = [
        ("/acme", "acme-token"),
        ("/a%20b", "a-space-b-token"),
        ("", "plain-token"),
    ];
    let tasks: Vec<(&str, &str, String)> = owners
        .iter()
        .map(|(prefix, token)| (*prefix, *token, configured(&server, prefix, token)))
        .collect();
    let others = [
        "",
        "/beta",
        "/Acme",
        "/ACME",
        "/acme%20",
        "/%20acme",
        "/acme%2F",
        "/acme%00",
        "/a+b",
        "/a%2Bb",
        "/a%20b%20",
        "/tasks",
        "/views",
    ];
    let queries = [
        "",
        "?tenant=acme",
        "?%74enant=acme",
        "?tenant=a+b",
        "?tenant=a%20b",
        "?tenant=acme&tenant=acme",
        "?tenant=",
    ];
    let mut leaks = Vec::new();
    for (owner, token, task) in &tasks {
        for other in others.into_iter().filter(|other| other != owner) {
            for query in queries {
                for path in [
                    format!("{other}/tasks/{task}/pushNotificationConfigs{query}"),
                    format!("{other}/tasks/{task}/pushNotificationConfigs/c-1{query}"),
                ] {
                    let reply = server.call("GET", &path, None);
                    if reply.status == 200 || reply.body.contains(token) {
                        leaks.push(format!(
                            "GET {path} (owned by {owner:?}): {} {}",
                            reply.status, reply.body
                        ));
                    }
                }
            }
        }
    }
    assert!(
        leaks.is_empty(),
        "a configuration answered outside the tenant it was created under: {leaks:#?}"
    );
    for (owner, token, task) in &tasks {
        assert_eq!(
            token_under(&server, owner, task),
            *token,
            "{owner:?} reads its own configuration"
        );
    }
}

#[test]
fn sec_w5_a_plain_route_takes_no_tenant_from_an_encoded_body_member() {
    let server = echo();
    let task = configured(&server, "/acme", "acme-token");
    // `tenant` with its first letter as a JSON escape: backslash, `u`, `0074`.
    let escaped = format!("\"{}u0074enant\"", '\\');
    for body in [
        format!("{{{escaped}:\"acme\",\"id\":\"c-1\",\"url\":\"https://example.invalid/x\",\"token\":\"x\"}}"),
        r#"{"tenant":"acme","tenant":"acme","id":"c-1","url":"https://example.invalid/x","token":"x"}"#.to_owned(),
    ] {
        let created = server.call(
            "POST",
            &format!("/tasks/{task}/pushNotificationConfigs"),
            Some(&body),
        );
        assert!(
            created.status >= 400,
            "a plain route created a configuration on acme's task from the body {body}: {} {}",
            created.status,
            created.body
        );
    }
    let deleted = server.call(
        "DELETE",
        &format!("/tasks/{task}/pushNotificationConfigs/c-1"),
        Some(&format!("{{{escaped}:\"acme\"}}")),
    );
    assert_eq!(
        deleted.status, 404,
        "a plain route deleted acme's configuration from an encoded body member: {}",
        deleted.body
    );
    for other in ["/beta", "/Acme", "/acme%20"] {
        let replaced = server.call(
            "POST",
            &format!("{other}/tasks/{task}/pushNotificationConfigs"),
            Some(r#"{"id":"c-1","url":"https://example.invalid/other","token":"other-token"}"#),
        );
        assert_eq!(replaced.status, 404, "{other}: {}", replaced.body);
        let deleted = server.call(
            "DELETE",
            &format!("{other}/tasks/{task}/pushNotificationConfigs/c-1"),
            None,
        );
        assert_eq!(deleted.status, 404, "{other}: {}", deleted.body);
    }
    assert_eq!(
        token_under(&server, "/acme", &task),
        "acme-token",
        "acme's configuration was changed by a request outside acme"
    );
}
