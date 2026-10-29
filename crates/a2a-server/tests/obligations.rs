//! Each obligation of `generated/rust/a2a/PLAN.md` over a real socket: its happy path and the
//! declared refusals the reference agent and the card can reach, answered as A2A v1.0.1 § 11
//! (HTTP+JSON) says.

mod common;

use a2a::discovery::AgentExtension;
use a2a::json::Value;
use a2a::tasks::{Message, TaskSnapshot};
use a2a_server::{Agent, Refusal, Settings, Step, Turn};
use common::{at, card, count, echo, echo_with, message, start, start_with, text};

// SendMessage

#[test]
fn send_message_creates_a_completed_task_holding_the_echo_artifact() {
    let server = echo();
    let reply = server.call("POST", "/message:send", Some(&message("hello", None, None)));
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert_eq!(reply.header("content-type"), Some("application/a2a+json"));
    let answer = reply.json();
    assert_eq!(
        text(&answer, &["task", "status", "state"]),
        "TASK_STATE_COMPLETED"
    );
    assert!(!text(&answer, &["task", "id"]).is_empty());
    assert!(!text(&answer, &["task", "contextId"]).is_empty());
    assert!(!text(&answer, &["task", "status", "timestamp"]).is_empty());
    assert_eq!(text(&answer, &["task", "artifacts", "0", "name"]), "echo");
    assert_eq!(
        text(&answer, &["task", "artifacts", "0", "parts", "0", "text"]),
        "hello"
    );
    assert_eq!(
        text(&answer, &["task", "history", "0", "role"]),
        "ROLE_USER"
    );
    assert!(at(&answer, &["message"]).is_none());
}

#[test]
fn send_message_answers_a_direct_message_without_a_task() {
    let server = echo();
    let body = "{\"message\":{\"messageId\":\"m-1\",\"role\":\"ROLE_USER\",\"parts\":[{\"text\":\"hi\"}],\"metadata\":{\"mode\":\"message\"}}}";
    let reply = server.call("POST", "/message:send", Some(body));
    assert_eq!(reply.status, 200, "{}", reply.body);
    let answer = reply.json();
    assert!(at(&answer, &["task"]).is_none(), "{}", reply.body);
    assert_eq!(text(&answer, &["message", "role"]), "ROLE_AGENT");
    assert_eq!(text(&answer, &["message", "parts", "0", "text"]), "hi");
}

#[test]
fn send_message_continues_a_task_waiting_for_input() {
    let server = echo();
    let (task, context) = server.waiting_task();
    let reply = server.call(
        "POST",
        "/message:send",
        Some(&message("now", Some(&task), Some(&context))),
    );
    assert_eq!(reply.status, 200, "{}", reply.body);
    let answer = reply.json();
    assert_eq!(text(&answer, &["task", "id"]), task);
    assert_eq!(
        text(&answer, &["task", "status", "state"]),
        "TASK_STATE_COMPLETED"
    );
    assert_eq!(
        text(&answer, &["task", "artifacts", "0", "parts", "0", "text"]),
        "now"
    );
    assert!(count(&answer, &["task", "history"]) >= 3, "{}", reply.body);
}

#[test]
fn send_message_to_an_unknown_task_is_task_not_found() {
    let server = echo();
    let reply = server.call(
        "POST",
        "/message:send",
        Some(&message("hello", Some("no-such-task"), None)),
    );
    assert_eq!(reply.status, 404, "{}", reply.body);
    assert_eq!(reply.rpc_status(), "NOT_FOUND");
    assert_eq!(reply.reason(), "TASK_NOT_FOUND");
}

#[test]
fn send_message_to_a_terminal_task_is_unsupported_operation() {
    let server = echo();
    let task = server.completed_task();
    let reply = server.call(
        "POST",
        "/message:send",
        Some(&message("again", Some(&task), None)),
    );
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.rpc_status(), "FAILED_PRECONDITION");
    assert_eq!(reply.reason(), "UNSUPPORTED_OPERATION");
}

