//! Security review of story:server (wave 3): the bounds on what one client can hold of the
//! server. Each case runs the binary with its default limits (256 connections, 10 s per request)
//! and measures whether another client is still served.

mod common;

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Running(Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// The binary with its default limits on a free port, and the address it listens on.
fn spawn() -> (Running, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_a2a-server"))
        .args(["--listen", "127.0.0.1:0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn a2a-server");
    let stdout = child.stdout.take().expect("stdout");
    let running = Running(child);
    let mut line = String::new();
    BufReader::new(stdout)
        .read_line(&mut line)
        .expect("the listening line");
    let address = line
        .trim()
        .strip_prefix("a2a-server listening on http://")
        .unwrap_or_else(|| panic!("not the listening line: {line:?}"))
        .to_owned();
    (running, address)
}

fn request(address: &str, method: &str, path: &str, body: &str) -> Option<TcpStream> {
    let mut stream = TcpStream::connect(address).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    let mut head = format!("{method} {path} HTTP/1.1\r\nHost: {address}\r\nA2A-Version: 1.0\r\n");
    if !body.is_empty() {
        head.push_str("Content-Type: application/a2a+json\r\n");
    }
    head.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
    stream.write_all(head.as_bytes()).ok()?;
    stream.write_all(body.as_bytes()).ok()?;
    Some(stream)
}

/// The whole raw answer, `None` where the connection closes without one.
fn call(address: &str, method: &str, path: &str, body: &str) -> Option<String> {
    let mut stream = request(address, method, path, body)?;
    let mut raw = Vec::new();
    let _ = stream.read_to_end(&mut raw);
    (!raw.is_empty()).then(|| String::from_utf8_lossy(&raw).into_owned())
}

fn card(address: &str) -> Option<String> {
    call(address, "GET", "/.well-known/agent-card.json", "")
}

fn body_of(raw: &str) -> &str {
    raw.split_once("\r\n\r\n").map_or("", |(_, body)| body)
}

/// The resident memory of a process, in KiB.
fn resident_kib(pid: u32) -> u64 {
    std::fs::read_to_string(format!("/proc/{pid}/status"))
        .unwrap_or_default()
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))
        .and_then(|value| value.trim().trim_end_matches("kB").trim().parse().ok())
        .unwrap_or(0)
}

/// connections.rs:343 `listen` reads what a client sends after its request until the client
/// hangs up, and returns on a read timeout only once the answer is written. A client that keeps
/// sending a byte more often than `http::READ_TIMEOUT` (1 s) keeps its reader thread, and with it
/// its seat, for good. Its seat no longer counts as reading (connections.rs:265 `taken`), so
/// `admit` (connections.rs:205) cannot close it to seat another and refuses every new connection.
#[test]
fn sec_clients_that_keep_sending_after_their_answer_do_not_lock_out_another_client() {
    let (_running, address) = spawn();
    let held = Arc::new(Mutex::new(Vec::<TcpStream>::new()));
    let stop = Arc::new(AtomicBool::new(false));
    let trickle = {
        let (held, stop) = (Arc::clone(&held), Arc::clone(&stop));
        std::thread::spawn(move || {
            while !stop.load(Ordering::Acquire) {
                std::thread::sleep(Duration::from_millis(300));
                for stream in held.lock().expect("held").iter_mut() {
                    let _ = stream.write_all(b"x");
                }
            }
        })
    };
    for _ in 0..300 {
        let Some(mut stream) = request(&address, "GET", "/.well-known/agent-card.json", "") else {
            break;
        };
        let mut raw = Vec::new();
        let _ = stream.read_to_end(&mut raw);
        if !raw.starts_with(b"HTTP/1.1 200") {
            break;
        }
        let _ = stream.write_all(b"x");
        held.lock().expect("held").push(stream);
    }
    let answered = held.lock().expect("held").len();
    std::thread::sleep(Duration::from_millis(1500));
    let card = card(&address);
    stop.store(true, Ordering::Release);
    trickle.join().expect("the trickle");
    assert!(
        card.as_deref()
            .is_some_and(|raw| raw.starts_with("HTTP/1.1 200")),
        "{answered} clients each got the agent card and then sent one byte every 300 ms; 1.5 s \
         later another client got no agent card ({card:?})"
    );
}

