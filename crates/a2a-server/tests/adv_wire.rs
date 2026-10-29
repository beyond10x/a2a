//! Adversarial cases against the HTTP+JSON wire behaviour of `a2a-server` (story:server, wave 3).

mod common;

use a2a::json::Value;
use common::{at, echo, message, text};

fn ids(list: &Value) -> Vec<String> {
    match at(list, &["tasks"]) {
        Some(Value::Array(tasks)) => tasks.iter().map(|task| text(task, &["id"])).collect(),
        _ => Vec::new(),
    }
}

/// upstream/a2a.proto:694-696: "Only tasks with a status timestamp time greater than or equal to
/// this value will be returned."
#[test]
fn adv_list_tasks_status_timestamp_after_includes_a_task_stamped_at_that_instant() {
    let server = echo();
    let id = server.completed_task();
    let task = server.call("GET", &format!("/tasks/{id}"), None).json();
    let stamp = text(&task, &["status", "timestamp"]);
    assert!(!stamp.is_empty(), "the task carries no status timestamp");
    let listed = server.call("GET", &format!("/tasks?statusTimestampAfter={stamp}"), None);
    assert_eq!(listed.status, 200, "{}", listed.body);
    assert!(
        ids(&listed.json()).contains(&id),
        "statusTimestampAfter={stamp} (the task's own status timestamp) left the task out: {}",
        listed.body
    );
}

/// specification.md:254 "MUST use cursor-based pagination for performance and consistency", :258
/// "rather than offset-based pagination".
#[test]
fn adv_list_tasks_second_page_does_not_repeat_the_first_when_a_task_arrives_between() {
    let server = echo();
    let older = server.completed_task();
    let newer = server.completed_task();
    let first = server.call("GET", "/tasks?pageSize=1", None);
    assert_eq!(first.status, 200, "{}", first.body);
    let first = first.json();
    assert_eq!(ids(&first), std::slice::from_ref(&newer));
    let token = text(&first, &["nextPageToken"]);
    assert!(!token.is_empty(), "no next page token");
    let _arrived = server.completed_task();
    let second = server.call("GET", &format!("/tasks?pageSize=1&pageToken={token}"), None);
    assert_eq!(second.status, 200, "{}", second.body);
    let second_ids = ids(&second.json());
    assert_eq!(
        second_ids,
        [older],
        "page 2 (token `{token}`) after one new task: {}",
        second.body
    );
}

/// specification.md:1236-1237: "Millisecond precision **SHOULD** be used", pattern
/// `YYYY-MM-DDTHH:mm:ss.sssZ`.
#[test]
fn adv_status_timestamps_have_millisecond_precision() {
    let server = echo();
    let id = server.completed_task();
    let task = server.call("GET", &format!("/tasks/{id}"), None).json();
    let stamp = text(&task, &["status", "timestamp"]);
    let fraction = stamp
        .split_once('.')
        .map(|(_, rest)| rest.trim_end_matches('Z'))
        .unwrap_or("");
    assert_eq!(
        fraction.len(),
        3,
        "`{stamp}` is not YYYY-MM-DDTHH:mm:ss.sssZ"
    );
}

/// specification.md:1169 (section 5.3) and :2795 (section 11.3.2) name `POST /tasks/{id}:subscribe`;
/// upstream/a2a.proto:78 and upstream/openapi/a2a.openapi.yaml:289-290 bind it to GET.
/// Coordinator decision (wave 3, fix 1): "Proto is normative (specification.md:107), proto and
/// openapi say GET". So the route stays GET, and POST is refused with 405 naming GET.
#[test]
fn adv_subscribe_refuses_post_and_names_get_as_the_proto_binds_it() {
    let server = echo();
    let (task, _) = server.waiting_task();
    let refused = server.call("POST", &format!("/tasks/{task}:subscribe"), None);
    assert_eq!(
        (refused.status, refused.header("Allow")),
        (405, Some("GET")),
        "POST /tasks/{{id}}:subscribe: {} {:?}",
        refused.body,
        refused.headers
    );
}

/// specification.md:467 "This parameter follows consistent semantics across all operations"; the
/// server refuses `historyLength=-1` on ListTasks with 400 (router.rs list()).
#[test]
fn adv_get_task_refuses_a_negative_history_length_as_list_tasks_does() {
    let server = echo();
    let id = server.completed_task();
    let listed = server.call("GET", "/tasks?historyLength=-1", None);
    assert_eq!(listed.status, 400, "{}", listed.body);
    let got = server.call("GET", &format!("/tasks/{id}?historyLength=-1"), None);
    assert_eq!(
        got.status, 400,
        "ListTasks refuses historyLength=-1 but GetTask answers: {}",
        got.body
    );
}

/// specification.md:1275 "Implementations **SHOULD** ignore unrecognized fields in messages".
#[test]
fn adv_list_tasks_ignores_an_unrecognized_query_field() {
    let server = echo();
    let _ = server.completed_task();
    let listed = server.call("GET", "/tasks?futureField=1", None);
    assert_eq!(listed.status, 200, "{}", listed.body);
}

/// specification.md:1275, the same rule on a bound route.
#[test]
fn adv_get_task_ignores_an_unrecognized_query_field() {
    let server = echo();
    let id = server.completed_task();
    let got = server.call("GET", &format!("/tasks/{id}?futureField=1"), None);
    assert_eq!(got.status, 200, "{}", got.body);
}

/// specification.md:1275, the same rule in a request body.
#[test]
fn adv_send_message_ignores_an_unrecognized_body_field() {
    let server = echo();
    let body = message("hello", None, None);
    let body = format!(
        "{},\"futureField\":1}}",
        body.strip_suffix('}').expect("an object")
    );
    let sent = server.call("POST", "/message:send", Some(&body));
    assert_eq!(sent.status, 200, "{body} -> {}", sent.body);
}
