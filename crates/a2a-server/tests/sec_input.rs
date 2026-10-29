//! Security review of story:server (wave 3): what the server does with request data it did not
//! expect: nesting, sizes, encoded line breaks, and malformed requests of every kind.

mod common;

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use common::{echo, quoted};

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

/// Writes `bytes` as they are and answers what comes back, `None` where nothing does.
fn raw(address: &str, bytes: &[u8]) -> Option<String> {
    let mut stream = TcpStream::connect(address).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    let _ = stream.write_all(bytes);
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let mut answer = Vec::new();
    let _ = stream.read_to_end(&mut answer);
    (!answer.is_empty()).then(|| String::from_utf8_lossy(&answer).into_owned())
}

fn status_line(answer: Option<&str>) -> String {
    answer
        .and_then(|raw| raw.lines().next())
        .unwrap_or("<no answer>")
        .to_owned()
}

fn card_answers(address: &str) -> bool {
    raw(
        address,
        format!("GET /.well-known/agent-card.json HTTP/1.1\r\nHost: {address}\r\n\r\n").as_bytes(),
    )
    .is_some_and(|answer| answer.starts_with("HTTP/1.1 200"))
}

/// A `SendMessageRequest` continuing `task` with no content and metadata nested `depth` arrays
/// deep under `metadata.k`.
fn nested_message(task: &str, depth: usize) -> String {
    format!(
        "{{\"message\":{{\"messageId\":{},\"role\":\"ROLE_USER\",\"taskId\":{},\"parts\":[{{\"text\":\"\"}}],\"metadata\":{{\"k\":{}{}}}}}}}",
        quoted(&uuid::Uuid::new_v4().to_string()),
        quoted(task),
        "[".repeat(depth),
        "]".repeat(depth)
    )
}

/// The generated parser refuses a document nested deeper than 64 (json.rs:19, :118), and every
/// answer about a task is parsed again from the generated layer's envelope (router.rs:681
/// `reply`, :913 `list`), which wraps the stored message several levels deeper than the request
/// did. A message the server took must leave its task readable.
#[test]
fn sec_a_message_nested_within_the_parse_limit_leaves_its_task_readable() {
    let server = echo();
    let (task, _) = server.waiting_task();
    let mut taken = None;
    for depth in (1..=70).rev() {
        let reply = server.call("POST", "/message:send", Some(&nested_message(&task, depth)));
        if reply.status != 400 {
            taken = Some((depth, reply.status, reply.body));
            break;
        }
    }
    let (depth, sent, body) = taken.expect("some nesting is taken");
    let read = server.call("GET", &format!("/tasks/{task}"), None);
    let listed = server.call("GET", "/tasks", None);
    assert!(
        sent == 200 && read.status == 200 && listed.status == 200,
        "the deepest metadata SendMessage took was {depth} arrays under `metadata.k`; it answered \
         {sent} ({body}); then GetTask answered {} ({}) and ListTasks {} ({})",
        read.status,
        read.body,
        listed.status,
        listed.body
    );
}