#[test]
fn send_message_with_another_context_than_its_task_is_context_mismatch() {
    let server = echo();
    let (task, _) = server.waiting_task();
    let reply = server.call(
        "POST",
        "/message:send",
        Some(&message("now", Some(&task), Some("another-context"))),
    );
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.rpc_status(), "INVALID_ARGUMENT");
    assert_eq!(reply.reason(), "CONTEXT_MISMATCH");
}

#[test]
fn send_message_with_a_part_of_an_unaccepted_media_type_is_content_type_not_supported() {
    let server = echo();
    let body = "{\"message\":{\"messageId\":\"m-1\",\"role\":\"ROLE_USER\",\"parts\":[{\"url\":\"https://example.invalid/a.png\",\"mediaType\":\"image/png\"}]}}";
    let reply = server.call("POST", "/message:send", Some(body));
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.rpc_status(), "INVALID_ARGUMENT");
    assert_eq!(reply.reason(), "CONTENT_TYPE_NOT_SUPPORTED");
}

struct Picky;

impl Agent for Picky {
    fn turn(&mut self, _message: &Message, _task: Option<&TaskSnapshot>) -> Turn {
        Turn::Refuse(Refusal::Context)
    }
}

#[test]
fn send_message_with_a_context_the_agent_cannot_accept_is_context_not_accepted() {
    let server = start(card(|_| {}), || Box::new(Picky));
    let reply = server.call(
        "POST",
        "/message:send",
        Some(&message("hello", None, Some("foreign-context"))),
    );
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.reason(), "CONTEXT_NOT_ACCEPTED");
}

#[test]
fn send_message_in_a_version_the_card_does_not_declare_is_version_not_supported() {
    let server = echo();
    let reply = server.call_with(
        "POST",
        "/message:send",
        Some(&message("hello", None, None)),
        &[("A2A-Version", "9.9")],
    );
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.rpc_status(), "FAILED_PRECONDITION");
    assert_eq!(reply.reason(), "VERSION_NOT_SUPPORTED");
}

#[test]
fn send_message_without_a_version_is_0_3_and_the_query_parameter_names_one() {
    let server = echo();
    let without = server.call_with(
        "POST",
        "/message:send",
        Some(&message("hello", None, None)),
        &[],
    );
    assert_eq!(without.status, 400, "{}", without.body);
    assert_eq!(without.reason(), "VERSION_NOT_SUPPORTED");
    let by_query = server.call_with(
        "POST",
        "/message:send?A2A-Version=1.0",
        Some(&message("hello", None, None)),
        &[],
    );
    assert_eq!(by_query.status, 200, "{}", by_query.body);
    let patch = server.call_with(
        "POST",
        "/message:send",
        Some(&message("hello", None, None)),
        &[("A2A-Version", "1.0.7")],
    );
    assert_eq!(patch.status, 200, "{}", patch.body);
}

fn requiring(uri: &str) -> Settings {
    card(|card| {
        card.capabilities.extensions = Some(vec![AgentExtension {
            uri: Some(uri.to_owned()),
            description: None,
            required: Some(true),
            params: None,
        }]);
    })
}

#[test]
fn send_message_without_a_required_extension_is_extension_support_required() {
    let server = echo_with(requiring("https://example.invalid/ext/trace"));
    let without = server.call("POST", "/message:send", Some(&message("hello", None, None)));
    assert_eq!(without.status, 400, "{}", without.body);
    assert_eq!(without.reason(), "EXTENSION_SUPPORT_REQUIRED");
    let with = server.call_with(
        "POST",
        "/message:send",
        Some(&message("hello", None, None)),
        &[
            ("A2A-Version", "1.0"),
            (
                "A2A-Extensions",
                "https://example.invalid/ext/other, https://example.invalid/ext/trace",
            ),
        ],
    );
    assert_eq!(with.status, 200, "{}", with.body);
}

