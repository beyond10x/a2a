//! Acceptance 1: the target's mapping from an ESS command or view to a client call is derived
//! from `ess/http.yaml`. `OPERATIONS` is held to the binding's routes, to the A2A routes
//! `ess/http-unmapped.tsv` names because the binding cannot carry them, and to the plain routes of
//! `upstream/openapi/`; each command is sent as the method and path its row names.

mod common;

use std::collections::BTreeSet;

use a2a_conformance::Target;
use a2a_testkit::{OPERATIONS, WireTarget};
use common::{error_answer, recording, root, value};

/// Every route of `ess/http.yaml`: the command or view, its method and its path.
fn binding_routes() -> Vec<(String, String, String)> {
    let text = std::fs::read_to_string(root().join("ess/http.yaml")).expect("ess/http.yaml");
    let mut routes = Vec::new();
    let mut name: Option<String> = None;
    let mut method: Option<String> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line
            .strip_prefix("- command:")
            .or_else(|| line.strip_prefix("- view:"))
        {
            name = Some(rest.trim().to_owned());
            method = None;
        } else if let Some(rest) = line.strip_prefix("method:") {
            method = Some(rest.trim().to_owned());
        } else if let Some(rest) = line.strip_prefix("path:")
            && let (Some(name), Some(method)) = (name.take(), method.take())
        {
            routes.push((name, method, rest.trim().to_owned()));
        }
    }
    routes
}

/// Every row of `ess/http-unmapped.tsv`: side, method, path, reason.
fn unmapped() -> Vec<[String; 4]> {
    let text = std::fs::read_to_string(root().join("ess/http-unmapped.tsv"))
        .expect("ess/http-unmapped.tsv");
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let fields: Vec<&str> = line.splitn(4, '\t').collect();
            assert_eq!(fields.len(), 4, "a row of four fields: {line}");
            [fields[0], fields[1], fields[2], fields[3]].map(str::to_owned)
        })
        .collect()
}

/// Every (method, path) of `upstream/openapi/a2a.openapi.yaml` without the `/{tenant}` prefix.
fn upstream_plain() -> BTreeSet<(String, String)> {
    let text = std::fs::read_to_string(root().join("upstream/openapi/a2a.openapi.yaml"))
        .expect("the upstream OpenAPI document");
    let mut pairs = BTreeSet::new();
    let mut in_paths = false;
    let mut path: Option<String> = None;
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
            if ["get", "post", "put", "delete", "patch"].contains(&method)
                && let Some(path) = &path
                && !path.starts_with("/{tenant}")
            {
                pairs.insert((method.to_owned(), path.clone()));
            }
        }
    }
    pairs
}

#[test]
fn operations_are_the_binding_of_ess_http_yaml_and_the_routes_it_cannot_carry() {
    let routes = binding_routes();
    assert_eq!(routes.len(), 10, "the routes were not read: {routes:?}");
    let rows = unmapped();
    let operations: Vec<(String, String, String)> = OPERATIONS
        .iter()
        .map(|(name, method, path)| ((*name).to_owned(), (*method).to_owned(), (*path).to_owned()))
        .collect();

    for (name, method, path) in &routes {
        if operations.contains(&(name.clone(), method.clone(), path.clone())) {
            continue;
        }
        let named = rows.iter().any(|[side, row_method, row_path, reason]| {
            side == "projected"
                && row_method == method
                && row_path == path
                && reason.starts_with("Not an A2A operation")
        });
        assert!(
            named,
            "ess/http.yaml routes {name} as {method} {path}, which OPERATIONS does not carry \
             and ess/http-unmapped.tsv does not name as no A2A operation"
        );
    }

    for (name, method, path) in &operations {
        if routes.contains(&(name.clone(), method.clone(), path.clone())) {
            continue;
        }
        assert!(
            !routes.iter().any(|(routed, _, _)| routed == name),
            "{name} is routed in ess/http.yaml, and OPERATIONS gives it {method} {path}"
        );
        let upstream = rows.iter().any(|[side, row_method, row_path, _]| {
            side == "upstream" && row_method == method && row_path == path
        });
        let stand_in = rows.iter().any(|[side, _, _, reason]| {
            side == "projected"
                && reason.contains(&format!("UNMAPPED {method} {path}:"))
                && reason.contains(&format!("derived route of {name} "))
        });
        assert!(
            upstream && stand_in,
            "OPERATIONS gives {name} {method} {path}, which neither ess/http.yaml routes nor \
             ess/http-unmapped.tsv names as its unmapped upstream route"
        );
    }

    let carried: BTreeSet<(String, String)> = operations
        .iter()
        .map(|(_, method, path)| (method.clone(), path.clone()))
        .collect();
    assert_eq!(carried.len(), operations.len(), "one operation per route");
    assert_eq!(
        carried,
        upstream_plain(),
        "every plain A2A route, and no other"
    );
}

