//! The bounds a server keeps ([`Limits`]) and who reads its extended card ([`Access`]): a
//! request over a bound is answered 429 and keeps nothing, a connection is held no longer than
//! its deadline, and other clients are still served.

mod common;

use std::io::{BufRead as _, BufReader, ErrorKind, Read as _, Write as _};
use std::net::{SocketAddr, TcpStream};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use a2a::json::Value;
use a2a_server::{Access, BearerToken, Limits};
use common::{Server, at, card, count, message, read_head, start_with, text};

fn limited(limits: Limits) -> Server {
    start_with(card(|_| {}), limits, Access::default())
}

fn card_served(server: &Server) -> bool {
    server
        .call("GET", "/.well-known/agent-card.json", None)
        .status
        == 200
}

/// Opens a stream following `task`: its status, and the connection, left open.
fn subscribe(server: &Server, task: &str) -> (u16, BufReader<TcpStream>) {
    let stream = server.send(
        "GET",
        &format!("/tasks/{task}:subscribe"),
        None,
        &[("A2A-Version", "1.0")],
    );
    let mut reader = BufReader::new(stream);
    let (status, _) = read_head(&mut reader);
    (status, reader)
}

fn history(server: &Server, task: &str) -> usize {
    let reply = server.call("GET", &format!("/tasks/{task}"), None);
    assert_eq!(reply.status, 200, "{}", reply.body);
    count(&reply.json(), &["history"])
}

/// A whole request for the public card on a new connection, its answer read to its end; the
/// connection is left open for the client to go on sending.
fn answered(address: SocketAddr) -> TcpStream {
    let mut stream = TcpStream::connect(address).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("read timeout");
    stream
        .write_all(
            format!("GET /.well-known/agent-card.json HTTP/1.1\r\nHost: {address}\r\n\r\n")
                .as_bytes(),
        )
        .expect("write the request");
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).expect("the answer");
    assert!(raw.starts_with(b"HTTP/1.1 200"), "{raw:?}");
    stream
}

// Connections (item 1): the deadline, and who gives way at the cap.

#[test]
fn a_client_still_sending_after_its_answer_is_closed_once_the_timeout_passes_again() {
    let server = limited(Limits {
        request_timeout: Duration::from_secs(1),
        ..Limits::DEFAULT
    });
    let mut stream = answered(server.address);
    let started = Instant::now();
    let mut closed = false;
    while started.elapsed() < Duration::from_secs(6) {
        if stream.write_all(b"x").is_err() {
            closed = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        closed && started.elapsed() < Duration::from_secs(4),
        "a client sending a byte every 100 ms after its answer, under a 1 s timeout, was \
         still connected after {:?}",
        started.elapsed()
    );
}

#[test]
fn a_connection_that_sends_nothing_is_closed_at_its_deadline_without_an_answer() {
    let server = limited(Limits {
        request_timeout: Duration::from_secs(1),
        ..Limits::DEFAULT
    });
    let mut stream = TcpStream::connect(server.address).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(6)))
        .expect("read timeout");
    let started = Instant::now();
    let mut raw = Vec::new();
    let outcome = stream.read_to_end(&mut raw);
    let waited = started.elapsed();
    let timed_out = matches!(
        &outcome,
        Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut)
    );
    assert!(
        !timed_out && raw.is_empty() && waited < Duration::from_secs(4),
        "an idle connection under a 1 s timeout answered {:?} after {waited:?} ({outcome:?})",
        String::from_utf8_lossy(&raw)
    );
}

#[test]
fn clients_still_sending_after_their_answers_give_way_to_a_new_client_at_the_cap() {
    let server = limited(Limits {
        max_connections: 4,
        ..Limits::DEFAULT
    });
    let mut held: Vec<TcpStream> = (0..4).map(|_| answered(server.address)).collect();
    for stream in &mut held {
        let _ = stream.write_all(b"x");
    }
    let started = Instant::now();
    let served = card_served(&server);
    assert!(
        served && started.elapsed() < Duration::from_millis(1500),
        "with every one of 4 connections answered and still sending, a new client was served: \
         {served}, after {:?}",
        started.elapsed()
    );
}

// Streams (item 3): one more than a bound is answered 429 before it opens.

