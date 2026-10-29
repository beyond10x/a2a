//! The `a2a-server` binary: it binds `--listen`, says where, and serves the built-in card or the
//! one `--card` names.

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};

struct Running(Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn start(arguments: &[&str]) -> (Running, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_a2a-server"))
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn a2a-server");
    let stdout = child.stdout.take().expect("stdout");
    let running = Running(child);
    let mut line = String::new();
    BufReader::new(stdout)
        .read_line(&mut line)
        .expect("the first line");
    let address = line
        .trim()
        .strip_prefix("a2a-server listening on http://")
        .unwrap_or_else(|| panic!("not the listening line: {line:?}"))
        .to_owned();
    (running, address)
}

fn card_at(address: &str) -> String {
    let mut stream = TcpStream::connect(address).expect("connect");
    stream
        .write_all(
            format!("GET /.well-known/agent-card.json HTTP/1.1\r\nHost: {address}\r\n\r\n")
                .as_bytes(),
        )
        .expect("write");
    let mut raw = String::new();
    stream.read_to_string(&mut raw).expect("read");
    assert!(raw.starts_with("HTTP/1.1 200"), "{raw}");
    raw.split_once("\r\n\r\n").expect("a body").1.to_owned()
}

#[test]
fn the_binary_serves_the_built_in_card_at_the_address_it_bound() {
    let (_server, address) = start(&["--listen", "127.0.0.1:0"]);
    let card = a2a_server::read_card(&card_at(&address)).expect("an AgentCard");
    assert_eq!(card.name, a2a_server::default_card().name);
    assert_eq!(
        card.supported_interfaces[0].url,
        format!("http://{address}")
    );
}

#[test]
fn the_binary_serves_the_card_it_is_given() {
    let mut card = a2a_server::default_card();
    card.name = "a configured agent".to_owned();
    let mut text = String::new();
    a2a::server::wire::encode_a2a_discovery_agent_card(&card, &mut text);
    let path =
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("a2a-server-binary-card.json");
    std::fs::write(&path, text).expect("write the card");
    let (_server, address) = start(&[
        "--listen",
        "127.0.0.1:0",
        "--card",
        path.to_str().expect("a UTF-8 path"),
    ]);
    let served = a2a_server::read_card(&card_at(&address)).expect("an AgentCard");
    assert_eq!(served, card);
}

#[test]
fn the_binary_refuses_a_card_that_is_not_an_agent_card() {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("a2a-server-not-a-card.json");
    std::fs::write(&path, "{\"name\":7}").expect("write");
    let output = Command::new(env!("CARGO_BIN_EXE_a2a-server"))
        .args([
            "--listen",
            "127.0.0.1:0",
            "--card",
            path.to_str().expect("a UTF-8 path"),
        ])
        .output()
        .expect("run a2a-server");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("a2a-server-not-a-card.json"), "{stderr}");
}