#[test]
fn each_command_is_sent_as_the_method_and_path_of_its_operation() {
    let inputs = [
        (
            "a2a.discovery.GetExtendedAgentCard",
            "{}",
            "/extendedAgentCard",
        ),
        (
            "a2a.tasks.SendMessage",
            r#"{"message":{"message_id":"m","role":"ROLE_USER","parts":[]}}"#,
            "/message:send",
        ),
        (
            "a2a.tasks.SendStreamingMessage",
            r#"{"message":{"message_id":"m","role":"ROLE_USER","parts":[]}}"#,
            "/message:stream",
        ),
        ("a2a.tasks.GetTask", r#"{"id":"t-1"}"#, "/tasks/t-1"),
        (
            "a2a.tasks.CancelTask",
            r#"{"id":"t-1"}"#,
            "/tasks/t-1:cancel",
        ),
        (
            "a2a.tasks.SubscribeToTask",
            r#"{"id":"t-1"}"#,
            "/tasks/t-1:subscribe",
        ),
        (
            "a2a.tasks.CreateTaskPushNotificationConfig",
            r#"{"task_id":"t-1","url":"http://example.invalid/hook"}"#,
            "/tasks/t-1/pushNotificationConfigs",
        ),
        (
            "a2a.tasks.GetTaskPushNotificationConfig",
            r#"{"task_id":"t-1","id":"c-1"}"#,
            "/tasks/t-1/pushNotificationConfigs/c-1",
        ),
        (
            "a2a.tasks.ListTaskPushNotificationConfigs",
            r#"{"task_id":"t-1"}"#,
            "/tasks/t-1/pushNotificationConfigs",
        ),
        (
            "a2a.tasks.DeleteTaskPushNotificationConfig",
            r#"{"task_id":"t-1","id":"c-1"}"#,
            "/tasks/t-1/pushNotificationConfigs/c-1",
        ),
    ];
    let commands: BTreeSet<&str> = inputs.iter().map(|(name, _, _)| *name).collect();
    let views: BTreeSet<&str> = OPERATIONS
        .iter()
        .map(|(name, _, _)| *name)
        .filter(|name| !commands.contains(name))
        .collect();
    assert_eq!(
        views,
        BTreeSet::from(["a2a.tasks.Tasks"]),
        "every command of OPERATIONS is sent here"
    );

    for (command, input, sent) in inputs {
        let (socket, seen) = recording(error_answer(404, "NOT_FOUND", "TASK_NOT_FOUND"));
        let mut target = WireTarget::new(socket.client());
        target.begin(command).expect("a context opens");
        let client = if command.starts_with("a2a.discovery.") {
            "a2a.discovery.Client"
        } else {
            "a2a.tasks.Client"
        };
        let _ = target.execute(command, Some(client), &value(input));
        let (_, method, _) = OPERATIONS
            .iter()
            .find(|(name, _, _)| *name == command)
            .expect("an operation of the command");
        let seen = seen.lock().expect("the list");
        assert_eq!(seen.len(), 1, "{command} sent {seen:?}");
        assert_eq!(seen[0].method, method.to_ascii_uppercase(), "{command}");
        assert_eq!(seen[0].target, sent, "{command}");
    }
}