#[test]
fn a_stream_over_its_task_s_bound_is_answered_429_and_other_tasks_are_followed() {
    let server = limited(Limits {
        max_subscribers_per_task: 2,
        ..Limits::DEFAULT
    });
    let (task, _) = server.waiting_task();
    let (other, _) = server.waiting_task();
    let first = subscribe(&server, &task);
    let second = subscribe(&server, &task);
    assert_eq!((first.0, second.0), (200, 200));
    let over = server.call("GET", &format!("/tasks/{task}:subscribe"), None);
    assert_eq!(over.status, 429, "{}", over.body);
    assert_eq!(over.rpc_status(), "RESOURCE_EXHAUSTED");
    let elsewhere = subscribe(&server, &other);
    assert_eq!(elsewhere.0, 200);
    assert!(card_served(&server));
}

#[test]
fn a_streaming_message_to_a_task_at_its_bound_is_answered_429_and_not_kept() {
    let server = limited(Limits {
        max_subscribers_per_task: 1,
        ..Limits::DEFAULT
    });
    let (task, context) = server.waiting_task();
    let open = subscribe(&server, &task);
    assert_eq!(open.0, 200);
    let before = history(&server, &task);
    let over = server.call(
        "POST",
        "/message:stream",
        Some(&message("more", Some(&task), Some(&context))),
    );
    assert_eq!(over.status, 429, "{}", over.body);
    assert_eq!(over.rpc_status(), "RESOURCE_EXHAUSTED");
    assert_eq!(history(&server, &task), before);
    let state = server.call("GET", &format!("/tasks/{task}"), None);
    assert_eq!(
        text(&state.json(), &["status", "state"]),
        "TASK_STATE_INPUT_REQUIRED"
    );
}