#[test]
fn send_message_returning_immediately_answers_the_task_before_the_agent_works() {
    let server = echo();
    let body = format!(
        "{{\"message\":{},\"configuration\":{{\"returnImmediately\":true}}}}",
        inner(&message("later", None, None))
    );
    let reply = server.call("POST", "/message:send", Some(&body));
    assert_eq!(reply.status, 200, "{}", reply.body);
    let answer = reply.json();
    assert_eq!(
        text(&answer, &["task", "status", "state"]),
        "TASK_STATE_SUBMITTED"
    );
    let task = text(&answer, &["task", "id"]);
    let later = server.call("GET", &format!("/tasks/{task}"), None);
    assert_eq!(later.status, 200, "{}", later.body);
    assert_eq!(
        text(&later.json(), &["status", "state"]),
        "TASK_STATE_COMPLETED"
    );
}

#[test]
fn send_message_history_length_zero_answers_no_history() {
    let server = echo();
    let body = format!(
        "{{\"message\":{},\"configuration\":{{\"historyLength\":0}}}}",
        inner(&message("hello", None, None))
    );
    let reply = server.call("POST", "/message:send", Some(&body));
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert_eq!(
        count(&reply.json(), &["task", "history"]),
        0,
        "{}",
        reply.body
    );
}

#[test]
fn send_message_with_a_malformed_body_is_invalid_argument() {
    let server = echo();
    let reply = server.call("POST", "/message:send", Some("{\"message\":7}"));
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.rpc_status(), "INVALID_ARGUMENT");
}

// GetTask

#[test]
fn get_task_answers_the_task() {
    let server = echo();
    let task = server.completed_task();
    let reply = server.call("GET", &format!("/tasks/{task}"), None);
    assert_eq!(reply.status, 200, "{}", reply.body);
    let answer = reply.json();
    assert_eq!(text(&answer, &["id"]), task);
    assert_eq!(text(&answer, &["status", "state"]), "TASK_STATE_COMPLETED");
    assert_eq!(count(&answer, &["artifacts"]), 1);
}

#[test]
fn get_task_history_length_keeps_the_latest_messages() {
    let server = echo();
    let (task, context) = server.waiting_task();
    let _ = server.call(
        "POST",
        "/message:send",
        Some(&message("now", Some(&task), Some(&context))),
    );
    let all = server.call("GET", &format!("/tasks/{task}"), None).json();
    assert!(count(&all, &["history"]) >= 3);
    let one = server
        .call("GET", &format!("/tasks/{task}?historyLength=1"), None)
        .json();
    assert_eq!(count(&one, &["history"]), 1);
    let last = count(&all, &["history"]) - 1;
    assert_eq!(
        text(&one, &["history", "0", "messageId"]),
        text(&all, &["history", &last.to_string(), "messageId"])
    );
}

#[test]
fn get_task_of_an_unknown_task_is_task_not_found() {
    let server = echo();
    let reply = server.call("GET", "/tasks/no-such-task", None);
    assert_eq!(reply.status, 404, "{}", reply.body);
    assert_eq!(reply.reason(), "TASK_NOT_FOUND");
}

// ListTasks (the `Tasks` view)

#[test]
fn list_tasks_answers_every_task_newest_first_without_artifacts() {
    let server = echo();
    let first = server.completed_task();
    let (second, _) = server.waiting_task();
    let reply = server.call("GET", "/tasks", None);
    assert_eq!(reply.status, 200, "{}", reply.body);
    let answer = reply.json();
    assert_eq!(count(&answer, &["tasks"]), 2, "{}", reply.body);
    assert_eq!(text(&answer, &["tasks", "0", "id"]), second);
    assert_eq!(text(&answer, &["tasks", "1", "id"]), first);
    assert!(
        at(&answer, &["tasks", "1", "artifacts"]).is_none(),
        "{}",
        reply.body
    );
    assert_eq!(
        at(&answer, &["nextPageToken"]),
        Some(&Value::Text(String::new()))
    );
    assert_eq!(number(&answer, "totalSize"), 2);
    assert_eq!(number(&answer, "pageSize"), 50);
}

