//! Adversarial cases against the availability of `a2a-server` (story:server, wave 3): one client
//! must not hold back or take down the others.

mod common;

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use common::echo;

/// lib.rs `serve()` answers one connection at a time; http.rs `READ_TIMEOUT` bounds each read,
/// not the request. A client sending its head one byte every 500 ms holds the loop.
#[test]
fn adv_a_slow_client_does_not_hold_back_another_client() {
    let server = echo();
    let address = server.address;
    let slow = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(address).expect("connect");
        stream
            .write_all(b"GET /.well-known/agent-card.json HTTP/1.1\r\nHost: slow\r\nX-Slow: ")
            .expect("write");
        for _ in 0..10 {
            std::thread::sleep(Duration::from_millis(500));
            if stream.write_all(b"a").is_err() {
                return;
            }
        }
        let _ = stream.write_all(b"\r\nContent-Length: 0\r\n\r\n");
        let mut rest = Vec::new();
        let _ = stream.read_to_end(&mut rest);
    });
    std::thread::sleep(Duration::from_millis(200));
    let started = Instant::now();
    let reply = server.call("GET", "/.well-known/agent-card.json", None);
    let waited = started.elapsed();
    let _ = slow.join();
    assert_eq!(reply.status, 200, "{}", reply.body);
    assert!(
        waited < Duration::from_secs(2),
        "a second client waited {waited:?} for the agent card behind a client sending one byte \
         every 500 ms"
    );
}

struct Running(Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn request(address: &str, method: &str, path: &str, body: &str) -> Option<TcpStream> {
    let mut stream = TcpStream::connect(address).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    let mut head = format!("{method} {path} HTTP/1.1\r\nHost: {address}\r\nA2A-Version: 1.0\r\n");
    if !body.is_empty() {
        head.push_str("Content-Type: application/a2a+json\r\n");
    }
    head.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
    stream.write_all(head.as_bytes()).ok()?;
    stream.write_all(body.as_bytes()).ok()?;
    Some(stream)
}

/// router.rs keeps a `SubscribeToTask` stream on a task waiting for input as a follower until an
/// event for that task fails to write; lib.rs `serve()` returns at the first accept error
/// (`connection?`), and main.rs exits with it.
#[test]
fn adv_subscribers_that_hang_up_do_not_take_the_server_down() {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("ulimit -n 64 && exec \"$0\" --listen 127.0.0.1:0")
        .arg(env!("CARGO_BIN_EXE_a2a-server"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn a2a-server under ulimit -n 64");
    let stdout = child.stdout.take().expect("stdout");
    let mut stderr = child.stderr.take().expect("stderr");
    let mut running = Running(child);
    let mut line = String::new();
    BufReader::new(stdout)
        .read_line(&mut line)
        .expect("the listening line");
    let address = line
        .trim()
        .strip_prefix("a2a-server listening on http://")
        .unwrap_or_else(|| panic!("not the listening line: {line:?}"))
        .to_owned();

    let mut created = String::new();
    let _ = request(
        &address,
        "POST",
        "/message:send",
        &common::message("", None, None),
    )
    .expect("a request to the fresh server")
    .read_to_string(&mut created);
    let body = created.split_once("\r\n\r\n").map_or("", |(_, body)| body);
    let task = common::text(
        &a2a::json::parse(body).unwrap_or_else(|error| panic!("{error}: {created}")),
        &["task", "id"],
    );
    assert!(!task.is_empty(), "no waiting task: {created}");

    let mut opened = 0;
    for _ in 0..80 {
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
        if common::next_frame(&mut reader).is_some() {
            opened += 1;
        }
    }
    std::thread::sleep(Duration::from_millis(500));
    eprintln!("subscribe streams opened and hung up: {opened}");

    let exited = running.0.try_wait().expect("try_wait");
    let mut said = String::new();
    if exited.is_some() {
        let _ = stderr.read_to_string(&mut said);
    }
    let card = TcpStream::connect(&address).ok().and_then(|stream| {
        let mut stream = stream;
        stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
        stream
            .write_all(
                format!("GET /.well-known/agent-card.json HTTP/1.1\r\nHost: {address}\r\n\r\n")
                    .as_bytes(),
            )
            .ok()?;
        let mut raw = String::new();
        let _ = stream.read_to_string(&mut raw);
        Some(raw)
    });
    assert!(
        exited.is_none()
            && card
                .as_deref()
                .is_some_and(|raw| raw.starts_with("HTTP/1.1 200")),
        "after {opened} subscribers to a waiting task hung up, the server exited ({exited:?}, \
         stderr {said:?}) or no longer answers the card ({card:?})"
    );
}
