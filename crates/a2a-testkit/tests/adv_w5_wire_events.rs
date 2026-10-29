//! Adversary, story:testkit-wire-events: endpoints that answer something other than what the
//! scenario's step asked for, each a proxy in front of `a2a-server` with an agent that takes no
//! step. A scenario must not come out `passed` on a value the endpoint did not answer for it:
//!
//! - a message answered with a task an earlier scenario created, not a new one;
//! - a subscription that streams another task than the one it names;
//! - tasks answered without a `contextId`, whose rows then carry the request's context;
//! - the push notification configuration commands of gap A against an endpoint that ignores the
//!   tenant, which acts across tenants.
//!
//! The controls (a GetTask that answers another task, a ListTasks that hides the task) show the
//! proxies plant what they claim where the run already catches it.

mod adv_support;
mod common;

use std::collections::HashMap;
use std::io::{Read as _, Write as _};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use a2a::json::{self, Value};
use a2a::tasks::{ContextId, SendMessage};
use a2a_conformance::{Results, Status, Suite};
use a2a_testkit::WireTarget;
use adv_support::{read_request, suite};
use common::{Endpoint, echo, idle, message, root};

const SEND_TASK_CREATED: &str = "a2a.tasks.SendMessage/outcome/task-created";
const GET_TASK_FOUND: &str = "a2a.tasks.GetTask/outcome/found";
const SUBSCRIBED: &str = "a2a.tasks.SubscribeToTask/outcome/subscribed";

/// The tenant and context the synthesized task scenarios send their message under.
const TENANT: &str = "tenant";
const CONTEXT: &str = "message.context_id";

/// The suite with only the scenarios `ids`, in that order.
fn only(ids: &[&str]) -> String {
    let text = std::fs::read_to_string(suite("adv_w5_wire_events")).expect("the suite");
    let mut document = json::parse_written(&text).expect("the suite is JSON");
    let Value::Object(root) = &mut document else {
        panic!("the suite is an object");
    };
    let (_, scenarios) = root
        .iter_mut()
        .find(|(name, _)| name == "scenarios")
        .expect("the suite's scenarios");
    let Value::Object(members) = scenarios else {
        panic!("the scenarios are an object");
    };
    let kept: Vec<(String, Value)> = ids
        .iter()
        .map(|id| {
            members
                .iter()
                .find(|(name, _)| name == id)
                .unwrap_or_else(|| panic!("{id} is in the suite"))
                .clone()
        })
        .collect();
    *members = kept;
    let mut written = String::new();
    json::push_value(&mut written, &document);
    written
}

/// The run of the scenarios `ids` against `endpoint`, as `a2a-testkit run` runs them.
fn run_only(endpoint: &Endpoint, ids: &[&str]) -> Results {
    let text = only(ids);
    let parsed = Suite::parse(&text).expect("the reduced suite");
    let mut target = WireTarget::new(endpoint.client())
        .with_suite(&text)
        .expect("the suite's expectations");
    a2a_conformance::run(&parsed, &mut target)
}

/// `scenario`'s status and message in `results`.
fn outcome(results: &Results, scenario: &str) -> (Option<Status>, String) {
    let result = results
        .results
        .iter()
        .find(|result| result.scenario_id == scenario);
    (
        result.map(|result| result.status),
        result
            .and_then(|result| result.message.clone())
            .unwrap_or_default(),
    )
}

/// Asserts `scenario` did not come out `passed`.
fn assert_not_passed(results: &Results, scenario: &str, why: &str) {
    let (status, message) = outcome(results, scenario);
    assert!(status.is_some(), "{scenario} was run");
    assert_ne!(
        status,
        Some(Status::Passed),
        "{scenario} passed against an endpoint that {why} ({message})"
    );
}

/// Where a proxy sends a request instead: its new target, by method and target.
type Retarget = Arc<dyn Fn(&str, &str) -> Option<String> + Send + Sync>;
/// What a proxy answers instead of the upstream's whole answer, by method and target; `None`
/// passes the answer as it streams.
type Rewrite = Arc<dyn Fn(&str, &str, Vec<u8>) -> Option<Vec<u8>> + Send + Sync>;
/// Which answers a proxy reads whole and hands to its [`Rewrite`].
type Whole = fn(&str, &str) -> bool;

fn never(_method: &str, _target: &str) -> bool {
    false
}