#[test]
fn list_tasks_filters_by_context_and_status_and_includes_artifacts_on_request() {
    let server = echo();
    let done = server.completed_task();
    let (waiting, context) = server.waiting_task();
    let by_status = server
        .call(
            "GET",
            "/tasks?status=TASK_STATE_COMPLETED&includeArtifacts=true",
            None,
        )
        .json();
    assert_eq!(count(&by_status, &["tasks"]), 1);
    assert_eq!(text(&by_status, &["tasks", "0", "id"]), done);
    assert_eq!(count(&by_status, &["tasks", "0", "artifacts"]), 1);
    let by_context = server
        .call("GET", &format!("/tasks?contextId={context}"), None)
        .json();
    assert_eq!(count(&by_context, &["tasks"]), 1);
    assert_eq!(text(&by_context, &["tasks", "0", "id"]), waiting);
}

#[test]
fn list_tasks_pages_with_the_next_page_token() {
    let server = echo();
    for _ in 0..3 {
        let _ = server.completed_task();
    }
    let first = server.call("GET", "/tasks?pageSize=2", None).json();
    assert_eq!(count(&first, &["tasks"]), 2);
    assert_eq!(number(&first, "totalSize"), 3);
    let token = text(&first, &["nextPageToken"]);
    assert!(!token.is_empty());
    let second = server
        .call("GET", &format!("/tasks?pageSize=2&pageToken={token}"), None)
        .json();
    assert_eq!(count(&second, &["tasks"]), 1);
    assert_eq!(text(&second, &["nextPageToken"]), "");
}

#[test]
fn list_tasks_with_a_page_size_out_of_range_is_invalid_argument() {
    let server = echo();
    for size in ["0", "101"] {
        let reply = server.call("GET", &format!("/tasks?pageSize={size}"), None);
        assert_eq!(reply.status, 400, "{size}: {}", reply.body);
        assert_eq!(reply.rpc_status(), "INVALID_ARGUMENT");
    }
}

// CancelTask

#[test]
fn cancel_task_cancels_a_task_waiting_for_input() {
    let server = echo();
    let (task, _) = server.waiting_task();
    let reply = server.call("POST", &format!("/tasks/{task}:cancel"), Some("{}"));
    assert_eq!(reply.status, 200, "{}", reply.body);
    let answer = reply.json();
    assert_eq!(text(&answer, &["id"]), task);
    assert_eq!(text(&answer, &["status", "state"]), "TASK_STATE_CANCELED");
    let after = server.call("GET", &format!("/tasks/{task}"), None).json();
    assert_eq!(text(&after, &["status", "state"]), "TASK_STATE_CANCELED");
}

#[test]
fn cancel_task_on_completed_task_is_not_cancelable() {
    let server = echo();
    let task = server.completed_task();
    let reply = server.call("POST", &format!("/tasks/{task}:cancel"), None);
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.rpc_status(), "FAILED_PRECONDITION");
    assert_eq!(reply.reason(), "TASK_NOT_CANCELABLE");
    let after = server.call("GET", &format!("/tasks/{task}"), None).json();
    assert_eq!(text(&after, &["status", "state"]), "TASK_STATE_COMPLETED");
}

#[test]
fn cancel_task_of_an_unknown_task_is_task_not_found() {
    let server = echo();
    let reply = server.call("POST", "/tasks/no-such-task:cancel", None);
    assert_eq!(reply.status, 404, "{}", reply.body);
    assert_eq!(reply.reason(), "TASK_NOT_FOUND");
}

struct Stubborn;

impl Agent for Stubborn {
    fn turn(&mut self, _message: &Message, _task: Option<&TaskSnapshot>) -> Turn {
        Turn::Steps(vec![Step::Work(None), Step::RequireInput(None)])
    }

    fn cancelable(&self, _task: &TaskSnapshot) -> bool {
        false
    }
}

#[test]
fn cancel_task_the_agent_cannot_cancel_now_is_not_cancelable() {
    let server = start(card(|_| {}), || Box::new(Stubborn));
    let (task, _) = server.waiting_task();
    let reply = server.call("POST", &format!("/tasks/{task}:cancel"), None);
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.reason(), "TASK_NOT_CANCELABLE");
}