#[test]
fn streams_over_the_bound_across_tasks_are_answered_429_until_one_ends() {
    let server = limited(Limits {
        max_subscribers: 2,
        ..Limits::DEFAULT
    });
    let tasks: Vec<String> = (0..3).map(|_| server.waiting_task().0).collect();
    let first = subscribe(&server, &tasks[0]);
    let second = subscribe(&server, &tasks[1]);
    assert_eq!((first.0, second.0), (200, 200));
    let over = server.call("GET", &format!("/tasks/{}:subscribe", tasks[2]), None);
    assert_eq!(over.status, 429, "{}", over.body);
    assert_eq!(over.rpc_status(), "RESOURCE_EXHAUSTED");
    let streamed = server.call("POST", "/message:stream", Some(&message("hi", None, None)));
    assert_eq!(streamed.status, 429, "{}", streamed.body);
    drop(first);
    let started = Instant::now();
    let mut admitted = false;
    while !admitted && started.elapsed() < Duration::from_secs(3) {
        admitted = subscribe(&server, &tasks[2]).0 == 200;
        if !admitted {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    assert!(
        admitted,
        "a stream was refused after another's client hung up"
    );
}

// Caps (item 5): over a cap, 429 and nothing stored.

#[test]
fn a_tenant_at_its_task_cap_is_answered_429_and_nothing_is_stored() {
    let server = limited(Limits {
        max_tasks: 2,
        ..Limits::DEFAULT
    });
    let (waiting, context) = server.waiting_task();
    server.completed_task();
    let over = server.call("POST", "/message:send", Some(&message("hello", None, None)));
    assert_eq!(over.status, 429, "{}", over.body);
    assert_eq!(over.rpc_status(), "RESOURCE_EXHAUSTED");
    let streamed = server.call(
        "POST",
        "/message:stream",
        Some(&message("hello", None, None)),
    );
    assert_eq!(streamed.status, 429, "{}", streamed.body);
    let listed = server.call("GET", "/tasks", None);
    assert_eq!(
        at(&listed.json(), &["totalSize"]),
        Some(&Value::Number("2".to_owned())),
        "{}",
        listed.body
    );
    let continued = server.call(
        "POST",
        "/message:send",
        Some(&message("hello", Some(&waiting), Some(&context))),
    );
    assert_eq!(continued.status, 200, "{}", continued.body);
    let elsewhere = server.call(
        "POST",
        "/acme/message:send",
        Some(&message("hello", None, None)),
    );
    assert_eq!(elsewhere.status, 200, "{}", elsewhere.body);
}

#[test]
fn a_message_to_a_task_at_its_history_cap_is_answered_429_and_not_kept() {
    let server = limited(Limits {
        max_history: 4,
        ..Limits::DEFAULT
    });
    let (task, context) = server.waiting_task();
    let mut refused = None;
    for _ in 0..8 {
        let before = history(&server, &task);
        let reply = server.call(
            "POST",
            "/message:send",
            Some(&message("", Some(&task), Some(&context))),
        );
        if reply.status == 429 {
            refused = Some((before, reply));
            break;
        }
        assert_eq!(reply.status, 200, "{}", reply.body);
    }
    let (before, reply) = refused.expect("a message past the history cap is refused");
    assert_eq!(reply.rpc_status(), "RESOURCE_EXHAUSTED");
    let after = history(&server, &task);
    assert_eq!(after, before);
    assert!(after <= 4, "the history holds {after} messages");
}

#[test]
fn a_task_at_its_push_config_cap_is_answered_429_and_a_replacement_is_not() {
    let server = limited(Limits {
        max_push_configs: 2,
        ..Limits::DEFAULT
    });
    let (task, _) = server.waiting_task();
    let create = |id: &str| {
        server.call(
            "POST",
            &format!("/tasks/{task}/pushNotificationConfigs"),
            Some(&format!(
                "{{\"id\":\"{id}\",\"url\":\"https://example.invalid/hook\"}}"
            )),
        )
    };
    assert_eq!(create("c-1").status, 200);
    assert_eq!(create("c-2").status, 200);
    let over = create("c-3");
    assert_eq!(over.status, 429, "{}", over.body);
    assert_eq!(over.rpc_status(), "RESOURCE_EXHAUSTED");
    let replaced = create("c-1");
    assert_eq!(replaced.status, 200, "{}", replaced.body);
    let listed = server.call(
        "GET",
        &format!("/tasks/{task}/pushNotificationConfigs"),
        None,
    );
    assert_eq!(count(&listed.json(), &["configs"]), 2, "{}", listed.body);
}

// The extended card's token (item 7).

const TOKEN: &str = "s3cret.Token-0~";

fn guarded(token: Option<BearerToken>) -> Server {
    let mut settings = card(|card| card.capabilities.extended_agent_card = Some(true));
    let mut extended = settings.card.clone();
    extended.name = "the extended card".to_owned();
    settings.extended_card = Some(extended);
    start_with(
        settings,
        Limits::DEFAULT,
        Access {
            extended_card_token: token,
        },
    )
}

#[test]
fn the_extended_card_is_served_for_its_bearer_token_alone() {
    let server = guarded(BearerToken::new(TOKEN));
    let get = |path: &str, authorization: Option<&str>| {
        let mut headers = vec![("A2A-Version", "1.0")];
        if let Some(authorization) = authorization {
            headers.push(("Authorization", authorization));
        }
        server.call_with("GET", path, None, &headers)
    };
    let right = format!("Bearer {TOKEN}");
    let lower = format!("bearer {TOKEN}");
    let longer = format!("Bearer {TOKEN}x");
    let shorter = format!("Bearer {}", &TOKEN[..TOKEN.len() - 1]);
    let basic = format!("Basic {TOKEN}");
    for (path, authorization) in [
        ("/extendedAgentCard", None),
        ("/extendedAgentCard", Some(longer.as_str())),
        ("/extendedAgentCard", Some(shorter.as_str())),
        ("/extendedAgentCard", Some(basic.as_str())),
        ("/extendedAgentCard", Some("Bearer")),
        ("/acme/extendedAgentCard", None),
    ] {
        let reply = get(path, authorization);
        assert_eq!(
            reply.status, 401,
            "{path} {authorization:?}: {}",
            reply.body
        );
        assert_eq!(reply.header("WWW-Authenticate"), Some("Bearer"));
        assert_eq!(reply.rpc_status(), "UNAUTHENTICATED");
        assert!(!reply.body.contains("the extended card"), "{}", reply.body);
        assert!(!reply.body.contains(TOKEN), "{}", reply.body);
    }
    for (path, authorization) in [
        ("/extendedAgentCard", right.as_str()),
        ("/extendedAgentCard", lower.as_str()),
        ("/acme/extendedAgentCard", right.as_str()),
    ] {
        let reply = get(path, Some(authorization));
        assert_eq!(reply.status, 200, "{path}: {}", reply.body);
        assert_eq!(text(&reply.json(), &["name"]), "the extended card");
    }
}

#[test]
fn a_server_without_a_token_serves_its_extended_card_to_nobody() {
    let server = guarded(None);
    let reply = server.call_with(
        "GET",
        "/extendedAgentCard",
        None,
        &[("A2A-Version", "1.0"), ("Authorization", "Bearer anything")],
    );
    assert_eq!(reply.status, 401, "{}", reply.body);
    assert_eq!(reply.rpc_status(), "UNAUTHENTICATED");
}

#[test]
fn a_bearer_token_is_one_b64token_compared_whole_and_never_printed() {
    for refused in ["", "  ", "two words", "=abc", "a,b", "tab\there"] {
        assert!(BearerToken::new(refused).is_none(), "{refused:?}");
    }
    for taken in ["abc", "abc==", " abc\n", "A-._~+/9"] {
        assert!(BearerToken::new(taken).is_some(), "{taken:?}");
    }
    let token = BearerToken::new(TOKEN).expect("a token");
    assert!(token.admits(TOKEN));
    for other in ["", "s3cret", "s3cret.Token-0~~", "S3cret.Token-0~"] {
        assert!(!token.admits(other), "{other:?}");
    }
    assert!(!format!("{token:?}").contains("s3cret"));
    assert!(
        !format!(
            "{:?}",
            Access {
                extended_card_token: Some(token)
            }
        )
        .contains("s3cret")
    );
}

// The binary's flags for the extended card's token.

fn written(name: &str, content: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, content).expect("write the file");
    path.to_str().expect("a UTF-8 path").to_owned()
}

fn card_file(name: &str, extended: bool) -> String {
    let mut card = a2a_server::default_card();
    card.capabilities.extended_agent_card = Some(extended);
    let mut text = String::new();
    a2a::server::wire::encode_a2a_discovery_agent_card(&card, &mut text);
    written(name, &text)
}

fn raw_get(address: &str, path: &str, authorization: Option<&str>) -> String {
    let mut stream = TcpStream::connect(address).expect("connect");
    let mut request = format!("GET {path} HTTP/1.1\r\nHost: {address}\r\nA2A-Version: 1.0\r\n");
    if let Some(authorization) = authorization {
        request.push_str(&format!("Authorization: {authorization}\r\n"));
    }
    request.push_str("\r\n");
    stream.write_all(request.as_bytes()).expect("write");
    let mut raw = String::new();
    stream.read_to_string(&mut raw).expect("read");
    raw
}

#[test]
fn the_binary_refuses_an_extended_card_without_a_token_file() {
    let extended = card_file("limits-refused-extended.json", true);
    let output = Command::new(env!("CARGO_BIN_EXE_a2a-server"))
        .args(["--listen", "127.0.0.1:0", "--extended-card", &extended])
        .output()
        .expect("run a2a-server");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("--extended-card-token-file"), "{stderr}");
}