/// A proxy in front of `upstream` that sends each request to the target `retarget` names, and
/// answers what `rewrite` makes of the upstream's answer to a request `whole` selects.
fn proxy(upstream: SocketAddr, retarget: Retarget, whole: Whole, rewrite: Rewrite) -> Endpoint {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = listener.local_addr().expect("bound address");
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let (retarget, rewrite) = (Arc::clone(&retarget), Arc::clone(&rewrite));
            std::thread::spawn(move || relay(stream, upstream, &retarget, whole, &rewrite));
        }
    });
    Endpoint { address }
}

fn relay(
    mut client: TcpStream,
    upstream: SocketAddr,
    retarget: &Retarget,
    whole: Whole,
    rewrite: &Rewrite,
) {
    let Some((head, body)) = read_request(&mut client) else {
        return;
    };
    let mut words = head.lines().next().unwrap_or_default().split(' ');
    let method = words.next().unwrap_or_default().to_owned();
    let target = words.next().unwrap_or_default().to_owned();
    let head = match retarget(&method, &target) {
        Some(new) => head.replacen(
            &format!("{method} {target} "),
            &format!("{method} {new} "),
            1,
        ),
        None => head,
    };
    let Ok(mut server) = TcpStream::connect(upstream) else {
        return;
    };
    if server.write_all(head.as_bytes()).is_err() || server.write_all(&body).is_err() {
        return;
    }
    if !whole(&method, &target) {
        let _ = std::io::copy(&mut server, &mut client);
        return;
    }
    let mut answered = Vec::new();
    let _ = server.read_to_end(&mut answered);
    let written = rewrite(&method, &target, answered.clone()).unwrap_or(answered);
    let _ = client.write_all(&written);
}

fn no_retarget() -> Retarget {
    Arc::new(|_method: &str, _target: &str| None)
}

fn no_rewrite() -> Rewrite {
    Arc::new(|_method: &str, _target: &str, _answered: Vec<u8>| None)
}

/// `answered` with its JSON body changed by `change`, where it is a `200` answer with a JSON
/// body, `Content-Length` counting the body written.
fn with_body(answered: &[u8], change: impl Fn(&mut Value)) -> Option<Vec<u8>> {
    let answered = std::str::from_utf8(answered).ok()?;
    if !answered.starts_with("HTTP/1.1 200") {
        return None;
    }
    let (head, body) = answered.split_once("\r\n\r\n")?;
    let mut value = json::parse(body).ok()?;
    change(&mut value);
    let mut written = String::new();
    json::push_value(&mut written, &value);
    let head: Vec<String> = head
        .split("\r\n")
        .map(|line| match line.split_once(':') {
            Some((name, _)) if name.eq_ignore_ascii_case("content-length") => {
                format!("{name}: {}", written.len())
            }
            _ => line.to_owned(),
        })
        .collect();
    Some(format!("{}\r\n\r\n{written}", head.join("\r\n")).into_bytes())
}

/// The path of `target`, without its query.
fn path(target: &str) -> &str {
    target.split('?').next().unwrap_or_default()
}

/// A task created on `endpoint` under [`TENANT`] and [`CONTEXT`] by a message no scenario sends:
/// the agent takes no step, so it stays `TASK_STATE_SUBMITTED`.
fn decoy(endpoint: &Endpoint) -> String {
    let mut sent = message("decoy");
    sent.context_id = Some(ContextId(CONTEXT.to_owned()));
    let answered = endpoint
        .client()
        .send_message(&SendMessage {
            tenant: Some(TENANT.to_owned()),
            message: sent,
            configuration: None,
            metadata: None,
        })
        .expect("the decoy message is taken");
    answered.task.expect("into a task").id.0
}

