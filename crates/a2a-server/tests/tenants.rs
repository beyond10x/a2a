//! Tenants scope tasks: a task answers only requests under the tenant it was created under, and a
//! task created on a plain route only requests on plain routes. Any other task answers as an
//! unknown id does (`ess/domains/tasks.yaml`, GetTask `other-tenant`; A2A v1.0.1 section 13.1).

mod common;

use a2a::json::Value;
use common::{Reply, Server, at, echo, message, text};

/// Creates a task waiting for input under `prefix` (`""` or `/acme`) and answers its id.
fn waiting_under(server: &Server, prefix: &str) -> String {
    let reply = server.call(
        "POST",
        &format!("{prefix}/message:send"),
        Some(&message("", None, None)),
    );
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert_eq!(
        text(&reply.json(), &["task", "status", "state"]),
        "TASK_STATE_INPUT_REQUIRED"
    );
    text(&reply.json(), &["task", "id"])
}

fn ids(reply: &Reply) -> Vec<String> {
    match at(&reply.json(), &["tasks"]) {
        Some(Value::Array(tasks)) => tasks.iter().map(|task| text(task, &["id"])).collect(),
        _ => panic!("no tasks: {}", reply.body),
    }
}

/// Every request a client could make about `task` under `prefix`, with what it takes.
fn about(prefix: &str, task: &str) -> Vec<(&'static str, String, Option<String>)> {
    vec![
        ("GET", format!("{prefix}/tasks/{task}"), None),
        ("POST", format!("{prefix}/tasks/{task}:cancel"), None),
        ("GET", format!("{prefix}/tasks/{task}:subscribe"), None),
        (
            "GET",
            format!("{prefix}/tasks/{task}/pushNotificationConfigs"),
            None,
        ),
        (
            "POST",
            format!("{prefix}/tasks/{task}/pushNotificationConfigs"),
            Some("{\"id\":\"c-2\",\"url\":\"https://example.invalid/other\"}".to_owned()),
        ),
        (
            "GET",
            format!("{prefix}/tasks/{task}/pushNotificationConfigs/c-1"),
            None,
        ),
        (
            "DELETE",
            format!("{prefix}/tasks/{task}/pushNotificationConfigs/c-1"),
            None,
        ),
        (
            "POST",
            format!("{prefix}/message:send"),
            Some(message("more", Some(task), None)),
        ),
        (
            "POST",
            format!("{prefix}/message:stream"),
            Some(message("more", Some(task), None)),
        ),
    ]
}

/// Asserts that every request about `task` under `prefix` answers exactly what the same request
/// about an id no task has answers.
fn answers_as_unknown(server: &Server, prefix: &str, task: &str) {
    let unknown = about(prefix, "no-such-task");
    for ((method, path, body), (_, unknown_path, unknown_body)) in
        about(prefix, task).into_iter().zip(unknown)
    {
        let reply = server.call(method, &path, body.as_deref());
        let expected = server.call(method, &unknown_path, unknown_body.as_deref());
        assert_eq!(
            expected.status, 404,
            "{method} {unknown_path}: {}",
            expected.body
        );
        assert_eq!(
            (reply.status, reply.body.as_str()),
            (expected.status, expected.body.as_str()),
            "{method} {path} answers otherwise than {method} {unknown_path}"
        );
    }
}

/// Asserts that the task and its configuration `c-1` are as they were made under `/acme`.
fn untouched(server: &Server, task: &str) {
    let read = server.call("GET", &format!("/acme/tasks/{task}"), None);
    assert_eq!(read.status, 200, "{}", read.body);
    assert_eq!(
        text(&read.json(), &["status", "state"]),
        "TASK_STATE_INPUT_REQUIRED",
        "a request under another tenant moved the task"
    );
    let config = server.call(
        "GET",
        &format!("/acme/tasks/{task}/pushNotificationConfigs/c-1"),
        None,
    );
    assert_eq!(
        config.status, 200,
        "a request under another tenant removed the configuration: {}",
        config.body
    );
    let configs = server.call(
        "GET",
        &format!("/acme/tasks/{task}/pushNotificationConfigs"),
        None,
    );
    assert_eq!(configs.status, 200, "{}", configs.body);
    match at(&configs.json(), &["configs"]) {
        Some(Value::Array(configs)) => assert_eq!(
            configs.len(),
            1,
            "a request under another tenant added a configuration: {configs:?}"
        ),
        _ => panic!("no configs: {}", configs.body),
    }
}

