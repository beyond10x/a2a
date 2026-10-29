//! Security review: a tenant or id value selects a path segment and nothing else. It cannot
//! inject a header or a second request line, change the origin, or step out of its place in the
//! route through a dot segment (RFC 3986 section 5.2.4) or an empty one.

mod common;

use a2a::discovery::GetExtendedAgentCard;
use a2a::tasks::{
    CancelTask, DeleteTaskPushNotificationConfig, GetTask, GetTaskPushNotificationConfig,
    ListTaskPushNotificationConfigs, PushNotificationConfigId, SubscribeToTask, TaskId,
};
use a2a_client::{Client, Error};
use common::{Seen, answer, answering};

/// One operation called on a client, its answer dropped.
type Operation = Box<dyn Fn(&Client) -> Result<(), Error>>;

/// Calls the operations a tenant and ids select a route segment in, under base path
/// `/agents/a`, and answers what each sent (or that it was refused before sending).
fn sent(tenant: Option<&str>, id: &str) -> Vec<(String, Result<Seen, String>)> {
    let (address, seen) = answering(answer(404, "application/json", "{}"));
    let client = Client::new(&format!("http://{address}/agents/a")).expect("a base URL");
    let tenant = tenant.map(str::to_owned);
    let task = TaskId(id.to_owned());
    let config = PushNotificationConfigId(id.to_owned());
    let calls: Vec<(&str, Operation)> = vec![
        ("GetExtendedAgentCard", {
            let tenant = tenant.clone();
            Box::new(move |client| {
                client
                    .extended_agent_card(&GetExtendedAgentCard {
                        tenant: tenant.clone(),
                    })
                    .map(|_| ())
            })
        }),
        ("GetTask", {
            let (tenant, task) = (tenant.clone(), task.clone());
            Box::new(move |client| {
                client
                    .get_task(&GetTask {
                        tenant: tenant.clone(),
                        id: task.clone(),
                        history_length: None,
                    })
                    .map(|_| ())
            })
        }),
        ("CancelTask", {
            let (tenant, task) = (tenant.clone(), task.clone());
            Box::new(move |client| {
                client
                    .cancel_task(&CancelTask {
                        tenant: tenant.clone(),
                        id: task.clone(),
                        metadata: None,
                    })
                    .map(|_| ())
            })
        }),
        ("SubscribeToTask", {
            let (tenant, task) = (tenant.clone(), task.clone());
            Box::new(move |client| {
                client
                    .subscribe_to_task(&SubscribeToTask {
                        tenant: tenant.clone(),
                        id: task.clone(),
                    })
                    .map(|_| ())
            })
        }),
        ("ListTaskPushNotificationConfigs", {
            let (tenant, task) = (tenant.clone(), task.clone());
            Box::new(move |client| {
                client
                    .list_task_push_notification_configs(&ListTaskPushNotificationConfigs {
                        tenant: tenant.clone(),
                        task_id: task.clone(),
                        page_size: None,
                        page_token: None,
                    })
                    .map(|_| ())
            })
        }),
        ("GetTaskPushNotificationConfig", {
            let (tenant, config) = (tenant.clone(), config.clone());
            Box::new(move |client| {
                client
                    .get_task_push_notification_config(&GetTaskPushNotificationConfig {
                        tenant: tenant.clone(),
                        task_id: TaskId("t-1".to_owned()),
                        id: config.clone(),
                    })
                    .map(|_| ())
            })
        }),
        ("DeleteTaskPushNotificationConfig", {
            let (tenant, config) = (tenant.clone(), config.clone());
            Box::new(move |client| {
                client
                    .delete_task_push_notification_config(&DeleteTaskPushNotificationConfig {
                        tenant: tenant.clone(),
                        task_id: TaskId("t-1".to_owned()),
                        id: config.clone(),
                    })
                    .map(|_| ())
            })
        }),
    ];
    let mut out = Vec::new();
    for (name, call) in calls {
        let before = seen.lock().expect("the list").len();
        let answered = call(&client);
        let list = seen.lock().expect("the list");
        let result = match (list.len() > before, answered) {
            (true, _) => Ok(list[before].clone()),
            (false, Err(Error::InvalidRequest(detail))) => Err(detail),
            (false, other) => panic!("{name}: nothing was sent and the answer is {other:?}"),
        };
        out.push((name.to_owned(), result));
    }
    out
}

/// The segments of a sent path below the base path `/agents/a` that are empty, `.` or `..`.
fn bad_segments(path: &str) -> Vec<String> {
    let below = path
        .strip_prefix("/agents/a/")
        .unwrap_or_else(|| panic!("{path} is not under the base path"));
    below
        .split('/')
        .filter(|segment| segment.is_empty() || *segment == "." || *segment == "..")
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_tenant_or_id_never_becomes_a_dot_segment_or_an_empty_one() {
    let mut escaped = Vec::new();
    let cases = [
        (Some(".."), "t-1"),
        (Some("."), "t-1"),
        (None, ".."),
        (None, "."),
        (None, ""),
    ];
    for (tenant, id) in cases {
        for (name, result) in sent(tenant, id) {
            if let Ok(request) = result {
                let bad = bad_segments(&request.path);
                if !bad.is_empty() {
                    escaped.push(format!(
                        "tenant {tenant:?} id {id:?}: {name} sent {} {}",
                        request.method, request.path
                    ));
                }
            }
        }
    }
    assert!(
        escaped.is_empty(),
        "a tenant or id value became a dot or empty path segment, which a normalizing server or \
         proxy resolves to another route:\n{}",
        escaped.join("\n")
    );
}

#[test]
fn a_tenant_or_id_cannot_inject_a_header_a_request_line_or_another_origin() {
    let hostile = "x\r\nX-Injected: 1\r\n\r\nGET /evil HTTP/1.1 @other.example:80/../?#%";
    for (tenant, id) in [
        (Some(hostile), "t-1"),
        (None, hostile),
        (Some(hostile), hostile),
    ] {
        for (name, result) in sent(tenant, id) {
            let request = result.unwrap_or_else(|detail| {
                panic!("{name}: an encodable value was refused: {detail}")
            });
            let first = request.head.lines().next().unwrap_or_default();
            assert_eq!(
                first.split(' ').count(),
                3,
                "{name}: the request line is not three words: {first}"
            );
            assert!(
                !request.head.contains("\nX-Injected") && !request.head.contains("GET /evil"),
                "{name}: a value reached the head unencoded: {}",
                request.head
            );
            assert!(
                request.path.starts_with("/agents/a/")
                    && request
                        .path
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"-._~%/:".contains(&byte)),
                "{name}: {}",
                request.path
            );
        }
    }
}
