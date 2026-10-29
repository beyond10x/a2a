//! story:view-tenant-scope: the `a2a.tasks.TaskPushNotificationConfigs` view answers a client only
//! the push notification configurations of the tenant it reads under (A2A v1.0.1 section 13.1:
//! "Implementations MUST scope results to the caller's authorized access boundaries"). A
//! configuration of another tenant, with the token the agent's webhook is called with, is never a
//! row of it: not through `InProcess`, and not over HTTP, where the binary serves the view on no
//! route at all.

mod common;

use a2a::actor::{Actor, Caller};
use a2a::json::Value;
use a2a::system::SystemEvent;
use a2a::tasks::{Message, TaskSnapshot};
use a2a_server::{Agent, Called, InProcess, Limits, Settings, Turn, default_card};

/// Takes every message into a task and asks for no step: the task stays submitted.
struct Holds;

impl Agent for Holds {
    fn turn(&mut self, _message: &Message, _task: Option<&TaskSnapshot>) -> Turn {
        Turn::Steps(Vec::new())
    }
}

const VIEW: &str = "a2a.tasks.TaskPushNotificationConfigs";

fn client() -> Caller {
    Caller {
        actor: Actor::TasksClient,
    }
}

fn server() -> InProcess {
    InProcess::new(
        Settings {
            card: default_card(),
            extended_card: None,
        },
        Box::new(Holds),
        Limits::default(),
    )
}

fn run(server: &mut InProcess, name: &str, input: &str) -> Called {
    server.call(
        Some(&client()),
        Some("1.0".to_owned()),
        Vec::new(),
        name,
        a2a::json::parse(input).expect("the test input is JSON"),
    )
}

fn tenant_member(tenant: Option<&str>) -> String {
    tenant.map_or_else(String::new, |tenant| format!(",\"tenant\":\"{tenant}\""))
}

/// A task under `tenant` with one configuration `c-1` whose token is `token`.
fn configured(
    server: &mut InProcess,
    tenant: Option<&str>,
    message_id: &str,
    token: &str,
) -> String {
    let sent = run(
        server,
        "a2a.tasks.SendMessage",
        &format!(
            "{{\"message\":{{\"messageId\":\"{message_id}\",\"role\":\"ROLE_USER\",\"parts\":[{{\"text\":\"hello\"}}]}}{}}}",
            tenant_member(tenant)
        ),
    );
    let task = sent
        .events
        .iter()
        .find_map(|event| match event {
            SystemEvent::TaskCreated(created) => Some(created.task_id.0.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no task under {tenant:?}: {:?}", sent.answer));
    let created = run(
        server,
        "a2a.tasks.CreateTaskPushNotificationConfig",
        &format!(
            "{{\"taskId\":\"{task}\",\"id\":\"c-1\",\"url\":\"https://example.invalid/c-1\",\"token\":\"{token}\"{}}}",
            tenant_member(tenant)
        ),
    );
    assert_eq!(
        created
            .answer
            .as_ref()
            .ok()
            .and_then(|answer| answer.member("outcome")),
        Some(&Value::Text("created".to_owned())),
        "{:?}",
        created.answer
    );
    task
}

/// The rows the view answers for `task` read under `tenant`.
fn rows(server: &mut InProcess, tenant: Option<&str>, task: &str) -> Vec<Value> {
    let called = run(
        server,
        VIEW,
        &format!("{{\"task_id\":\"{task}\"{}}}", tenant_member(tenant)),
    );
    match called
        .answer
        .as_ref()
        .ok()
        .and_then(|answer| answer.member("rows"))
    {
        Some(Value::Array(rows)) => rows.clone(),
        _ => panic!("the view answered no rows: {:?}", called.answer),
    }
}

#[test]
fn the_view_answers_a_client_of_another_tenant_or_none_no_row_of_a_tenants_task() {
    let mut server = server();
    let task = configured(&mut server, Some("acme"), "m-acme", "acme-webhook-token");
    for other in [None, Some("beta"), Some("Acme"), Some("acme ")] {
        let rows = rows(&mut server, other, &task);
        assert!(
            rows.is_empty(),
            "story:view-tenant-scope: a client under {other:?} read acme's configuration: {rows:?}"
        );
    }
    let own = rows(&mut server, Some("acme"), &task);
    assert_eq!(own.len(), 1, "acme reads its own configuration: {own:?}");
    assert_eq!(
        own[0].member("token"),
        Some(&Value::Text("acme-webhook-token".to_owned()))
    );
}

#[test]
fn the_view_answers_a_tenant_no_row_of_a_task_made_without_one() {
    let mut server = server();
    let task = configured(&mut server, None, "m-plain", "plain-webhook-token");
    for other in [Some("acme"), Some("")] {
        let rows = rows(&mut server, other, &task);
        assert!(
            rows.is_empty(),
            "story:view-tenant-scope: a client under {other:?} read the plain task's configuration: {rows:?}"
        );
    }
    let own = rows(&mut server, None, &task);
    assert_eq!(
        own.len(),
        1,
        "a plain client reads its own configuration: {own:?}"
    );
}

#[test]
fn over_http_no_route_answers_another_tenants_configuration() {
    let server = common::echo();
    let created = server.call(
        "POST",
        "/acme/message:send",
        Some(&common::message("", None, None)),
    );
    assert_eq!(created.status, 200, "{}", created.body);
    let task = common::text(&created.json(), &["task", "id"]);
    let config = server.call(
        "POST",
        &format!("/acme/tasks/{task}/pushNotificationConfigs"),
        Some(r#"{"id":"c-1","url":"https://example.invalid/acme","token":"acme-webhook-token"}"#),
    );
    assert_eq!(config.status, 200, "{}", config.body);
    let mut leaks = Vec::new();
    for prefix in ["", "/beta", "/Acme", "/acme%2F", "/views"] {
        for path in [
            format!("{prefix}/tasks/{task}/pushNotificationConfigs"),
            format!("{prefix}/tasks/{task}/pushNotificationConfigs/c-1"),
            format!("{prefix}/views/tasks/pushNotificationConfigs?task_id={task}"),
            format!("{prefix}/views/tasks/pushNotificationConfigs?task_id={task}&tenant=acme"),
            format!("{prefix}/views%2Ftasks%2FpushNotificationConfigs?task_id={task}"),
            format!("{prefix}/tasks/views/pushNotificationConfigs?task_id={task}"),
            format!("{prefix}/tasks/{task}/pushNotificationConfigs?tenant=acme"),
        ] {
            let reply = server.call("GET", &path, None);
            if reply.body.contains("acme-webhook-token") || reply.status == 200 {
                leaks.push(format!("GET {path}: {} {}", reply.status, reply.body));
            }
        }
    }
    assert!(
        leaks.is_empty(),
        "story:view-tenant-scope: acme's configuration answered outside acme: {leaks:#?}"
    );
}