#[test]
fn a_task_answers_under_another_tenant_as_an_unknown_id() {
    let server = echo();
    let task = waiting_under(&server, "/acme");
    let created = server.call(
        "POST",
        &format!("/acme/tasks/{task}/pushNotificationConfigs"),
        Some("{\"id\":\"c-1\",\"url\":\"https://example.invalid/hook\"}"),
    );
    assert_eq!(created.status, 200, "{}", created.body);

    answers_as_unknown(&server, "/beta", &task);
    untouched(&server, &task);
    assert!(ids(&server.call("GET", "/acme/tasks", None)).contains(&task));
    let listed = ids(&server.call("GET", "/beta/tasks", None));
    assert!(
        !listed.contains(&task),
        "ListTasks under /beta lists a task of /acme: {listed:?}"
    );
}

#[test]
fn a_task_of_a_tenant_answers_on_a_plain_route_as_an_unknown_id() {
    let server = echo();
    let task = waiting_under(&server, "/acme");
    let created = server.call(
        "POST",
        &format!("/acme/tasks/{task}/pushNotificationConfigs"),
        Some("{\"id\":\"c-1\",\"url\":\"https://example.invalid/hook\"}"),
    );
    assert_eq!(created.status, 200, "{}", created.body);

    answers_as_unknown(&server, "", &task);
    untouched(&server, &task);
    let listed = ids(&server.call("GET", "/tasks", None));
    assert!(
        !listed.contains(&task),
        "ListTasks on the plain route lists a task of /acme: {listed:?}"
    );
}

#[test]
fn a_plain_route_takes_no_tenant_from_the_request() {
    let server = echo();
    let task = waiting_under(&server, "/acme");
    let created = server.call(
        "POST",
        &format!("/acme/tasks/{task}/pushNotificationConfigs"),
        Some("{\"id\":\"c-1\",\"url\":\"https://example.invalid/hook\"}"),
    );
    assert_eq!(created.status, 200, "{}", created.body);

    let read = server.call("GET", &format!("/tasks/{task}?tenant=acme"), None);
    assert_eq!(read.status, 404, "{}", read.body);
    assert_eq!(read.reason(), "TASK_NOT_FOUND");
    let canceled = server.call(
        "POST",
        &format!("/tasks/{task}:cancel"),
        Some("{\"tenant\":\"acme\"}"),
    );
    assert_eq!(canceled.status, 404, "{}", canceled.body);
    assert_eq!(canceled.reason(), "TASK_NOT_FOUND");
    let deleted = server.call(
        "DELETE",
        &format!("/tasks/{task}/pushNotificationConfigs/c-1"),
        Some("{\"tenant\":\"acme\"}"),
    );
    assert_eq!(deleted.status, 404, "{}", deleted.body);
    let listed = ids(&server.call("GET", "/tasks?tenant=acme", None));
    assert!(
        !listed.contains(&task),
        "ListTasks on the plain route took `tenant=acme` from the query: {listed:?}"
    );
    untouched(&server, &task);

    let sent = server.call(
        "POST",
        "/message:send",
        Some(&format!(
            "{{\"tenant\":\"acme\",{}",
            message("hello", None, None).trim_start_matches('{')
        )),
    );
    assert_eq!(sent.status, 200, "{}", sent.body);
    let plain = text(&sent.json(), &["task", "id"]);
    assert_eq!(
        server.call("GET", &format!("/tasks/{plain}"), None).status,
        200,
        "a task sent on a plain route is not the plain route's"
    );
    assert_eq!(
        server
            .call("GET", &format!("/acme/tasks/{plain}"), None)
            .status,
        404,
        "a task sent on a plain route took the body's `tenant`"
    );
}

#[test]
fn a_task_made_on_a_plain_route_answers_under_a_tenant_as_an_unknown_id() {
    let server = echo();
    let task = waiting_under(&server, "");
    let created = server.call(
        "POST",
        &format!("/tasks/{task}/pushNotificationConfigs"),
        Some("{\"id\":\"c-1\",\"url\":\"https://example.invalid/hook\"}"),
    );
    assert_eq!(created.status, 200, "{}", created.body);

    answers_as_unknown(&server, "/acme", &task);
    let read = server.call("GET", &format!("/tasks/{task}"), None);
    assert_eq!(read.status, 200, "{}", read.body);
    assert_eq!(
        text(&read.json(), &["status", "state"]),
        "TASK_STATE_INPUT_REQUIRED"
    );
    assert_eq!(
        server
            .call(
                "GET",
                &format!("/tasks/{task}/pushNotificationConfigs/c-1"),
                None
            )
            .status,
        200
    );
    assert!(ids(&server.call("GET", "/tasks", None)).contains(&task));
    let listed = ids(&server.call("GET", "/acme/tasks", None));
    assert!(
        !listed.contains(&task),
        "ListTasks under /acme lists a task made on a plain route: {listed:?}"
    );
}
