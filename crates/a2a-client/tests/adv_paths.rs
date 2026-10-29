//! Adversary, story:client: how a tenant and an id become path segments, and what the request
//! head carries. Each case records the request a socket on port 0 read.

mod common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use a2a::tasks::{
    DeleteTaskPushNotificationConfig, GetTask, GetTaskPushNotificationConfig,
    ListTaskPushNotificationConfigs, PushNotificationConfigId, TaskId,
};
use a2a_client::{Client, Error, ListTasksRequest};
use common::{Seen, answer, answering};

const TASK_BODY: &str = "{\"id\":\"t-1\",\"status\":{\"state\":\"TASK_STATE_COMPLETED\"}}";

fn recorder(body: &str) -> (Client, String, Arc<Mutex<Vec<Seen>>>) {
    let (address, seen) = answering(answer(200, "application/a2a+json", body));
    let client = Client::new(&format!("http://{address}/agents/a1"))
        .expect("a base URL")
        .with_timeout(Some(Duration::from_secs(10)));
    (client, address.to_string(), seen)
}

/// The request target the socket read last, as it was on the request line.
fn target_of(seen: &Arc<Mutex<Vec<Seen>>>) -> Option<String> {
    let seen = seen.lock().expect("the list");
    let request = seen.last()?;
    request.head.split_whitespace().nth(1).map(str::to_owned)
}

/// The segments of a path after its leading `/`.
fn segments(path: &str) -> Vec<&str> {
    path.split('?')
        .next()
        .unwrap_or(path)
        .strip_prefix('/')
        .unwrap_or(path)
        .split('/')
        .collect()
}

/// A tenant or id of `.` or `..` is a dot segment once written into the path as it is: RFC 3986
/// section 5.2.4 removes it, and a proxy or server that normalizes the path (nginx does by
/// default) routes the request elsewhere. A tenant `..` under the base path `/agents/a1` leaves
/// it: `/agents/a1/../tasks/x` is `/agents/tasks/x`. The client refuses such a value or
/// encodes it so that it is no dot segment.
#[test]
fn a_dot_segment_is_never_sent_as_a_path_segment() {
    let mut wrong = Vec::new();
    for (what, tenant, task, config) in [
        ("tenant ..", Some(".."), "x", "c"),
        ("tenant .", Some("."), "x", "c"),
        ("task id ..", None, "..", "c"),
        ("task id .", None, ".", "c"),
        ("config id ..", None, "x", ".."),
    ] {
        let (client, _, seen) = recorder(TASK_BODY);
        let answered = if what.starts_with("config") {
            client.delete_task_push_notification_config(&DeleteTaskPushNotificationConfig {
                tenant: tenant.map(str::to_owned),
                task_id: TaskId(task.to_owned()),
                id: PushNotificationConfigId(config.to_owned()),
            })
        } else {
            client
                .get_task(&GetTask {
                    tenant: tenant.map(str::to_owned),
                    id: TaskId(task.to_owned()),
                    history_length: None,
                })
                .map(|_| ())
        };
        if matches!(answered, Err(Error::InvalidRequest(_))) {
            continue;
        }
        let target = target_of(&seen).unwrap_or_else(|| panic!("{what}: nothing was sent"));
        if segments(&target).iter().any(|s| *s == "." || *s == "..") {
            wrong.push(format!("{what}: sent the dot segment in `{target}`"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// An empty task or config id becomes an empty path segment (`/tasks/`,
/// `/tasks//pushNotificationConfigs/c`): another route on a router that ignores a trailing
/// slash or merges slashes (nginx `merge_slashes` is on by default). The client refuses it.
#[test]
fn an_empty_id_is_never_sent_as_an_empty_path_segment() {
    let mut wrong = Vec::new();
    for what in ["GetTask id", "GetTaskPushNotificationConfig taskId"] {
        let (client, _, seen) = recorder(TASK_BODY);
        let answered = if what.starts_with("GetTask ") {
            client
                .get_task(&GetTask {
                    tenant: None,
                    id: TaskId(String::new()),
                    history_length: None,
                })
                .map(|_| ())
        } else {
            client
                .get_task_push_notification_config(&GetTaskPushNotificationConfig {
                    tenant: None,
                    task_id: TaskId(String::new()),
                    id: PushNotificationConfigId("c".to_owned()),
                })
                .map(|_| ())
        };
        if matches!(answered, Err(Error::InvalidRequest(_))) {
            continue;
        }
        let target = target_of(&seen).unwrap_or_else(|| panic!("{what}: nothing was sent"));
        if segments(&target).iter().any(|s| s.is_empty()) {
            wrong.push(format!(
                "{what} empty: sent the empty segment in `{target}`"
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// Guards `segment`: every byte but the unreserved ones is percent-encoded, upper-case hex,
/// UTF-8 per byte; the tenant cannot add a segment, a query or a fragment.
#[test]
fn reserved_and_unicode_characters_are_percent_encoded_exactly() {
    let (client, _, seen) = recorder(TASK_BODY);
    client
        .get_task(&GetTask {
            tenant: Some("a/b?c#d%e f".to_owned()),
            id: TaskId("\u{fc} x+y:z~-._".to_owned()),
            history_length: None,
        })
        .expect("the task");
    assert_eq!(
        target_of(&seen).as_deref(),
        Some("/agents/a1/a%2Fb%3Fc%23d%25e%20f/tasks/%C3%BC%20x%2By%3Az~-._")
    );
}

/// Guards the `Host` header: the base URL's authority, port included.
#[test]
fn the_host_header_is_the_base_url_s_authority() {
    let (client, authority, seen) = recorder(TASK_BODY);
    client
        .get_task(&GetTask {
            tenant: None,
            id: TaskId("t-1".to_owned()),
            history_length: None,
        })
        .expect("the task");
    let head = seen.lock().expect("the list")[0].head.clone();
    assert!(
        head.contains(&format!("\r\nHost: {authority}\r\n")),
        "{head}"
    );
    assert_eq!(head.matches("\r\nHost:").count(), 1, "{head}");
}

/// Guards the query: a page token or context id holding `&`, `=`, `#` or a space adds no
/// parameter and no fragment.
#[test]
fn query_values_cannot_add_a_parameter_or_a_fragment() {
    let (client, _, seen) = recorder("{\"tasks\":[]}");
    client
        .list_tasks(&ListTasksRequest {
            context_id: Some(a2a::tasks::ContextId("c&status=X".to_owned())),
            page_token: Some("p#q r&pageSize=1".to_owned()),
            ..ListTasksRequest::default()
        })
        .expect("the list");
    let target = target_of(&seen).expect("a request");
    let (_, query) = target.split_once('?').expect("a query");
    let names: Vec<_> = query
        .split('&')
        .map(|pair| pair.split('=').next().unwrap_or_default())
        .collect();
    assert_eq!(names, ["contextId", "pageToken"], "{target}");
    assert!(!target.contains('#'), "{target}");

    let (client, _, seen) = recorder("{}");
    client
        .list_task_push_notification_configs(&ListTaskPushNotificationConfigs {
            tenant: None,
            task_id: TaskId("t-1".to_owned()),
            page_size: Some(2),
            page_token: Some("a&pageSize=9#f".to_owned()),
        })
        .expect("the list");
    let target = target_of(&seen).expect("a request");
    let (_, query) = target.split_once('?').expect("a query");
    assert_eq!(query.matches('&').count(), 1, "{target}");
    assert!(!target.contains('#'), "{target}");
}