/// An endpoint that answers every message after the first under a route with the task the first
/// created: the endpoint created no task for it, yet the run reads `TaskCreated` from the answer,
/// as the target forgets at each scenario's start which tasks the endpoint already answered.
#[test]
fn a_message_answered_with_a_task_an_earlier_scenario_created_is_not_task_created() {
    let server = idle();
    let first: Arc<Mutex<HashMap<String, Vec<u8>>>> = Arc::default();
    let replayed = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&replayed);
    let rewrite: Rewrite = Arc::new(move |_method: &str, target: &str, answered: Vec<u8>| {
        let mut first = first.lock().expect("the first answers");
        match first.get(target) {
            Some(kept) => {
                counted.fetch_add(1, Ordering::SeqCst);
                Some(kept.clone())
            }
            None => {
                first.insert(target.to_owned(), answered);
                None
            }
        }
    });
    let sends = |method: &str, target: &str| method == "POST" && target.ends_with("/message:send");
    let proxy = proxy(server.address, no_retarget(), sends, rewrite);
    let results = run_only(&proxy, &[GET_TASK_FOUND, SEND_TASK_CREATED]);
    assert_eq!(
        replayed.load(Ordering::SeqCst),
        1,
        "precondition: the second message is answered with the first task"
    );
    assert_eq!(
        outcome(&results, GET_TASK_FOUND).0,
        Some(Status::Passed),
        "precondition: the first scenario's task is a new one: {:?}",
        outcome(&results, GET_TASK_FOUND)
    );
    assert_not_passed(
        &results,
        SEND_TASK_CREATED,
        "answered the message with the task an earlier scenario created",
    );
}

/// An endpoint whose SubscribeToTask streams another task than the one it names: the decoy, a
/// task of the same tenant and context in the same state.
#[test]
fn a_subscription_that_streams_another_task_is_not_subscribed() {
    let server = idle();
    let decoy = decoy(&server);
    let prefix = format!("/{TENANT}/tasks/");
    let redirected = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&redirected);
    let retarget: Retarget = Arc::new(move |method: &str, target: &str| {
        let task = target.strip_prefix(&prefix)?.strip_suffix(":subscribe")?;
        let redirect = method == "GET" && task != decoy;
        if redirect {
            counted.fetch_add(1, Ordering::SeqCst);
        }
        redirect.then(|| format!("{prefix}{decoy}:subscribe"))
    });
    let proxy = proxy(server.address, retarget, never, no_rewrite());
    let results = run_only(&proxy, &[SUBSCRIBED]);
    assert!(
        redirected.load(Ordering::SeqCst) >= 1,
        "precondition: the subscription was sent to the decoy"
    );
    assert_not_passed(
        &results,
        SUBSCRIBED,
        "streamed another task than the one SubscribeToTask named",
    );
}

/// Control: an endpoint whose GetTask answers another task than the one it names. The run's own
/// reads of `AllTasks` go through the same GetTask, so the scenario fails on their rows.
#[test]
fn a_get_task_that_answers_another_task_is_not_found() {
    let server = idle();
    let decoy = decoy(&server);
    let prefix = format!("/{TENANT}/tasks/");
    let retarget: Retarget = Arc::new(move |method: &str, target: &str| {
        let task = target.strip_prefix(&prefix)?;
        (method == "GET" && !task.contains([':', '/', '?']) && task != decoy)
            .then(|| format!("{prefix}{decoy}"))
    });
    let proxy = proxy(server.address, retarget, never, no_rewrite());
    let results = run_only(&proxy, &[GET_TASK_FOUND]);
    assert_not_passed(
        &results,
        GET_TASK_FOUND,
        "answered GetTask with another task than the one it named",
    );
}

