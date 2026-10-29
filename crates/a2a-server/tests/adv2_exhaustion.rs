//! Adversarial cases, pass 2, against the connection handling that fix pass 1 added to
//! `a2a-server` (story:server, wave 3): the reader thread per connection (lib.rs:90-125) and the
//! pruning of streams whose client hung up (router.rs:401, :937). Each runs the binary under
//! `ulimit -n 64`, as tests/adv_availability.rs does, so that exhausting descriptors is cheap.

mod common;

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

struct Running(Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// The binary under `ulimit -n 64` and the address it listens on.
fn spawn_limited() -> (Running, String) {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("ulimit -n 64 && exec \"$0\" --listen 127.0.0.1:0")
        .arg(env!("CARGO_BIN_EXE_a2a-server"))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn a2a-server under ulimit -n 64");
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

/// The raw answer to an agent card request, `None` when none comes within 2 s.
fn card(address: &str) -> Option<String> {
    let mut stream = request(address, "GET", "/.well-known/agent-card.json", "")?;
    let mut raw = String::new();
    let _ = stream.read_to_string(&mut raw);
    (!raw.is_empty()).then_some(raw)
}

/// router.rs:937 `open` peeks one byte and counts a stream open while a byte is waiting. A client
/// that sends one byte after its `SubscribeToTask` request and hangs up leaves that byte ahead of
/// the end of file, so `prune` keeps its socket until an event for the task comes, which for a
/// task waiting for input may be never.
#[test]
fn adv2_subscribers_that_send_a_byte_and_hang_up_do_not_take_the_server_down() {
    let (mut running, address) = spawn_limited();
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
        if common::next_frame(&mut reader).is_none() {
            break;
        }
        opened += 1;
        let _ = reader.get_mut().write_all(b"x");
    }
    std::thread::sleep(Duration::from_millis(1000));

    let exited = running.0.try_wait().expect("try_wait");
    let card = card(&address);
    assert!(
        exited.is_none()
            && card
                .as_deref()
                .is_some_and(|raw| raw.starts_with("HTTP/1.1 200")),
        "after {opened} subscribers to a waiting task each read the first event, sent one byte \
         and hung up, and 1 s of pruning ticks, the server exited ({exited:?}) or no longer \
         answers the card ({card:?})"
    );
}

/// lib.rs:111-125 reads each request on a thread of its own, and generated http.rs
/// `READ_TIMEOUT` bounds each read, not the request (http.rs:40-47): a client that sends its
/// head one byte every 500 ms keeps its thread and its descriptor for as long as it likes. There
/// is no bound on the number of such connections.
#[test]
fn adv2_clients_sending_their_heads_slowly_do_not_lock_out_another_client() {
    let (_running, address) = spawn_limited();
    let mut slow = Vec::new();
    for _ in 0..70 {
        let Ok(mut stream) = TcpStream::connect(&address) else {
            break;
        };
        if stream
            .write_all(b"GET /.well-known/agent-card.json HTTP/1.1\r\nX-Slow: ")
            .is_err()
        {
            break;
        }
        slow.push(stream);
    }
    let held = slow.len();
    let trickle = std::thread::spawn(move || {
        for _ in 0..12 {
            std::thread::sleep(Duration::from_millis(500));
            for stream in &mut slow {
                let _ = stream.write_all(b"a");
            }
        }
        slow
    });
    std::thread::sleep(Duration::from_millis(1500));
    let card = card(&address);
    drop(trickle.join());
    assert!(
        card.as_deref()
            .is_some_and(|raw| raw.starts_with("HTTP/1.1 200")),
        "while {held} clients each sent their head one byte every 500 ms, another client got no \
         agent card within 2 s ({card:?})"
    );
}
