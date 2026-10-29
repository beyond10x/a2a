//! Adversary pass 1 on story:view-tenant-scope: edges of the `TaskPushNotificationConfigs` view's
//! tenant filter through `InProcess`, and a positive control for the HTTP leak probe of
//! `tests/view_tenancy.rs`.

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
        Some(&Caller {
            actor: Actor::TasksClient,
        }),
        Some("1.0".to_owned()),
        Vec::new(),
        name,
        a2a::json::parse(input).expect("the test input is JSON"),
    )
}

fn outcome(called: &Called) -> String {
    match called
        .answer
        .as_ref()
        .ok()
        .and_then(|answer| answer.member("outcome"))
    {
        Some(Value::Text(outcome)) => outcome.clone(),
        _ => panic!("no outcome: {:?}", called.answer),
    }
}

/// `,"tenant":"…"` for a tenant, nothing for none.
fn tenant_member(tenant: Option<&str>) -> String {
    tenant.map_or_else(String::new, |tenant| {
        format!(",\"tenant\":{}", common::quoted(tenant))
    })
}

/// A task under `tenant`.
fn task(server: &mut InProcess, tenant: Option<&str>, message_id: &str) -> String {
    let sent = run(
        server,
        "a2a.tasks.SendMessage",
        &format!(
            "{{\"message\":{{\"messageId\":\"{message_id}\",\"role\":\"ROLE_USER\",\"parts\":[{{\"text\":\"hello\"}}]}}{}}}",
            tenant_member(tenant)
        ),
    );
    sent.events
        .iter()
        .find_map(|event| match event {
            SystemEvent::TaskCreated(created) => Some(created.task_id.0.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no task under {tenant:?}: {:?}", sent.answer))
}

/// Creates configuration `id` of `task` under `tenant` with `token`; the outcome.
fn configure(
    server: &mut InProcess,
    tenant: Option<&str>,
    task: &str,
    id: &str,
    token: &str,
) -> String {
    let called = run(
        server,
        "a2a.tasks.CreateTaskPushNotificationConfig",
        &format!(
            "{{\"taskId\":\"{task}\",\"id\":\"{id}\",\"url\":\"https://example.invalid/{id}\",\"token\":\"{token}\"{}}}",
            tenant_member(tenant)
        ),
    );
    outcome(&called)
}

/// The rows the view answers for `task` read with the raw `tenant` member text, if any.
fn rows_raw(server: &mut InProcess, task: &str, tenant_json: Option<&str>) -> Vec<Value> {
    let tenant = tenant_json.map_or_else(String::new, |json| format!(",\"tenant\":{json}"));
    let called = run(server, VIEW, &format!("{{\"task_id\":\"{task}\"{tenant}}}"));
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

fn rows(server: &mut InProcess, task: &str, tenant: Option<&str>) -> Vec<Value> {
    rows_raw(server, task, tenant.map(common::quoted).as_deref())
}

fn tokens(rows: &[Value]) -> Vec<String> {
    rows.iter()
        .map(|row| match row.member("token") {
            Some(Value::Text(token)) => token.clone(),
            other => panic!("a row with no token: {other:?}"),
        })
        .collect()
}

#[test]
fn adv_w5_one_config_id_on_two_tenants_tasks_answers_each_tenant_only_its_own() {
    let mut server = server();
    let acme = task(&mut server, Some("acme"), "m-acme");
    let beta = task(&mut server, Some("beta"), "m-beta");
    assert_eq!(
        configure(&mut server, Some("acme"), &acme, "c-1", "acme-token"),
        "created"
    );
    assert_eq!(
        configure(&mut server, Some("beta"), &beta, "c-1", "beta-token"),
        "created"
    );
    assert_eq!(
        tokens(&rows(&mut server, &acme, Some("acme"))),
        ["acme-token"]
    );
    assert_eq!(
        tokens(&rows(&mut server, &beta, Some("beta"))),
        ["beta-token"]
    );
    assert!(rows(&mut server, &acme, Some("beta")).is_empty());
    assert!(rows(&mut server, &beta, Some("acme")).is_empty());
    assert!(rows(&mut server, &acme, None).is_empty());
    assert!(rows(&mut server, &beta, None).is_empty());
}

#[test]
fn adv_w5_a_deleted_config_is_no_row_under_its_own_tenant() {
    let mut server = server();
    let acme = task(&mut server, Some("acme"), "m-acme");
    assert_eq!(
        configure(&mut server, Some("acme"), &acme, "c-1", "acme-token"),
        "created"
    );
    assert_eq!(rows(&mut server, &acme, Some("acme")).len(), 1);
    let deleted = run(
        &mut server,
        "a2a.tasks.DeleteTaskPushNotificationConfig",
        &format!("{{\"taskId\":\"{acme}\",\"id\":\"c-1\",\"tenant\":\"acme\"}}"),
    );
    assert_eq!(outcome(&deleted), "deleted");
    assert!(rows(&mut server, &acme, Some("acme")).is_empty());
    assert!(rows(&mut server, &acme, None).is_empty());
}

#[test]
fn adv_w5_a_create_naming_another_tenant_than_its_tasks_stores_no_row_under_either() {
    let mut server = server();
    let acme = task(&mut server, Some("acme"), "m-acme");
    for (tenant, token) in [(Some("beta"), "beta-token"), (None, "plain-token")] {
        assert_eq!(
            configure(&mut server, tenant, &acme, "c-1", token),
            "task-not-found",
            "a create under {tenant:?} on acme's task"
        );
    }
    for tenant in [Some("acme"), Some("beta"), None] {
        let rows = rows(&mut server, &acme, tenant);
        assert!(
            rows.is_empty(),
            "a refused create left a row under {tenant:?}: {rows:?}"
        );
    }
}

#[test]
fn adv_w5_lookalike_tenants_read_no_row() {
    let mut server = server();
    let acme = task(&mut server, Some("acme"), "m-acme");
    assert_eq!(
        configure(&mut server, Some("acme"), &acme, "c-1", "acme-token"),
        "created"
    );
    for other in [
        "\u{430}cme",
        "\u{ff41}cme",
        "acme\u{200b}",
        "acme\u{0}",
        " acme",
        "ACME",
        "acme/",
    ] {
        let rows = rows(&mut server, &acme, Some(other));
        assert!(rows.is_empty(), "{other:?} read acme's row: {rows:?}");
    }
}

#[test]
fn adv_w5_an_empty_tenant_is_a_tenant_and_null_is_none() {
    let mut server = server();
    let plain = task(&mut server, None, "m-plain");
    assert_eq!(
        configure(&mut server, None, &plain, "c-1", "plain-token"),
        "created"
    );
    assert_eq!(
        tokens(&rows_raw(&mut server, &plain, Some("null"))),
        ["plain-token"],
        "a read with `tenant: null` is a read without a tenant"
    );
    let empty = task(&mut server, Some(""), "m-empty");
    assert_eq!(
        configure(&mut server, Some(""), &empty, "c-1", "empty-token"),
        "created"
    );
    assert!(rows(&mut server, &empty, None).is_empty());
    assert!(rows_raw(&mut server, &empty, Some("null")).is_empty());
    assert!(rows(&mut server, &plain, Some("")).is_empty());
    assert_eq!(
        tokens(&rows(&mut server, &empty, Some(""))),
        ["empty-token"]
    );
}

/// The leak probe of `view_tenancy.rs` reports a route that answers 200 or the token; this is
/// the route that does, so the probe is shown to see a leak where one is.
#[test]
fn adv_w5_the_http_probe_sees_the_configuration_under_its_own_tenant() {
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
    let own = server.call(
        "GET",
        &format!("/acme/tasks/{task}/pushNotificationConfigs"),
        None,
    );
    assert_eq!(own.status, 200, "{}", own.body);
    assert!(own.body.contains("acme-webhook-token"), "{}", own.body);
    let plain = server.call(
        "GET",
        &format!("/tasks/{task}/pushNotificationConfigs"),
        None,
    );
    assert_ne!(plain.status, 200, "{}", plain.body);
    assert!(!plain.body.contains("acme-webhook-token"), "{}", plain.body);
}
