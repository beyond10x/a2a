//! Route coverage: every (method, path) of `upstream/openapi/a2a.openapi.yaml` is served; every
//! hand route has its row in `ess/http-unmapped.tsv`; every bound route is one the generated layer
//! serves; and the generated layer's own routes are not exposed.

mod common;

use std::collections::BTreeSet;

use common::echo;

const OPENAPI: &str = include_str!("../../../upstream/openapi/a2a.openapi.yaml");
const UNMAPPED: &str = include_str!("../../../ess/http-unmapped.tsv");

/// The (method, path) pairs of the upstream OpenAPI document, methods in upper case.
fn upstream() -> BTreeSet<(String, String)> {
    let mut routes = BTreeSet::new();
    let mut in_paths = false;
    let mut path = None;
    for line in OPENAPI.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            in_paths = line == "paths:";
            continue;
        }
        if !in_paths {
            continue;
        }
        if let Some(key) = line.strip_prefix("  ").filter(|key| !key.starts_with(' ')) {
            let key = key.strip_suffix(':').expect("a path key ends with `:`");
            path = Some(key.trim_matches('"').to_owned());
            continue;
        }
        if let Some(method) = line
            .strip_prefix("    ")
            .filter(|key| !key.starts_with(' '))
        {
            let method = method.strip_suffix(':').unwrap_or(method);
            if ["get", "post", "put", "patch", "delete"].contains(&method) {
                let path = path.clone().expect("a method under a path");
                routes.insert((method.to_uppercase(), path));
            }
        }
    }
    routes
}

/// The rows of `ess/http-unmapped.tsv` of this kind, as (method, path).
fn unmapped(kind: &str) -> BTreeSet<(String, String)> {
    UNMAPPED
        .lines()
        .filter_map(|line| {
            let mut columns = line.split('\t');
            let row_kind = columns.next()?;
            let method = columns.next()?;
            let path = columns.next()?;
            (row_kind == kind).then(|| (method.to_uppercase(), path.to_owned()))
        })
        .collect()
}

fn served(kind: Option<&str>) -> BTreeSet<(String, String)> {
    a2a_server::served()
        .into_iter()
        .filter(|(_, _, served_as)| kind.is_none_or(|kind| kind == *served_as))
        .map(|(method, path, _)| (method.to_owned(), path))
        .collect()
}

#[test]
fn the_upstream_document_is_read() {
    assert_eq!(upstream().len(), 22, "{:#?}", upstream());
    assert_eq!(unmapped("upstream").len(), 13);
}

#[test]
fn every_upstream_route_is_served() {
    let missing: Vec<_> = upstream().difference(&served(None)).cloned().collect();
    assert!(
        missing.is_empty(),
        "upstream routes not served: {missing:#?}"
    );
}

#[test]
fn every_hand_route_has_its_unmapped_row_and_every_unmapped_row_is_served() {
    let rows = unmapped("upstream");
    let hand: BTreeSet<_> = served(Some("hand"))
        .into_iter()
        .filter(|(_, path)| path != "/.well-known/agent-card.json")
        .collect();
    let without_row: Vec<_> = hand.difference(&rows).cloned().collect();
    assert!(
        without_row.is_empty(),
        "hand routes without a row in ess/http-unmapped.tsv: {without_row:#?}"
    );
    let unserved: Vec<_> = rows.difference(&hand).cloned().collect();
    assert!(
        unserved.is_empty(),
        "unmapped rows not served by a hand route: {unserved:#?}"
    );
    assert!(
        served(Some("hand"))
            .contains(&("GET".to_owned(), "/.well-known/agent-card.json".to_owned()))
    );
}

#[test]
fn every_bound_route_is_a_route_of_the_generated_layer() {
    let generated: BTreeSet<_> = a2a::server::a2a_agent::ROUTES
        .iter()
        .map(|(method, path)| ((*method).to_owned(), (*path).to_owned()))
        .collect();
    let bound = served(Some("bound"));
    assert_eq!(bound.len(), 9, "{bound:#?}");
    let foreign: Vec<_> = bound.difference(&generated).cloned().collect();
    assert!(
        foreign.is_empty(),
        "bound routes the generated layer does not serve: {foreign:#?}"
    );
    let upstream = upstream();
    assert!(
        bound.is_subset(&upstream),
        "bound routes are upstream routes"
    );
}

fn concrete(path: &str) -> String {
    path.replace("{tenant}", "acme")
        .replace("{taskId}", "no-such-task")
        .replace("{id}", "no-such-task")
}

#[test]
fn every_served_route_answers_and_none_is_a_route_miss() {
    let server = echo();
    for (method, path, _) in a2a_server::served() {
        let body = (method != "GET").then_some("{}");
        let reply = server.call(method, &concrete(&path), body);
        assert_ne!(reply.status, 405, "{method} {path}: {}", reply.body);
        if reply.status == 404 {
            assert_eq!(
                reply.reason(),
                "TASK_NOT_FOUND",
                "{method} {path} is a route miss: {}",
                reply.body
            );
        }
        assert!(
            !reply.body.contains("\"outcome\"") && !reply.body.contains("\"refused\""),
            "{method} {path} leaks the generated envelope: {}",
            reply.body
        );
    }
}

#[test]
fn an_unknown_path_is_not_found_and_another_method_is_not_allowed() {
    let server = echo();
    let missing = server.call("GET", "/nothing-here", None);
    assert_eq!(missing.status, 404, "{}", missing.body);
    assert_eq!(missing.rpc_status(), "NOT_FOUND");
    let wrong = server.call("DELETE", "/message:send", None);
    assert_eq!(wrong.status, 405, "{}", wrong.body);
    assert_eq!(wrong.header("allow"), Some("POST"));
    let verb = server.call("GET", "/tasks/t-1:cancel", None);
    assert_eq!(verb.status, 405, "{}", verb.body);
    assert_eq!(verb.header("allow"), Some("POST"));
    let configs = server.call("PUT", "/tasks/t-1/pushNotificationConfigs", None);
    assert_eq!(configs.status, 405, "{}", configs.body);
    assert_eq!(configs.header("allow"), Some("GET, POST"));
}

#[test]
fn the_generated_layers_own_routes_are_not_exposed() {
    let server = echo();
    let bound = served(Some("bound"));
    let mut hidden: BTreeSet<(String, String)> = a2a::server::a2a_agent::ROUTES
        .iter()
        .map(|(method, path)| ((*method).to_owned(), (*path).to_owned()))
        .filter(|route| !bound.contains(route))
        .collect();
    hidden.extend(unmapped("projected"));
    assert!(hidden.len() >= 14, "{hidden:#?}");
    for (method, path) in hidden {
        let body = (method != "GET").then_some("{}");
        let reply = server.call(&method, &path, body);
        assert_ne!(
            reply.status, 200,
            "{method} {path} is exposed: {}",
            reply.body
        );
        assert!(
            !reply.body.contains("\"outcome\"")
                && !reply.body.contains("\"rows\"")
                && !reply.body.contains("\"refused\""),
            "{method} {path} answers from the generated layer: {}",
            reply.body
        );
    }
}
