//! Security review of story:server (wave 3): tenants whose segment reads like a route word or
//! carries an encoded slash, page tokens carried across tenants, push-notification URLs, and the
//! extended agent card.

mod common;

use std::io::ErrorKind;
use std::net::TcpListener;
use std::time::Duration;

use a2a::json::Value;
use a2a_server::Settings;
use common::{Reply, Server, at, echo, echo_with, message, text};

fn ids(reply: &Reply) -> Vec<String> {
    match at(&reply.json(), &["tasks"]) {
        Some(Value::Array(tasks)) => tasks.iter().map(|task| text(task, &["id"])).collect(),
        _ => panic!("no tasks: {}", reply.body),
    }
}

fn created(server: &Server, prefix: &str, words: &str) -> String {
    let reply = server.call(
        "POST",
        &format!("{prefix}/message:send"),
        Some(&message(words, None, None)),
    );
    assert_eq!(
        reply.status, 200,
        "POST {prefix}/message:send: {}",
        reply.body
    );
    text(&reply.json(), &["task", "id"])
}

/// A tenant segment that equals a route word, or holds an encoded slash, scopes its tasks like
/// any other: the plain routes and the tenant the segment would read as without decoding do not
/// reach them.
#[test]
fn sec_tenants_named_like_route_words_or_with_encoded_slashes_stay_apart() {
    let server = echo();
    let mut leaks = Vec::new();
    for (prefix, neighbour) in [
        ("/tasks", ""),
        ("/message:send", ""),
        ("/.well-known", ""),
        ("/extendedAgentCard", ""),
        ("/acme%2Fbeta", "/acme"),
        ("/acme%2Fbeta", "/acme%252Fbeta"),
        ("/%2F", ""),
        ("/%2E%2E", ""),
    ] {
        let task = created(&server, prefix, "");
        for (method, path) in [
            ("GET", format!("{neighbour}/tasks/{task}")),
            ("POST", format!("{neighbour}/tasks/{task}:cancel")),
            ("GET", format!("{neighbour}/tasks/{task}:subscribe")),
            (
                "GET",
                format!("{neighbour}/tasks/{task}/pushNotificationConfigs"),
            ),
        ] {
            let reply = server.call(method, &path, None);
            if reply.status != 404 {
                leaks.push(format!(
                    "a task made under {prefix}: {method} {path} answered {} {}",
                    reply.status, reply.body
                ));
            }
        }
        let listed = server.call("GET", &format!("{neighbour}/tasks"), None);
        if listed.status != 200 || ids(&listed).contains(&task) {
            leaks.push(format!(
                "a task made under {prefix}: GET {neighbour}/tasks answered {} {}",
                listed.status, listed.body
            ));
        }
        let own = server.call("GET", &format!("{prefix}/tasks/{task}"), None);
        if own.status != 200 {
            leaks.push(format!(
                "a task made under {prefix} is not read back under it: {} {}",
                own.status, own.body
            ));
        }
    }
    assert!(leaks.is_empty(), "{leaks:#?}");
}

/// `totalSize` counts the request's tenant only, and a page token issued under one tenant pages
/// only the tasks of the tenant it is used under.
#[test]
fn sec_page_tokens_and_totals_stay_within_their_tenant() {
    let server = echo();
    let mut acme = Vec::new();
    for _ in 0..3 {
        acme.push(created(&server, "/acme", "hello"));
    }
    let beta = created(&server, "/beta", "hello");
    let first = server.call("GET", "/acme/tasks?pageSize=1", None);
    assert_eq!(first.status, 200, "{}", first.body);
    let token = text(&first.json(), &["nextPageToken"]);
    assert!(!token.is_empty(), "no token: {}", first.body);
    let total = |reply: &Reply| at(&reply.json(), &["totalSize"]).cloned();
    let carried = server.call("GET", &format!("/beta/tasks?pageToken={token}"), None);
    let plain = server.call("GET", &format!("/tasks?pageToken={token}"), None);
    let beta_all = server.call("GET", "/beta/tasks", None);
    let carried_ids = if carried.status == 200 {
        ids(&carried)
    } else {
        Vec::new()
    };
    let plain_ids = if plain.status == 200 {
        ids(&plain)
    } else {
        Vec::new()
    };
    assert!(
        total(&first) == Some(Value::Number("3".to_owned()))
            && total(&beta_all) == Some(Value::Number("1".to_owned()))
            && carried_ids.iter().all(|id| *id == beta)
            && plain_ids.is_empty()
            && !acme
                .iter()
                .any(|id| carried_ids.contains(id) || plain_ids.contains(id)),
        "acme's first page {}; acme's token under beta {} {}; under no tenant {} {}; beta's \
         list {}",
        first.body,
        carried.status,
        carried.body,
        plain.status,
        plain.body,
        beta_all.body
    );
}

/// A push-notification config's URL is kept and answered, and nothing connects to it: not when
/// the config is made, nor when its task moves, completes or is canceled.
#[test]
fn sec_a_push_config_url_is_never_called() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.set_nonblocking(true).expect("nonblocking");
    let hook = format!("http://{}/hook", listener.local_addr().expect("address"));
    let server = echo();
    let (waiting, _) = server.waiting_task();
    let (canceled, _) = server.waiting_task();
    for task in [&waiting, &canceled] {
        let reply = server.call(
            "POST",
            &format!("/tasks/{task}/pushNotificationConfigs"),
            Some(&format!(
                "{{\"url\":{},\"token\":\"t\"}}",
                common::quoted(&hook)
            )),
        );
        assert_eq!(reply.status, 200, "{}", reply.body);
    }
    let completed = server.call(
        "POST",
        "/message:send",
        Some(&message("done", Some(&waiting), None)),
    );
    assert_eq!(completed.status, 200, "{}", completed.body);
    let cancel = server.call("POST", &format!("/tasks/{canceled}:cancel"), None);
    assert_eq!(cancel.status, 200, "{}", cancel.body);
    std::thread::sleep(Duration::from_millis(1500));
    let called = match listener.accept() {
        Ok((_, peer)) => Some(peer.to_string()),
        Err(error) if error.kind() == ErrorKind::WouldBlock => None,
        Err(error) => Some(format!("accept failed: {error}")),
    };
    assert!(
        called.is_none(),
        "the server connected to the config's URL: {called:?}"
    );
}

/// ess/domains/discovery.yaml:368 names the caller of GetExtendedAgentCard "an authenticated
/// caller of the agent's A2A interface", as upstream/a2a.proto:121 does ("for the authenticated
/// agent"). router.rs `caller` hands every request that actor, so the card given with
/// `--extended-card` answers a request that carries no credentials at all.
#[test]
fn sec_the_extended_card_is_not_served_to_a_caller_without_credentials() {
    let mut card = a2a_server::default_card();
    card.capabilities.extended_agent_card = Some(true);
    let mut extended = a2a_server::default_card();
    extended.name = "the extended card".to_owned();
    let server = echo_with(Settings {
        card,
        extended_card: Some(extended),
    });
    let reply = server.call("GET", "/extendedAgentCard", None);
    assert!(
        reply.status != 200,
        "GET /extendedAgentCard with no Authorization header answered {} {}",
        reply.status,
        reply.body.chars().take(80).collect::<String>()
    );
}
