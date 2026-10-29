//! Acceptance 1: one client method per operation of A2A's HTTP+JSON binding, each with and without
//! a tenant. A socket records the (method, path) every call sends; together they are exactly the
//! (method, path) pairs of `upstream/openapi/a2a.openapi.yaml`, plus the public card.

mod common;

use std::collections::BTreeSet;
use std::path::Path;

use a2a::discovery::GetExtendedAgentCard;
use a2a::tasks::{
    CancelTask, CreateTaskPushNotificationConfig, DeleteTaskPushNotificationConfig, GetTask,
    GetTaskPushNotificationConfig, ListTaskPushNotificationConfigs, PushNotificationConfigId,
    SubscribeToTask, TaskId,
};
use a2a_client::ListTasksRequest;
use common::{answer, answering, client_of, message, send, stream};

/// The one pair the client sends that the upstream OpenAPI does not list: the public card
/// (A2A v1.0.1 section 8.2, RFC 8615), which upstream's proto has no RPC for.
const PUBLIC_CARD: (&str, &str) = ("get", "/.well-known/agent-card.json");

/// Every (method, path template) of the upstream OpenAPI document.
fn upstream_pairs() -> BTreeSet<(String, String)> {
    let file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../upstream/openapi/a2a.openapi.yaml");
    let text = std::fs::read_to_string(&file).expect("the upstream OpenAPI document");
    let mut pairs = BTreeSet::new();
    let mut in_paths = false;
    let mut path = None;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            in_paths = line.trim_end() == "paths:";
            continue;
        }
        if !in_paths {
            continue;
        }
        if let Some(rest) = line.strip_prefix("  ")
            && !rest.starts_with(' ')
        {
            path = Some(
                rest.trim_end()
                    .trim_end_matches(':')
                    .trim_matches('"')
                    .to_owned(),
            );
            continue;
        }
        if let Some(rest) = line.strip_prefix("    ")
            && !rest.starts_with(' ')
        {
            let method = rest.trim_end().trim_end_matches(':');
            if ["get", "post", "put", "delete", "patch"].contains(&method) {
                let path = path.clone().expect("a method under a path");
                pairs.insert((method.to_owned(), path));
            }
        }
    }
    pairs
}

/// Whether `path` is an instance of `template`: each `{name}` holds one non-empty segment, a
/// whole-segment variable holds no `:verb`, and a variable may be followed by a literal suffix.
fn fits(template: &str, path: &str) -> bool {
    let template: Vec<&str> = template.split('/').collect();
    let given: Vec<&str> = path.split('/').collect();
    template.len() == given.len()
        && template.iter().zip(&given).all(|(expected, text)| {
            match expected
                .strip_prefix('{')
                .and_then(|rest| rest.split_once('}'))
            {
                Some((_, suffix)) => text.strip_suffix(suffix).is_some_and(|value| {
                    !value.is_empty() && (!suffix.is_empty() || !value.contains(':'))
                }),
                None => expected == text,
            }
        })
}

/// Calls every client method once under `tenant` against a socket that answers each with an
/// error, and answers the (method, path) pairs the socket saw.
fn sent(tenant: Option<&str>) -> Vec<(String, String)> {
    let body = "{\"error\":{\"code\":404,\"status\":\"NOT_FOUND\",\"message\":\"recorded\",\"details\":[]}}";
    let (address, seen) = answering(answer(404, "application/a2a+json", body));
    let client = client_of(address);
    let task = TaskId("t-1".to_owned());
    let config = PushNotificationConfigId("c-1".to_owned());
    let owned = tenant.map(str::to_owned);

    if tenant.is_none() {
        let _ = client.agent_card();
    }
    let _ = client.extended_agent_card(&GetExtendedAgentCard {
        tenant: owned.clone(),
    });
    let _ = client.send_message(&send(message("hello"), tenant));
    let _ = client.send_streaming_message(&stream(message("hello"), tenant));
    let _ = client.get_task(&GetTask {
        tenant: owned.clone(),
        id: task.clone(),
        history_length: None,
    });
    let _ = client.list_tasks(&ListTasksRequest {
        tenant: owned.clone(),
        ..ListTasksRequest::default()
    });
    let _ = client.cancel_task(&CancelTask {
        tenant: owned.clone(),
        id: task.clone(),
        metadata: None,
    });
    let _ = client.subscribe_to_task(&SubscribeToTask {
        tenant: owned.clone(),
        id: task.clone(),
    });
    let _ = client.create_task_push_notification_config(&CreateTaskPushNotificationConfig {
        tenant: owned.clone(),
        id: None,
        task_id: task.clone(),
        url: "https://example.invalid/hook".to_owned(),
        token: None,
        authentication: None,
    });
    let _ = client.get_task_push_notification_config(&GetTaskPushNotificationConfig {
        tenant: owned.clone(),
        task_id: task.clone(),
        id: config.clone(),
    });
    let _ = client.list_task_push_notification_configs(&ListTaskPushNotificationConfigs {
        tenant: owned.clone(),
        task_id: task.clone(),
        page_size: None,
        page_token: None,
    });
    let _ = client.delete_task_push_notification_config(&DeleteTaskPushNotificationConfig {
        tenant: owned,
        task_id: task,
        id: config,
    });

    let seen = seen.lock().expect("the list");
    seen.iter()
        .map(|request| (request.method.to_ascii_lowercase(), request.path.clone()))
        .collect()
}

#[test]
fn every_upstream_route_is_sent_and_nothing_else() {
    let upstream = upstream_pairs();
    assert_eq!(
        upstream.len(),
        22,
        "the upstream routes were not read: {upstream:?}"
    );
    let plain = sent(None);
    let tenant = sent(Some("acme"));
    assert_eq!(plain.len(), 12, "a plain call was not sent: {plain:?}");
    assert_eq!(tenant.len(), 11, "a tenant call was not sent: {tenant:?}");

    let mut reached = BTreeSet::new();
    for (method, path) in plain.iter().chain(&tenant) {
        if (method.as_str(), path.as_str()) == PUBLIC_CARD {
            continue;
        }
        let matched: Vec<_> = upstream
            .iter()
            .filter(|(verb, template)| verb == method && fits(template, path))
            .collect();
        assert!(
            !matched.is_empty(),
            "sent {method} {path}, which the upstream OpenAPI does not list"
        );
        reached.extend(matched.into_iter().cloned());
    }
    for (method, path) in &tenant {
        assert!(
            path.starts_with("/acme/"),
            "a call with a tenant went to {method} {path}"
        );
    }
    for (method, path) in &plain {
        assert!(
            !path.starts_with("/acme/"),
            "a call without a tenant went to {method} {path}"
        );
    }
    let missed: Vec<_> = upstream.difference(&reached).collect();
    assert!(missed.is_empty(), "no method sends {missed:?}");
    assert!(
        plain
            .iter()
            .any(|(method, path)| (method.as_str(), path.as_str()) == PUBLIC_CARD),
        "the public card was not fetched: {plain:?}"
    );
}

#[test]
fn the_route_matcher_tells_verbs_and_tenants_apart() {
    assert!(fits("/tasks/{id}", "/tasks/t-1"));
    assert!(!fits("/tasks/{id}", "/tasks/t-1:cancel"));
    assert!(fits("/tasks/{id}:cancel", "/tasks/t-1:cancel"));
    assert!(!fits("/{tenant}/tasks", "/tasks/t-1"));
    assert!(fits("/{tenant}/tasks/{id}", "/acme/tasks/t-1"));
    assert!(!fits("/tasks/{id}", "/tasks/"));
}