/// connections.rs:205 `admit`: at the cap, only a connection still sending its request can be
/// closed to seat another. A subscriber holds its seat until its task ends, and nothing bounds
/// the subscribers of one task, so subscribers to one task waiting for input hold every seat.
#[test]
fn sec_subscribers_to_one_task_do_not_lock_out_another_client() {
    let (_running, address) = spawn();
    let created = call(
        &address,
        "POST",
        "/message:send",
        &common::message("", None, None),
    )
    .expect("an answer from the fresh server");
    let task = common::text(
        &a2a::json::parse(body_of(&created)).unwrap_or_else(|error| panic!("{error}: {created}")),
        &["task", "id"],
    );
    assert!(!task.is_empty(), "no waiting task: {created}");
    let mut followers = Vec::new();
    for _ in 0..300 {
        let Some(stream) = request(&address, "GET", &format!("/tasks/{task}:subscribe"), "") else {
            break;
        };
        let mut reader = BufReader::new(stream);
        let mut status = String::new();
        if reader.read_line(&mut status).is_err() || !status.starts_with("HTTP/1.1 200") {
            break;
        }
        let mut line = String::new();
        while reader.read_line(&mut line).is_ok_and(|read| read > 0) && !line.trim().is_empty() {
            line.clear();
        }
        if common::next_frame(&mut reader).is_none() {
            break;
        }
        followers.push(reader);
    }
    let held = followers.len();
    let card = card(&address);
    drop(followers);
    assert!(
        card.as_deref()
            .is_some_and(|raw| raw.starts_with("HTTP/1.1 200")),
        "{held} clients subscribed to one task waiting for input and read its first event; \
         another client then got no agent card ({card:?})"
    );
}

/// AGENTS.md: "a request not in within the timeout (seconds) is answered 408". A client sending
/// its head one byte every 250 ms, inside the 1 s `http::READ_TIMEOUT` of each read, is answered
/// 408 once `--request-timeout` has passed (connections.rs:246 `watch`, router.rs:338).
#[test]
fn sec_a_request_head_still_arriving_after_the_timeout_is_answered_408() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_a2a-server"))
        .args(["--listen", "127.0.0.1:0", "--request-timeout", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn a2a-server");
    let stdout = child.stdout.take().expect("stdout");
    let _running = Running(child);
    let mut line = String::new();
    BufReader::new(stdout)
        .read_line(&mut line)
        .expect("the listening line");
    let address = line
        .trim()
        .strip_prefix("a2a-server listening on http://")
        .unwrap_or_else(|| panic!("not the listening line: {line:?}"))
        .to_owned();
    let mut stream = TcpStream::connect(&address).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_millis(250)))
        .expect("read timeout");
    stream
        .write_all(b"GET /.well-known/agent-card.json HTTP/1.1\r\nX-Slow: ")
        .expect("the start of the head");
    let started = Instant::now();
    let mut answer = Vec::new();
    let mut chunk = [0_u8; 1024];
    while started.elapsed() < Duration::from_secs(4) {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => {
                answer.extend_from_slice(&chunk[..read]);
                if answer.windows(2).any(|pair| pair == b"\r\n") {
                    break;
                }
            }
            Err(_) => {}
        }
        if answer.is_empty() {
            let _ = stream.write_all(b"a");
        }
    }
    let waited = started.elapsed();
    let status = String::from_utf8_lossy(&answer)
        .lines()
        .next()
        .unwrap_or("")
        .to_owned();
    assert!(
        status.starts_with("HTTP/1.1 408") && waited < Duration::from_secs(4),
        "a client sending its head one byte every 250 ms under `--request-timeout 1` got \
         {status:?} after {waited:?}"
    );
}

/// Nothing bounds the tasks the server keeps, nor deletes one; `ListTasks` builds the generated
/// `Tasks` view of every task of the tenant, whole, then parses, decodes and sorts it on the one
/// thread that answers every request (router.rs:828-964), whatever `pageSize` asks for. A client
/// listing one task holds back every other client for as long as that takes.
#[test]
fn sec_listing_one_task_does_not_hold_back_another_client() {
    let (running, address) = spawn();
    let pid = running.0.id();
    let before = resident_kib(pid);
    let text = "a".repeat(1_000_000);
    let mut stored = 0;
    for _ in 0..48 {
        let answer = call(
            &address,
            "POST",
            "/message:send",
            &common::message(&text, None, None),
        );
        if answer
            .as_deref()
            .is_some_and(|raw| raw.starts_with("HTTP/1.1 200"))
        {
            stored += 1;
        }
    }
    let after = resident_kib(pid);
    let lister = {
        let address = address.clone();
        std::thread::spawn(move || {
            let started = Instant::now();
            let answer = call(&address, "GET", "/tasks?pageSize=1", "");
            (
                started.elapsed(),
                answer.map(|raw| raw.chars().take(15).collect::<String>()),
            )
        })
    };
    std::thread::sleep(Duration::from_millis(100));
    let started = Instant::now();
    let card = card(&address);
    let waited = started.elapsed();
    let (listed_in, listed) = lister.join().expect("the lister");
    assert!(
        card.as_deref()
            .is_some_and(|raw| raw.starts_with("HTTP/1.1 200"))
            && waited < Duration::from_millis(1000),
        "after {stored} tasks of a 1 MB message each, resident memory {before} KiB -> {after} \
         KiB; `GET /tasks?pageSize=1` answered {listed:?} in {listed_in:?}, and another client \
         sent 100 ms later waited {waited:?} for the agent card ({:?})",
        card.map(|raw| raw.chars().take(15).collect::<String>())
    );
}