/// A document nested far past the parser's limit is refused, and the server goes on answering.
#[test]
fn sec_a_body_nested_past_the_limit_is_refused_and_the_server_survives() {
    let (mut running, address) = spawn();
    let body = format!(
        "{{\"message\":{}{}}}",
        "[".repeat(300_000),
        "]".repeat(300_000)
    );
    let request = format!(
        "POST /message:send HTTP/1.1\r\nHost: {address}\r\nA2A-Version: 1.0\r\nContent-Type: application/a2a+json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let answer = raw(&address, request.as_bytes());
    std::thread::sleep(Duration::from_millis(300));
    let exited = running.0.try_wait().expect("try_wait");
    assert!(
        status_line(answer.as_deref()).starts_with("HTTP/1.1 400")
            && exited.is_none()
            && card_answers(&address),
        "a body nested 300000 deep answered {:?}; the server exited: {exited:?}",
        status_line(answer.as_deref())
    );
}

/// generated http.rs:24 `MAX_BODY` (1 MiB, 413) and :38 `MAX_HEAD` (1 MiB + 4096, 431).
#[test]
fn sec_a_body_or_head_over_its_limit_is_refused() {
    let server = echo();
    let address = server.address.to_string();
    let over_body = format!(
        "POST /message:send HTTP/1.1\r\nHost: {address}\r\nA2A-Version: 1.0\r\nContent-Type: application/a2a+json\r\nContent-Length: 1048577\r\n\r\n{}",
        " ".repeat(1_048_577)
    );
    let huge_length = format!(
        "POST /message:send HTTP/1.1\r\nHost: {address}\r\nContent-Length: 18446744073709551615\r\n\r\n"
    );
    let over_head = format!(
        "GET /.well-known/agent-card.json HTTP/1.1\r\nHost: {address}\r\nX-Pad: {}\r\n\r\n",
        "a".repeat(1_052_673)
    );
    let many_headers = format!(
        "GET /.well-known/agent-card.json HTTP/1.1\r\nHost: {address}\r\n{}\r\n",
        (0..101)
            .map(|at| format!("X-{at}: a\r\n"))
            .collect::<String>()
    );
    let chunked = format!(
        "POST /message:send HTTP/1.1\r\nHost: {address}\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n"
    );
    let answers = [
        status_line(raw(&address, over_body.as_bytes()).as_deref()),
        status_line(raw(&address, huge_length.as_bytes()).as_deref()),
        status_line(raw(&address, over_head.as_bytes()).as_deref()),
        status_line(raw(&address, many_headers.as_bytes()).as_deref()),
        status_line(raw(&address, chunked.as_bytes()).as_deref()),
    ];
    assert!(
        answers[0].starts_with("HTTP/1.1 413")
            && answers[1].starts_with("HTTP/1.1 413")
            && answers[2].starts_with("HTTP/1.1 431")
            && answers[3].starts_with("HTTP/1.1 431")
            && answers[4].starts_with("HTTP/1.1 411"),
        "a body of 1 MiB + 1, a Content-Length of 2^64 - 1, a head over 1 MiB + 4096, 101 \
         headers and a chunked body answered {answers:?}"
    );
}

/// No answer carries a header built from request data: an id, a tenant or a config id holding an
/// encoded CR LF adds no header to any answer.
#[test]
fn sec_encoded_line_breaks_in_ids_and_tenants_add_no_header() {
    let server = echo();
    let address = server.address.to_string();
    let crlf = "%0D%0AX-Injected:%20yes";
    let paths = [
        ("GET", format!("/tasks/a{crlf}")),
        ("DELETE", format!("/tasks/a{crlf}")),
        ("GET", format!("/t{crlf}/tasks")),
        ("PUT", format!("/t{crlf}/tasks")),
        ("POST", format!("/tasks/a{crlf}:cancel")),
        ("GET", format!("/tasks/a{crlf}:subscribe")),
        ("GET", format!("/t{crlf}/tasks/a{crlf}:subscribe")),
        ("GET", format!("/tasks/a/pushNotificationConfigs/c{crlf}")),
        ("GET", format!("/no{crlf}/such/route")),
        ("GET", format!("/tasks?pageToken=t{crlf}")),
    ];
    let mut injected = Vec::new();
    for (method, path) in &paths {
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {address}\r\nA2A-Version: 1.0\r\nContent-Length: 0\r\n\r\n"
        );
        let answer = raw(&address, request.as_bytes()).unwrap_or_default();
        let head = answer
            .split_once("\r\n\r\n")
            .map_or(answer.as_str(), |(head, _)| head);
        if head
            .lines()
            .any(|line| line.to_ascii_lowercase().starts_with("x-injected"))
        {
            injected.push(format!("{method} {path}: {head:?}"));
        }
    }
    let message = "{\"message\":{\"messageId\":\"m\\r\\nX-Injected: yes\",\"role\":\"ROLE_USER\",\"parts\":[{\"text\":\"\"}],\"metadata\":{\"mode\":\"message\"}}}".to_owned();
    let sent = server.call(
        "POST",
        "/t%0D%0AX-Injected:%20yes/message:send",
        Some(&message),
    );
    if sent.header("X-Injected").is_some() {
        injected.push(format!("SendMessage: {:?}", sent.headers));
    }
    assert!(
        injected.is_empty(),
        "answers carrying an injected header: {injected:#?}"
    );
}

