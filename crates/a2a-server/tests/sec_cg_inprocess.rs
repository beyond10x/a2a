//! Security review of story:conformance-gate: the public entry `InProcess` the unit added, run
//! over the `ServerPorts` the binary serves, held to the invariants of story:server (Decisions 5
//! and 7) and to its own documentation ("It decides every command as [`serve_with`] does").
//!
//! The first two cases are the safety facts the binary relies on, measured through the new
//! entry: tenant scoping of push notification configurations and the storage limits, also after
//! `reconfigure`. The third is the extended card's bearer token, which the entry checks as the
//! binary does. The last is what the entry, a trusted harness entry, answers and the binary does
//! not: the binary serves the push notification configuration views on no route.

mod common;

use std::io::{BufRead as _, BufReader};
use std::process::{Child, Command, Stdio};

use a2a::actor::{Actor, Caller};
use a2a::json::Value;
use a2a::system::SystemEvent;
use a2a::tasks::{Message, TaskSnapshot};
use a2a_server::{
    Access, Agent, BearerToken, Called, InProcess, Limits, Settings, Turn, default_card,
};

/// Takes every message into a task and asks for no step: the task stays submitted.
struct Holds;

impl Agent for Holds {
    fn turn(&mut self, _message: &Message, _task: Option<&TaskSnapshot>) -> Turn {
        Turn::Steps(Vec::new())
    }
}

fn client() -> Caller {
    Caller {
        actor: Actor::TasksClient,
    }
}

fn settings() -> Settings {
    Settings {
        card: default_card(),
        extended_card: None,
    }
}

fn server(limits: Limits) -> InProcess {
    InProcess::new(settings(), Box::new(Holds), limits)
}

fn parse(text: &str) -> Value {
    a2a::json::parse(text).expect("the test input is JSON")
}

