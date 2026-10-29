//! Security review of story:view-tenant-scope (wave 5): the `a2a.tasks.TaskPushNotificationConfigs`
//! view and the four push notification configuration commands, run through `InProcess` over the
//! `ServerPorts` the binary serves, answer a configuration (its url, token and authentication) only
//! under the exact tenant it was created under, and one created without a tenant only without one
//! (A2A v1.0.1 section 13.1; `ess/domains/tasks.yaml`, the view's filter: both defined and equal,
//! or both absent).
//!
//! The tenants below are chosen at the boundaries of that rule: absent and the empty string
//! (defined, so not absent), a case variant, a trailing space, and the two spellings a form
//! decoder conflates (`a b`, `a+b`).

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
const ALL_CONFIGS: &str = "a2a.tasks.PushNotificationConfigs";
const ALL_TASKS: &str = "a2a.tasks.AllTasks";

/// Every tenant a task is made under and read under: each differs from every other.
const TENANTS: [Option<&str>; 8] = [
    None,
    Some(""),
    Some("acme"),
    Some("Acme"),
    Some("acme "),
    Some("a b"),
    Some("a+b"),
    Some("tasks"),
];

fn as_actor(actor: Actor) -> Caller {
    Caller { actor }
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

fn call(server: &mut InProcess, caller: Option<&Caller>, name: &str, input: &str) -> Called {
    server.call(
        caller,
        Some("1.0".to_owned()),
        Vec::new(),
        name,
        a2a::json::parse(input).expect("the test input is JSON"),
    )
}

fn client_call(server: &mut InProcess, name: &str, input: &str) -> Called {
    call(server, Some(&as_actor(Actor::TasksClient)), name, input)
}

fn tenant_member(tenant: Option<&str>) -> String {
    tenant.map_or_else(String::new, |tenant| format!(",\"tenant\":\"{tenant}\""))
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

/// The token of the configuration of the task made under `tenant`.
fn token_of(tenant: Option<&str>) -> String {
    format!("token-of-{tenant:?}").replace(['"', ' ', '+'], "_")
}

/// A task under `tenant` with one configuration `c-1` whose token is [`token_of`] `tenant`.
fn configured(server: &mut InProcess, tenant: Option<&str>, message_id: &str) -> String {
    let sent = client_call(
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
    let created = client_call(
        server,
        "a2a.tasks.CreateTaskPushNotificationConfig",
        &format!(
            "{{\"taskId\":\"{task}\",\"id\":\"c-1\",\"url\":\"https://example.invalid/c-1\",\"token\":\"{}\"{}}}",
            token_of(tenant),
            tenant_member(tenant)
        ),
    );
    assert_eq!(outcome(&created), "created", "{:?}", created.answer);
    task
}

/// The rows the view answers for `task` read under `tenant`.
fn rows(server: &mut InProcess, tenant: Option<&str>, task: &str) -> Vec<Value> {
    let called = client_call(
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

fn text(value: &Value) -> String {
    format!("{value:?}")
}

#[test]
fn sec_w5_the_view_answers_a_configuration_only_under_its_exact_tenant() {
    let mut server = server();
    let tasks: Vec<(Option<&str>, String)> = TENANTS
        .iter()
        .enumerate()
        .map(|(index, made)| (*made, configured(&mut server, *made, &format!("m-{index}"))))
        .collect();
    let mut leaks = Vec::new();
    let mut missing = Vec::new();
    for (made, task) in &tasks {
        for read in TENANTS {
            let rows = rows(&mut server, read, task);
            let foreign: Vec<&Value> = rows
                .iter()
                .filter(|row| {
                    row.member("tenant") != made.map(|made| Value::Text(made.to_owned())).as_ref()
                })
                .collect();
            if !foreign.is_empty() {
                leaks.push(format!(
                    "view of the task made under {made:?} read under {read:?}: {foreign:?}"
                ));
            }
            if read == *made {
                let own = rows
                    .iter()
                    .any(|row| row.member("token") == Some(&Value::Text(token_of(*made))));
                if !own || rows.len() != 1 {
                    missing.push(format!(
                        "{made:?} reads {} rows of its own task: {rows:?}",
                        rows.len()
                    ));
                }
            } else if !rows.is_empty() {
                leaks.push(format!(
                    "story:view-tenant-scope: the configuration of the task made under {made:?} is a row \
                     of the view read under {read:?}: {}",
                    rows.iter().map(text).collect::<Vec<_>>().join(", ")
                ));
            }
        }
    }
    assert!(leaks.is_empty(), "{leaks:#?}");
    assert!(missing.is_empty(), "{missing:#?}");
}

#[test]
fn sec_w5_the_push_config_commands_answer_another_spelling_of_a_tenant_as_an_unknown_task() {
    let mut server = server();
    let tasks: Vec<(Option<&str>, String)> = TENANTS
        .iter()
        .enumerate()
        .map(|(index, made)| (*made, configured(&mut server, *made, &format!("m-{index}"))))
        .collect();
    let mut crossed = Vec::new();
    for (made, task) in &tasks {
        for other in TENANTS.into_iter().filter(|other| other != made) {
            let input = format!(
                "{{\"taskId\":\"{task}\",\"id\":\"c-1\",\"url\":\"https://example.invalid/other\",\"token\":\"other-token\"{}}}",
                tenant_member(other)
            );
            for (name, expected) in [
                (
                    "a2a.tasks.CreateTaskPushNotificationConfig",
                    "task-not-found",
                ),
                ("a2a.tasks.GetTaskPushNotificationConfig", "not-found"),
                ("a2a.tasks.ListTaskPushNotificationConfigs", "other-tenant"),
                (
                    "a2a.tasks.DeleteTaskPushNotificationConfig",
                    "task-not-found",
                ),
            ] {
                let called = client_call(&mut server, name, &input);
                let answered = outcome(&called);
                let shown = text(called.answer.as_ref().expect("an answer"));
                if answered != expected || shown.contains(&token_of(*made)) {
                    crossed.push(format!(
                        "{name} under {other:?} on the task made under {made:?}: {answered}, {shown}"
                    ));
                }
            }
        }
    }
    for (made, task) in &tasks {
        let own = rows(&mut server, *made, task);
        if own.len() != 1 || own[0].member("token") != Some(&Value::Text(token_of(*made))) {
            crossed.push(format!(
                "the configuration of the task made under {made:?} was changed from another tenant: {own:?}"
            ));
        }
    }
    assert!(
        crossed.is_empty(),
        "a push notification configuration command crossed a tenant: {crossed:#?}"
    );
}

#[test]
fn sec_w5_only_the_system_actor_reads_the_views_that_span_tenants() {
    let mut server = server();
    let task = configured(&mut server, Some("acme"), "m-acme");
    let callers = [
        None,
        Some(as_actor(Actor::TasksClient)),
        Some(as_actor(Actor::DiscoveryClient)),
    ];
    let mut read = Vec::new();
    for caller in &callers {
        for view in [ALL_CONFIGS, ALL_TASKS] {
            let called = call(&mut server, caller.as_ref(), view, "{}");
            if called.answer.is_ok() {
                read.push(format!("{caller:?} read {view}: {:?}", called.answer));
            }
        }
    }
    for caller in [
        None,
        Some(as_actor(Actor::DiscoveryClient)),
        Some(as_actor(Actor::TasksAgent)),
    ] {
        for tenant in [None, Some("acme")] {
            let called = call(
                &mut server,
                caller.as_ref(),
                VIEW,
                &format!("{{\"task_id\":\"{task}\"{}}}", tenant_member(tenant)),
            );
            if called.answer.is_ok() {
                read.push(format!(
                    "{caller:?} read {VIEW} under {tenant:?}: {:?}",
                    called.answer
                ));
            }
        }
    }
    assert!(
        read.is_empty(),
        "a caller other than the one the grant names read a view of push notification \
         configurations: {read:#?}"
    );
}