/// Every member named `contextId`, at any depth, removed.
fn without_context(value: &mut Value) {
    match value {
        Value::Object(members) => {
            members.retain(|(name, _)| name != "contextId");
            for (_, member) in members {
                without_context(member);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(without_context),
        _ => {}
    }
}

/// An endpoint that answers every task without its `contextId`, which a2a.proto lets it omit:
/// `AllTasks` claims `context_id`, and the endpoint never answered one, so the claim cannot be
/// observed. The target fills the row with the context of the scenario's own request.
#[test]
fn a_task_answered_without_a_context_does_not_pass_a_claim_on_its_context() {
    let server = idle();
    let tasks = |method: &str, target: &str| {
        (method == "POST" && target.ends_with("/message:send"))
            || (method == "GET" && path(target).contains("/tasks"))
    };
    let stripped = Arc::new(AtomicUsize::new(0));
    let counted = Arc::clone(&stripped);
    let rewrite: Rewrite = Arc::new(move |_method: &str, _target: &str, answered: Vec<u8>| {
        if answered.windows(9).any(|window| window == b"contextId") {
            counted.fetch_add(1, Ordering::SeqCst);
        }
        with_body(&answered, without_context)
    });
    let proxy = proxy(server.address, no_retarget(), tasks, rewrite);
    let results = run_only(&proxy, &[SEND_TASK_CREATED]);
    assert!(
        stripped.load(Ordering::SeqCst) >= 2,
        "precondition: the answers carried a contextId the proxy removed"
    );
    assert_not_passed(
        &results,
        SEND_TASK_CREATED,
        "answered no contextId for any task, while the scenario claims the task's context",
    );
}

/// Control: an endpoint whose ListTasks answers no task. `Tasks` keeps the scenario's tasks of
/// what the endpoint listed, so the claim that it holds the created task fails.
#[test]
fn a_list_that_hides_the_task_fails_the_claim_on_tasks() {
    let server = idle();
    let lists = |method: &str, target: &str| method == "GET" && path(target).ends_with("/tasks");
    let rewrite: Rewrite = Arc::new(|_method: &str, _target: &str, answered: Vec<u8>| {
        with_body(&answered, |value| {
            if let Value::Object(members) = value {
                for (name, member) in members.iter_mut() {
                    if name == "tasks" {
                        *member = Value::Array(Vec::new());
                    }
                }
            }
        })
    });
    let proxy = proxy(server.address, no_retarget(), lists, rewrite);
    let results = run_only(&proxy, &[SEND_TASK_CREATED]);
    let (status, message) = outcome(&results, SEND_TASK_CREATED);
    assert_eq!(status, Some(Status::Failed), "{message}");
}

/// The first segments of a path that are routes, not a tenant.
const ROUTES: [&str; 5] = [
    "message:send",
    "message:stream",
    "tasks",
    "extendedAgentCard",
    ".well-known",
];

/// An endpoint that ignores the tenant (every request sent to the plain route), so it acts across
/// tenants: the scenarios the in-process target declares unsupported on gap A expect the command
/// to act across tenants, so none may come out `passed` against such an endpoint either.
///
/// Today each stops `unsupported` at the configuration event the binding does not carry
/// (`PushNotificationConfigCreated`, `PushNotificationConfigDeleted`), not at the gap A check:
/// `WireTarget` reads a success on another tenant's task as the outcome the suite expects. A
/// target that read those events from the answer would pass them, and this test goes red.
#[test]
fn gap_a_scenarios_do_not_pass_against_an_endpoint_that_acts_across_tenants() {
    let reasons = std::fs::read_to_string(root().join("crates/a2a-conformance/unsupported.tsv"))
        .expect("the unsupported list");
    let across: Vec<&str> = reasons
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .filter(|(_, reason)| reason.contains("on a task of another tenant"))
        .map(|(scenario, _)| scenario)
        .collect();
    assert!(!across.is_empty(), "precondition: gap A scenarios listed");

    let server = idle();
    let retarget: Retarget = Arc::new(|_method: &str, target: &str| {
        let rest = target.strip_prefix('/')?;
        let (first, after) = rest.split_once('/')?;
        let first = first.split('?').next().unwrap_or_default();
        (!ROUTES.contains(&first)).then(|| format!("/{after}"))
    });
    let proxy = proxy(server.address, retarget, never, no_rewrite());
    let results = run_only(&proxy, &across);
    let passed: Vec<String> = across
        .iter()
        .filter(|scenario| outcome(&results, scenario).0 == Some(Status::Passed))
        .map(|scenario| (*scenario).to_owned())
        .collect();
    assert!(
        passed.is_empty(),
        "passed against an endpoint that ignores the tenant: {passed:?}"
    );
}

/// story:testkit-wire-events acceptance 6, the other half: against the echo agent, which moves
/// every task by steps of its own, the five task scenarios are `unsupported` with a reason, never
/// `passed` and never `failed`.
#[test]
fn the_task_scenarios_are_unsupported_with_a_reason_against_the_echo_agent() {
    let on_a_task = [
        SEND_TASK_CREATED,
        "a2a.tasks.SendStreamingMessage/outcome/task-created",
        GET_TASK_FOUND,
        "a2a.tasks.CancelTask/outcome/canceled",
        SUBSCRIBED,
    ];
    let results = run_only(&echo(), &on_a_task);
    for scenario in on_a_task {
        let (status, message) = outcome(&results, scenario);
        assert_eq!(status, Some(Status::Unsupported), "{scenario}: {message}");
        assert!(
            !message.is_empty(),
            "{scenario} is unsupported without a reason"
        );
    }
}