/// Nothing bounds one task's history, nor its push-notification configs: a task waiting for
/// input takes every message without content and stays waiting, and every config with a fresh
/// id is kept (ports.rs:362 `record`, :890 `store_config`). `GetTask` encodes the whole history
/// in the generated envelope and parses it again before `historyLength` cuts it (router.rs:681,
/// ports.rs:460), on the thread that answers every request.
#[test]
fn sec_reading_one_long_task_does_not_hold_back_another_client() {
    let (running, address) = spawn();
    let pid = running.0.id();
    let answer = call(
        &address,
        "POST",
        "/message:send",
        &common::message("", None, None),
    )
    .expect("an answer from the fresh server");
    let task = common::text(
        &a2a::json::parse(body_of(&answer)).unwrap_or_else(|error| panic!("{error}: {answer}")),
        &["task", "id"],
    );
    let before = resident_kib(pid);
    let pad = "p".repeat(1_000_000);
    let mut kept = 0;
    for _ in 0..40 {
        let body = format!(
            "{{\"message\":{{\"messageId\":{},\"role\":\"ROLE_USER\",\"taskId\":{},\"parts\":[{{\"text\":\"\"}}],\"metadata\":{{\"pad\":{}}}}},\"configuration\":{{\"historyLength\":0}}}}",
            common::quoted(&uuid::Uuid::new_v4().to_string()),
            common::quoted(&task),
            common::quoted(&pad)
        );
        if call(&address, "POST", "/message:send", &body)
            .as_deref()
            .is_some_and(|raw| raw.starts_with("HTTP/1.1 200"))
        {
            kept += 1;
        }
    }
    let after_history = resident_kib(pid);
    let url = format!("https://example.invalid/{}", "u".repeat(4000));
    let mut configs = 0;
    for at in 0..2000 {
        let body = format!("{{\"id\":\"c-{at}\",\"url\":{}}}", common::quoted(&url));
        if call(
            &address,
            "POST",
            &format!("/tasks/{task}/pushNotificationConfigs"),
            &body,
        )
        .as_deref()
        .is_some_and(|raw| raw.starts_with("HTTP/1.1 200"))
        {
            configs += 1;
        }
    }
    let after_configs = resident_kib(pid);
    let reader = {
        let (address, task) = (address.clone(), task.clone());
        std::thread::spawn(move || {
            let started = Instant::now();
            let answer = call(
                &address,
                "GET",
                &format!("/tasks/{task}?historyLength=1"),
                "",
            );
            (
                started.elapsed(),
                answer.map(|raw| raw.chars().take(15).collect::<String>()),
            )
        })
    };
    std::thread::sleep(Duration::from_millis(100));
    let started = Instant::now();
    let card = card(&address);
    let waited = started.elapsed();
    let (read_in, read) = reader.join().expect("the reader");
    let measured = format!(
        "one task took {kept} of 40 messages of 1 MB metadata and stayed waiting, resident \
         memory {before} KiB -> {after_history} KiB; it took {configs} of 2000 configs with a \
         4 KB url, -> {after_configs} KiB; `GET /tasks/<id>?historyLength=1` answered {read:?} \
         in {read_in:?}, and another client sent 100 ms later waited {waited:?} for the agent \
         card ({:?})",
        card.as_deref()
            .map(|raw| raw.chars().take(15).collect::<String>())
    );
    eprintln!("{measured}");
    assert!(
        card.as_deref()
            .is_some_and(|raw| raw.starts_with("HTTP/1.1 200"))
            && waited < Duration::from_millis(1000),
        "{measured}"
    );
}
