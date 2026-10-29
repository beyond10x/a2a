//! Adversarial case against the `/{tenant}/` twins of `a2a-server` (story:server, wave 3): a twin
//! answers for a task of its tenant what its plain route answers for a task without one, not
//! merely some status other than 405. Tenants scope tasks (fix pass 2), so a twin answers for a
//! task made on a plain route what it answers for an id no task has (`tests/tenants.rs`).

mod common;

use a2a::json::Value;
use common::{Reply, Server, echo, message, text};

/// Creates a task waiting for input under `prefix` with a configuration `c-1`; answers its id.
fn task_with_config(server: &Server, prefix: &str) -> String {
    let sent = server.call(
        "POST",
        &format!("{prefix}/message:send"),
        Some(&message("", None, None)),
    );
    assert_eq!(sent.status, 200, "{}", sent.body);
    let task = text(&sent.json(), &["task", "id"]);
    let created = server.call(
        "POST",
        &format!("{prefix}/tasks/{task}/pushNotificationConfigs"),
        Some("{\"id\":\"c-1\",\"url\":\"https://example.invalid/hook\"}"),
    );
    assert_eq!(created.status, 200, "{}", created.body);
    assert_eq!(text(&created.json(), &["id"]), "c-1");
    task
}

/// The status of an answer and the names of its body's members but `tenant`, sorted.
fn shape(reply: &Reply) -> (u16, Vec<String>) {
    let mut names: Vec<String> = match reply.json() {
        Value::Object(members) => members
            .into_iter()
            .map(|(name, _)| name)
            .filter(|name| name != "tenant")
            .collect(),
        _ => Vec::new(),
    };
    names.sort();
    (reply.status, names)
}

#[test]
fn adv_tenant_twins_answer_what_their_plain_routes_answer() {
    let server = echo();
    let plain = task_with_config(&server, "");
    let acme = task_with_config(&server, "/acme");
    for path in [
        "/tasks/{task}",
        "/tasks",
        "/tasks/{task}/pushNotificationConfigs",
        "/tasks/{task}/pushNotificationConfigs/c-1",
        "/extendedAgentCard",
    ] {
        let mine = server.call("GET", &path.replace("{task}", &plain), None);
        let twin = server.call(
            "GET",
            &format!("/acme{}", path.replace("{task}", &acme)),
            None,
        );
        assert_eq!(
            shape(&twin),
            shape(&mine),
            "GET /acme{path} answers otherwise than GET {path}: {} against {}",
            twin.body,
            mine.body
        );
        if !path.contains("{task}") && path != "/tasks" {
            assert_eq!(twin.body, mine.body, "GET /acme{path}");
        }
        if path.ends_with("/c-1") {
            assert_eq!(text(&twin.json(), &["tenant"]), "acme", "{}", twin.body);
        }
        let foreign = server.call(
            "GET",
            &format!("/acme{}", path.replace("{task}", &plain)),
            None,
        );
        let unknown = server.call(
            "GET",
            &format!("/acme{}", path.replace("{task}", "no-such-task")),
            None,
        );
        if path.contains("{task}") {
            assert_eq!(
                (foreign.status, foreign.body.as_str()),
                (unknown.status, unknown.body.as_str()),
                "GET /acme{path} for a task made on a plain route answers otherwise than for an \
                 unknown id"
            );
        }
    }
}
