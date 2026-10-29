//! The operations of A2A's HTTP+JSON binding (A2A v1.0.1 section 11.3) and the ESS command or
//! view each carries.
//!
//! The rows are the routes of `ess/http.yaml` that are A2A paths, and the two A2A routes it cannot
//! bind, which `ess/http-unmapped.tsv` names (`POST /tasks/{id}:cancel` and
//! `GET /tasks/{id}:subscribe`, ESS-HTTP-007). `tests/binding.rs` fails on a row that differs
//! from either file or from the plain routes of `upstream/openapi/`, and on a command the target
//! does not send as its row says.

/// Each ESS command or view A2A's HTTP+JSON binding has an operation for: its qualified name, and
/// the operation's method and path template without the `/{tenant}` prefix a request's tenant
/// adds.
pub const OPERATIONS: [(&str, &str, &str); 11] = [
    (
        "a2a.discovery.GetExtendedAgentCard",
        "get",
        "/extendedAgentCard",
    ),
    ("a2a.tasks.SendMessage", "post", "/message:send"),
    ("a2a.tasks.SendStreamingMessage", "post", "/message:stream"),
    ("a2a.tasks.GetTask", "get", "/tasks/{id}"),
    ("a2a.tasks.Tasks", "get", "/tasks"),
    ("a2a.tasks.CancelTask", "post", "/tasks/{id}:cancel"),
    ("a2a.tasks.SubscribeToTask", "get", "/tasks/{id}:subscribe"),
    (
        "a2a.tasks.CreateTaskPushNotificationConfig",
        "post",
        "/tasks/{taskId}/pushNotificationConfigs",
    ),
    (
        "a2a.tasks.GetTaskPushNotificationConfig",
        "get",
        "/tasks/{taskId}/pushNotificationConfigs/{id}",
    ),
    (
        "a2a.tasks.ListTaskPushNotificationConfigs",
        "get",
        "/tasks/{taskId}/pushNotificationConfigs",
    ),
    (
        "a2a.tasks.DeleteTaskPushNotificationConfig",
        "delete",
        "/tasks/{taskId}/pushNotificationConfigs/{id}",
    ),
];

/// Whether A2A has an operation that runs or reads `name`.
pub(crate) fn carried(name: &str) -> bool {
    OPERATIONS.iter().any(|(carried, _, _)| *carried == name)
}