/// Malformed requests of every kind are answered, and none ends the server: a panic on the
/// thread that answers requests ends the process (main.rs runs `serve_with` on it).
#[test]
fn sec_malformed_requests_do_not_end_the_server() {
    let (mut running, address) = spawn();
    let get =
        |target: &str| format!("GET {target} HTTP/1.1\r\nHost: x\r\nA2A-Version: 1.0\r\n\r\n");
    let post = |target: &str, body: &str| {
        format!(
            "POST {target} HTTP/1.1\r\nHost: x\r\nA2A-Version: 1.0\r\nContent-Type: application/a2a+json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
    };
    let requests = vec![
        get("/tasks?pageSize=99999999999999999999999"),
        get("/tasks?pageSize=0"),
        get("/tasks?historyLength=-1"),
        get("/tasks?historyLength=9223372036854775808"),
        get("/tasks?historyLength=9223372036854775807"),
        get("/tasks?pageToken=t"),
        get("/tasks?pageToken=t0"),
        get("/tasks?pageToken=%FF"),
        get("/tasks?pageToken=t%"),
        get("/tasks?pageToken=t0a"),
        get("/tasks?pageToken=t%E2%82%AC"),
        get("/tasks?pageToken=t0a0a"),
        get("/tasks?pageToken=c00"),
        get("/tasks?statusTimestampAfter=9999-99-99T99:99:99Z"),
        get("/tasks?statusTimestampAfter=-999999-01-01T00:00:00Z"),
        get("/tasks?status=%00"),
        get("/tasks/%"),
        get("/tasks/%FF"),
        get("/%FF/tasks"),
        get("/%C0%80/tasks"),
        get("/tasks/x:subscribe?historyLength=-5"),
        get("/tasks/x/pushNotificationConfigs?pageSize=-1&pageToken=zz"),
        get("/tasks/x/pushNotificationConfigs?pageSize=99999999999999999999999"),
        get("/tasks/:subscribe"),
        get("/tasks/:cancel"),
        get("//tasks"),
        get("/a/b/c/d/e/f/g/h"),
        get("*"),
        get(""),
        post("/message:send", ""),
        post("/message:send", "null"),
        post("/message:send", "[]"),
        post("/message:send", "\"x\""),
        post("/message:send", "{\"message\":null}"),
        post("/message:send", "{\"message\":{}}"),
        post(
            "/message:send",
            "{\"message\":{\"messageId\":\"m\",\"role\":\"ROLE_USER\",\"parts\":[{\"raw\":\"!!\"}]}}",
        ),
        post(
            "/message:send",
            "{\"message\":{\"messageId\":\"m\",\"role\":\"ROLE_USER\",\"parts\":[]}}",
        ),
        post(
            "/message:send",
            "{\"message\":{\"messageId\":\"m\",\"role\":\"ROLE_USER\",\"parts\":[{\"text\":\"\"}]},\"configuration\":{\"historyLength\":-9223372036854775808}}",
        ),
        post(
            "/message:send",
            "{\"message\":{\"messageId\":\"m\",\"role\":\"ROLE_USER\",\"parts\":[{\"text\":\"\"}]},\"configuration\":{\"historyLength\":99999999999999999999999}}",
        ),
        post(
            "/message:send",
            "{\"message\":{\"messageId\":\"m\",\"role\":\"ROLE_USER\",\"parts\":[{\"text\":\"\"}],\"taskId\":\"\"}}",
        ),
        post(
            "/message:stream",
            "{\"message\":{\"messageId\":\"m\",\"role\":\"ROLE_USER\",\"parts\":[{\"text\":\"\\ud800\"}]}}",
        ),
        post("/tasks/x:cancel", "{\"id\":\"y\"}"),
        post("/tasks/x/pushNotificationConfigs", "{\"url\":\"\"}"),
        post(
            "/tasks/x/pushNotificationConfigs",
            "{\"taskId\":\"y\",\"url\":\"u\"}",
        ),
        "GET /tasks HTTP/1.1\r\nHost: x\r\nA2A-Version: 99999999999999999999999.0\r\n\r\n"
            .to_owned(),
        "GET /tasks HTTP/1.1\r\nHost: x\r\nA2A-Version: \r\nA2A-Extensions: ,,,\r\n\r\n".to_owned(),
        "GET /tasks HTTP/1.1\r\nHost: x\r\nAccept: text/event-stream;q=99999999999999999\r\n\r\n"
            .to_owned(),
        "POST /message:send HTTP/1.1\r\nContent-Length: -1\r\n\r\n".to_owned(),
        "POST /message:send HTTP/1.1\r\nContent-Length: 1\r\nContent-Length: 2\r\n\r\nab"
            .to_owned(),
        "POST /message:send HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n".to_owned(),
        "GET /tasks\r\n\r\n".to_owned(),
        "GET /tasks HTTP/9.9\r\n\r\n".to_owned(),
        "get /tasks HTTP/1.1\r\n\r\n".to_owned(),
        "GET /tasks HTTP/1.1\r\nNo-Colon\r\n\r\n".to_owned(),
        "GET /tasks HTTP/1.1\r\n: empty\r\n\r\n".to_owned(),
        "\r\n\r\n".to_owned(),
        "GET /tasks HTTP/1.1\n\n".to_owned(),
    ];
    let mut answers = Vec::new();
    for request in &requests {
        let answer = raw(&address, request.as_bytes());
        answers.push((
            request.lines().next().unwrap_or("").to_owned(),
            status_line(answer.as_deref()),
        ));
        if running.0.try_wait().expect("try_wait").is_some() {
            break;
        }
    }
    let mut bad_bytes = b"POST /message:send HTTP/1.1\r\nContent-Length: 4\r\n\r\n".to_vec();
    bad_bytes.extend_from_slice(&[0xFF, 0xFE, 0x00, 0x7B]);
    answers.push((
        "invalid UTF-8 body".to_owned(),
        status_line(raw(&address, &bad_bytes).as_deref()),
    ));
    std::thread::sleep(Duration::from_millis(300));
    let exited = running.0.try_wait().expect("try_wait");
    assert!(
        exited.is_none() && card_answers(&address),
        "the server exited ({exited:?}) or stopped answering the agent card after: {answers:#?}"
    );
}