// Push notification configurations

#[test]
fn push_notification_config_is_created_read_listed_and_deleted() {
    let server = echo();
    let (task, _) = server.waiting_task();
    let created = server.call(
        "POST",
        &format!("/tasks/{task}/pushNotificationConfigs"),
        Some("{\"url\":\"https://example.invalid/hook\",\"token\":\"t-1\"}"),
    );
    assert_eq!(created.status, 200, "{}", created.body);
    let config = created.json();
    let id = text(&config, &["id"]);
    assert!(!id.is_empty(), "{}", created.body);
    assert_eq!(text(&config, &["taskId"]), task);
    assert_eq!(text(&config, &["url"]), "https://example.invalid/hook");

    let named = server.call(
        "POST",
        &format!("/tasks/{task}/pushNotificationConfigs"),
        Some("{\"id\":\"mine\",\"url\":\"https://example.invalid/other\"}"),
    );
    assert_eq!(named.status, 200, "{}", named.body);
    assert_eq!(text(&named.json(), &["id"]), "mine");

    let read = server.call(
        "GET",
        &format!("/tasks/{task}/pushNotificationConfigs/{id}"),
        None,
    );
    assert_eq!(read.status, 200, "{}", read.body);
    assert_eq!(text(&read.json(), &["token"]), "t-1");

    let listed = server.call(
        "GET",
        &format!("/tasks/{task}/pushNotificationConfigs"),
        None,
    );
    assert_eq!(listed.status, 200, "{}", listed.body);
    assert_eq!(count(&listed.json(), &["configs"]), 2, "{}", listed.body);

    let paged = server
        .call(
            "GET",
            &format!("/tasks/{task}/pushNotificationConfigs?pageSize=1"),
            None,
        )
        .json();
    assert_eq!(count(&paged, &["configs"]), 1);
    assert!(!text(&paged, &["nextPageToken"]).is_empty());

    let deleted = server.call(
        "DELETE",
        &format!("/tasks/{task}/pushNotificationConfigs/{id}"),
        None,
    );
    assert_eq!(deleted.status, 200, "{}", deleted.body);
    let gone = server.call(
        "GET",
        &format!("/tasks/{task}/pushNotificationConfigs/{id}"),
        None,
    );
    assert_eq!(gone.status, 404, "{}", gone.body);
    assert_eq!(gone.reason(), "TASK_NOT_FOUND");
}

#[test]
fn push_notification_config_of_an_unknown_task_is_task_not_found() {
    let server = echo();
    let reply = server.call(
        "POST",
        "/tasks/no-such-task/pushNotificationConfigs",
        Some("{\"url\":\"https://example.invalid/hook\"}"),
    );
    assert_eq!(reply.status, 404, "{}", reply.body);
    assert_eq!(reply.reason(), "TASK_NOT_FOUND");
    let listed = server.call("GET", "/tasks/no-such-task/pushNotificationConfigs", None);
    assert_eq!(listed.status, 404, "{}", listed.body);
}

#[test]
fn push_notification_config_without_the_capability_is_push_notification_not_supported() {
    let server = echo_with(card(|card| {
        card.capabilities.push_notifications = Some(false)
    }));
    let (task, _) = server.waiting_task();
    let reply = server.call(
        "POST",
        &format!("/tasks/{task}/pushNotificationConfigs"),
        Some("{\"url\":\"https://example.invalid/hook\"}"),
    );
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.rpc_status(), "FAILED_PRECONDITION");
    assert_eq!(reply.reason(), "PUSH_NOTIFICATION_NOT_SUPPORTED");
}

// GetExtendedAgentCard and the public card