#[test]
fn the_binary_refuses_a_token_file_without_one_token_and_does_not_print_it() {
    let token = written("limits-bad-token", "not-one secret-words\n");
    let output = Command::new(env!("CARGO_BIN_EXE_a2a-server"))
        .args([
            "--listen",
            "127.0.0.1:0",
            "--extended-card-token-file",
            &token,
        ])
        .output()
        .expect("run a2a-server");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains(&token), "{stderr}");
    assert!(!stderr.contains("secret-words"), "{stderr}");
}

#[test]
fn the_binary_serves_its_extended_card_for_the_token_in_its_file_and_never_prints_it() {
    let card = card_file("limits-served-card.json", true);
    let extended = card_file("limits-served-extended.json", true);
    let token = written("limits-served-token", &format!("  {TOKEN}\n"));
    let mut child = Command::new(env!("CARGO_BIN_EXE_a2a-server"))
        .args([
            "--listen",
            "127.0.0.1:0",
            "--card",
            &card,
            "--extended-card",
            &extended,
            "--extended-card-token-file",
            &token,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn a2a-server");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout"));
    let mut line = String::new();
    stdout.read_line(&mut line).expect("the first line");
    let address = line
        .trim()
        .strip_prefix("a2a-server listening on http://")
        .unwrap_or_else(|| panic!("not the listening line: {line:?}"))
        .to_owned();
    let without = raw_get(&address, "/extendedAgentCard", None);
    let wrong = raw_get(&address, "/extendedAgentCard", Some("Bearer s3cret"));
    let right = raw_get(
        &address,
        "/extendedAgentCard",
        Some(&format!("Bearer {TOKEN}")),
    );
    let _ = child.kill();
    let _ = child.wait();
    let mut printed = line.clone();
    let _ = stdout.read_to_string(&mut printed);
    let _ = child
        .stderr
        .take()
        .expect("stderr")
        .read_to_string(&mut printed);
    assert!(without.starts_with("HTTP/1.1 401"), "{without}");
    assert!(wrong.starts_with("HTTP/1.1 401"), "{wrong}");
    assert!(right.starts_with("HTTP/1.1 200"), "{right}");
    assert!(!printed.contains(TOKEN), "{printed}");
}