fn run(server: &mut InProcess, caller: &Caller, name: &str, input: &str) -> Called {
    server.call(
        Some(caller),
        Some("1.0".to_owned()),
        Vec::new(),
        name,
        parse(input),
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

fn tenant_member(tenant: Option<&str>) -> String {
    tenant.map_or_else(String::new, |tenant| format!(",\"tenant\":\"{tenant}\""))
}

/// Sends a message under `tenant` and answers the outcome and the id of the task it created.
fn send(
    server: &mut InProcess,
    tenant: Option<&str>,
    message_id: &str,
) -> (String, Option<String>) {
    let input = format!(
        "{{\"message\":{{\"messageId\":\"{message_id}\",\"role\":\"ROLE_USER\",\"parts\":[{{\"text\":\"hello\"}}]}}{}}}",
        tenant_member(tenant)
    );
    let called = run(server, &client(), "a2a.tasks.SendMessage", &input);
    let task = called.events.iter().find_map(|event| match event {
        SystemEvent::TaskCreated(created) => Some(created.task_id.0.clone()),
        _ => None,
    });
    (outcome(&called), task)
}

fn task_under(server: &mut InProcess, tenant: Option<&str>, message_id: &str) -> String {
    let (outcome, task) = send(server, tenant, message_id);
    task.unwrap_or_else(|| panic!("no task created under {tenant:?}: {outcome}"))
}

fn create_config(
    server: &mut InProcess,
    tenant: Option<&str>,
    task: &str,
    id: &str,
    token: &str,
) -> String {
    let input = format!(
        "{{\"taskId\":\"{task}\",\"id\":\"{id}\",\"url\":\"https://example.invalid/{id}\",\"token\":\"{token}\"{}}}",
        tenant_member(tenant)
    );
    outcome(&run(
        server,
        &client(),
        "a2a.tasks.CreateTaskPushNotificationConfig",
        &input,
    ))
}

fn by_config(
    server: &mut InProcess,
    name: &str,
    tenant: Option<&str>,
    task: &str,
    id: &str,
) -> String {
    let input = format!(
        "{{\"taskId\":\"{task}\",\"id\":\"{id}\"{}}}",
        tenant_member(tenant)
    );
    outcome(&run(server, &client(), name, &input))
}

fn list_configs(server: &mut InProcess, tenant: Option<&str>, task: &str) -> Called {
    let input = format!("{{\"taskId\":\"{task}\"{}}}", tenant_member(tenant));
    run(
        server,
        &client(),
        "a2a.tasks.ListTaskPushNotificationConfigs",
        &input,
    )
}

fn get_task(server: &mut InProcess, tenant: Option<&str>, task: &str) -> String {
    let input = format!("{{\"id\":\"{task}\"{}}}", tenant_member(tenant));
    outcome(&run(server, &client(), "a2a.tasks.GetTask", &input))
}

#[test]
fn sec_cg_in_process_push_configs_answer_another_tenant_as_an_unknown_task() {
    let mut server = server(Limits::DEFAULT);
    let task = task_under(&mut server, Some("acme"), "m-1");
    assert_eq!(
        create_config(&mut server, Some("acme"), &task, "c-1", "acme-secret"),
        "created"
    );

    for other in [Some("globex"), None] {
        assert_eq!(
            create_config(&mut server, other, &task, "c-2", "other-secret"),
            "task-not-found",
            "CreateTaskPushNotificationConfig under {other:?} on acme's task"
        );
        assert_eq!(
            by_config(
                &mut server,
                "a2a.tasks.GetTaskPushNotificationConfig",
                other,
                &task,
                "c-1"
            ),
            "not-found",
            "GetTaskPushNotificationConfig under {other:?} on acme's task"
        );
        assert_eq!(
            outcome(&list_configs(&mut server, other, &task)),
            "other-tenant",
            "ListTaskPushNotificationConfigs under {other:?} on acme's task"
        );
        assert_eq!(
            by_config(
                &mut server,
                "a2a.tasks.DeleteTaskPushNotificationConfig",
                other,
                &task,
                "c-1"
            ),
            "task-not-found",
            "DeleteTaskPushNotificationConfig under {other:?} on acme's task"
        );
        assert_eq!(
            get_task(&mut server, other, &task),
            "other-tenant",
            "GetTask under {other:?} on acme's task"
        );
    }

    assert_eq!(
        by_config(
            &mut server,
            "a2a.tasks.GetTaskPushNotificationConfig",
            Some("acme"),
            &task,
            "c-1"
        ),
        "found",
        "acme's configuration is untouched by the other tenants' requests"
    );
    let listed = list_configs(&mut server, Some("acme"), &task);
    let configs = listed
        .answer
        .as_ref()
        .ok()
        .and_then(|answer| answer.member("response"))
        .and_then(|response| response.member("configs"));
    match configs {
        Some(Value::Array(configs)) => assert_eq!(configs.len(), 1, "{configs:?}"),
        other => panic!("acme lists no configs: {other:?}"),
    }
}

#[test]
fn sec_cg_in_process_keeps_the_storage_limits_per_tenant_and_task_across_reconfigure() {
    let limits = Limits {
        max_tasks: 1,
        max_push_configs: 1,
        ..Limits::DEFAULT
    };
    let mut server = server(limits);
    let acme = task_under(&mut server, Some("acme"), "m-1");
    let (refused, created) = send(&mut server, Some("acme"), "m-2");
    assert_eq!(
        refused, "task-limit",
        "acme's second task is over max_tasks"
    );
    assert_eq!(created, None, "a refused message creates no task");
    let globex = task_under(&mut server, Some("globex"), "m-3");
    assert_ne!(acme, globex);

    assert_eq!(
        create_config(&mut server, Some("acme"), &acme, "c-1", "t-1"),
        "created"
    );
    assert_eq!(
        create_config(&mut server, Some("acme"), &acme, "c-2", "t-2"),
        "config-limit",
        "a second configuration is over max_push_configs"
    );
    assert_eq!(
        create_config(&mut server, Some("acme"), &acme, "c-1", "t-3"),
        "created",
        "replacing a configuration of the same id is not refused"
    );

    server.reconfigure(settings(), Limits::DEFAULT);
    assert_eq!(get_task(&mut server, Some("acme"), &acme), "found");
    assert_eq!(
        get_task(&mut server, Some("globex"), &acme),
        "other-tenant",
        "reconfigure keeps the tenant of what is stored"
    );
    server.reconfigure(settings(), limits);
    let (refused, _) = send(&mut server, Some("acme"), "m-4");
    assert_eq!(
        refused, "task-limit",
        "the limit set back by reconfigure counts what was stored before"
    );
}

#[test]
fn sec_cg_in_process_serves_the_extended_card_to_nobody_without_a_token() {
    // `InProcess` "decides every command as `serve_with` does" (crates/a2a-server/src/lib.rs),
    // and `serve_with` serves the extended card only for the bearer token `Access` holds:
    // "Without one, the extended card is served to nobody" (story:server Decision 7). An
    // `InProcess` given no `Access` holds no token, and no caller may be served the card.
    let mut card = default_card();
    card.capabilities.extended_agent_card = Some(true);
    let mut extended = card.clone();
    extended.name = "extended card for authenticated callers".to_owned();
    let mut server = InProcess::new(
        Settings {
            card,
            extended_card: Some(extended),
        },
        Box::new(Holds),
        Limits::DEFAULT,
    );
    let caller = Caller {
        actor: Actor::DiscoveryClient,
    };
    let called = run(
        &mut server,
        &caller,
        "a2a.discovery.GetExtendedAgentCard",
        "{}",
    );
    assert_ne!(
        outcome(&called),
        "returned",
        "InProcess returned the extended card to a caller that presented no token: {:?}",
        called.answer
    );
}

#[test]
fn in_process_serves_the_extended_card_only_for_the_token_its_access_holds() {
    // With an `Access`, the entry checks the presented bearer token as the binary's router does:
    // no token or another token is answered `unauthenticated`, the token is served the extended
    // card, not the public one.
    let mut card = default_card();
    card.capabilities.extended_agent_card = Some(true);
    let mut extended = card.clone();
    extended.name = "extended card for authenticated callers".to_owned();
    let mut server = InProcess::new(
        Settings {
            card,
            extended_card: Some(extended),
        },
        Box::new(Holds),
        Limits::DEFAULT,
    )
    .with_access(Access {
        extended_card_token: BearerToken::new("card-token"),
    });
    let caller = Caller {
        actor: Actor::DiscoveryClient,
    };
    let mut present = |token: Option<&str>| {
        server.call_presenting(
            Some(&caller),
            token,
            Some("1.0".to_owned()),
            Vec::new(),
            "a2a.discovery.GetExtendedAgentCard",
            parse("{}"),
        )
    };
    for token in [None, Some("another-token")] {
        let called = present(token);
        assert_eq!(outcome(&called), "unauthenticated", "{token:?}");
        assert!(called.events.is_empty(), "{token:?}: {:?}", called.events);
    }
    let called = present(Some("card-token"));
    assert_eq!(outcome(&called), "returned", "{:?}", called.answer);
    let name = called
        .answer
        .as_ref()
        .ok()
        .and_then(|answer| answer.member("response"))
        .and_then(|response| response.member("card"))
        .and_then(|card| card.member("name"));
    assert_eq!(
        name,
        Some(&Value::Text(
            "extended card for authenticated callers".to_owned()
        )),
        "{:?}",
        called.answer
    );
}

/// The `a2a-server` binary on a free loopback port, killed when dropped.
struct Binary(Child);

impl Drop for Binary {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn binary() -> (Binary, common::Server) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_a2a-server"))
        .args(["--listen", "127.0.0.1:0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn a2a-server");
    let stdout = child.stdout.take().expect("stdout");
    let running = Binary(child);
    let mut line = String::new();
    BufReader::new(stdout)
        .read_line(&mut line)
        .expect("the listening line");
    let address = line
        .trim()
        .strip_prefix("a2a-server listening on http://")
        .unwrap_or_else(|| panic!("not the listening line: {line:?}"))
        .parse()
        .expect("a socket address");
    (running, common::Server { address })
}

#[test]
fn sec_cg_the_binary_answers_no_route_for_the_push_config_views() {
    // Tenant scoping of push notification configurations (story:server, A2A v1.0.1 section
    // 13.1): acme's configuration, and the token the agent's webhook is called with, are not
    // answered to a client of another tenant or none. `InProcess` is a trusted harness entry and
    // runs the `TaskPushNotificationConfigs` view, scoped by its `tenant` parameter
    // (story:view-tenant-scope, `tests/view_tenancy.rs`), for any caller its grant admits; the
    // binary serves that view, and every other view route the generated layer derives, on no
    // route, under no tenant and with no query.
    let (_binary, server) = binary();
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

    let views: Vec<(&str, &str)> = a2a::server::a2a_agent::ROUTES
        .iter()
        .copied()
        .filter(|(_, path)| path.contains("/views/"))
        .collect();
    assert!(
        views.contains(&("GET", "/views/tasks/pushNotificationConfigs")),
        "{views:?}"
    );
    for (method, view) in views {
        for path in [
            view.to_owned(),
            format!("/acme{view}"),
            format!("{view}?taskId={task}"),
            format!("{view}?task_id={task}"),
            format!("/acme{view}?taskId={task}"),
            format!("/acme{view}?task_id={task}"),
        ] {
            let reply = server.call(method, &path, None);
            assert!(
                reply.status == 404 && !reply.body.contains("acme-webhook-token"),
                "story:conformance-gate: the a2a-server binary answers {method} {path} with {}, \
                 not 404: {}",
                reply.status,
                reply.body
            );
        }
    }
}