#[test]
fn extended_agent_card_is_answered_when_declared_and_configured() {
    let mut settings = card(|card| card.capabilities.extended_agent_card = Some(true));
    let mut extended = settings.card.clone();
    extended.name = "the extended card".to_owned();
    settings.extended_card = Some(extended);
    let server = start_with(
        settings,
        a2a_server::Limits::DEFAULT,
        a2a_server::Access {
            extended_card_token: a2a_server::BearerToken::new("t0ken"),
        },
    );
    let reply = server.call_with(
        "GET",
        "/extendedAgentCard",
        None,
        &[("A2A-Version", "1.0"), ("Authorization", "Bearer t0ken")],
    );
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert_eq!(text(&reply.json(), &["name"]), "the extended card");
}

#[test]
fn extended_agent_card_without_the_capability_is_unsupported_operation() {
    let server = echo();
    let reply = server.call("GET", "/extendedAgentCard", None);
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.reason(), "UNSUPPORTED_OPERATION");
}

#[test]
fn extended_agent_card_declared_but_not_configured_is_not_configured() {
    let server = echo_with(card(|card| {
        card.capabilities.extended_agent_card = Some(true)
    }));
    let reply = server.call("GET", "/extendedAgentCard", None);
    assert_eq!(reply.status, 400, "{}", reply.body);
    assert_eq!(reply.reason(), "EXTENDED_AGENT_CARD_NOT_CONFIGURED");
}

#[test]
fn the_public_agent_card_is_served_at_the_well_known_path() {
    let server = echo();
    let reply = server.call_with("GET", "/.well-known/agent-card.json", None, &[]);
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert_eq!(reply.header("content-type"), Some("application/json"));
    let card = a2a_server::read_card(&reply.body).expect("an AgentCard");
    assert_eq!(card, a2a_server::default_card());
}

// Tenants and the error shape

#[test]
fn tenant_routes_answer_as_the_plain_routes_do() {
    let server = echo();
    let reply = server.call(
        "POST",
        "/acme/message:send",
        Some(&message("hello", None, None)),
    );
    assert_eq!(reply.status, 200, "{}", reply.body);
    let task = text(&reply.json(), &["task", "id"]);
    let read = server.call("GET", &format!("/acme/tasks/{task}"), None);
    assert_eq!(read.status, 200, "{}", read.body);
    let listed = server.call("GET", "/acme/tasks", None);
    assert_eq!(listed.status, 200, "{}", listed.body);
    let canceled = server.call("POST", &format!("/acme/tasks/{task}:cancel"), None);
    assert_eq!(canceled.reason(), "TASK_NOT_CANCELABLE");
    let configs = server.call(
        "POST",
        &format!("/acme/tasks/{task}/pushNotificationConfigs"),
        Some("{\"url\":\"https://example.invalid/hook\"}"),
    );
    assert_eq!(configs.status, 200, "{}", configs.body);
    assert_eq!(text(&configs.json(), &["tenant"]), "acme");
}

#[test]
fn errors_are_google_rpc_status_objects() {
    let server = echo();
    let reply = server.call("GET", "/tasks/no-such-task", None);
    assert_eq!(reply.header("content-type"), Some("application/a2a+json"));
    let answer = reply.json();
    assert_eq!(number(at(&answer, &["error"]).expect("error"), "code"), 404);
    assert_eq!(text(&answer, &["error", "status"]), "NOT_FOUND");
    assert!(!text(&answer, &["error", "message"]).is_empty());
    assert_eq!(
        text(&answer, &["error", "details", "0", "@type"]),
        "type.googleapis.com/google.rpc.ErrorInfo"
    );
    assert_eq!(
        text(&answer, &["error", "details", "0", "domain"]),
        "a2a-protocol.org"
    );
}

fn number(value: &Value, name: &str) -> i64 {
    match value.member(name) {
        Some(Value::Number(digits)) => digits.parse().unwrap_or(-1),
        other => panic!("{name} is not a number: {other:?}"),
    }
}

/// The `message` member of a request body `message` built.
fn inner(body: &str) -> String {
    body.strip_prefix("{\"message\":")
        .and_then(|rest| rest.strip_suffix('}'))
        .expect("a body `message` built")
        .to_owned()
}
