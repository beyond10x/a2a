//! Adversary support: a suite synthesized once per test binary, a proxy that rewrites the answer
//! to chosen requests the upstream answered `404`, and a stub endpoint that answers each request
//! from a function of its method and target.

#![allow(dead_code)]

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::{Arc, OnceLock};

use crate::common::{Endpoint, printed, results, run, scratch, synthesize};
use a2a_conformance::{Results, Status};

/// The suite, synthesized with the `ess` on `PATH` once for this test binary.
pub fn suite(binary: &str) -> PathBuf {
    static SUITE: OnceLock<PathBuf> = OnceLock::new();
    SUITE
        .get_or_init(|| synthesize(&scratch(&format!("adv-suite-{binary}"))))
        .clone()
}

/// Runs the CLI against `url` with `suite`, the results written under a scratch directory named
/// `name`, and answers the output and the results.
pub fn run_against(url: &str, suite: &Path, name: &str) -> (Output, Results) {
    let directory = scratch(name);
    let written = directory.join("results.json");
    let output = run(url, suite, &written);
    let read = results(&written);
    (output, read)
}

/// The status line of `scenario` the run printed.
pub fn line<'a>(output: &'a str, scenario: &str) -> Option<&'a str> {
    output.lines().find(|line| {
        line.split_once(' ')
            .is_some_and(|(_, rest)| rest == scenario || rest.starts_with(&format!("{scenario}:")))
    })
}

/// The summary: the last line the run printed.
pub fn counts(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .last()
        .unwrap_or_default()
        .to_owned()
}

/// Asserts the run reported `scenario` failed and exited 1.
pub fn assert_caught(output: &Output, results: &Results, scenario: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        results.status(scenario),
        Some(Status::Failed),
        "the run did not report {scenario} failed: {:?}\n{}",
        line(&stdout, scenario),
        counts(output)
    );
    assert_eq!(output.status.code(), Some(1), "{}", printed(output));
}

/// Asserts the run reported `scenario` failed or in error, and exited 1: the defect is caught,
/// whichever of the two it is filed under.
pub fn assert_rejected(output: &Output, results: &Results, scenario: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        matches!(
            results.status(scenario),
            Some(Status::Failed | Status::Error)
        ),
        "the run did not reject {scenario}: {:?}\n{}",
        line(&stdout, scenario),
        counts(output)
    );
    assert_eq!(output.status.code(), Some(1), "{}", printed(output));
}

/// One request read off `stream`: its head, and its body by `Content-Length`.
pub fn read_request(stream: &mut TcpStream) -> Option<(String, Vec<u8>)> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut head = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let end = line == "\r\n";
        head.push_str(&line);
        if end {
            break;
        }
    }
    let length = head
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(0);
    let mut body = vec![0; length];
    reader.read_exact(&mut body).ok()?;
    Some((head, body))
}

fn request_line(head: &str) -> (String, String) {
    let mut words = head.lines().next().unwrap_or_default().split(' ');
    let method = words.next().unwrap_or_default().to_owned();
    let target = words.next().unwrap_or_default().to_owned();
    (method, target)
}

/// Which requests a mutant rewrites, by method and target.
pub type Matches = fn(&str, &str) -> bool;

/// A proxy in front of `upstream`: a request `matches` selects, which the upstream answered
/// `404`, is answered `rewritten` instead. Every other exchange passes as it is.
pub fn mutant(upstream: SocketAddr, matches: Matches, rewritten: String) -> Endpoint {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = listener.local_addr().expect("bound address");
    let rewritten = Arc::new(rewritten);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let rewritten = Arc::clone(&rewritten);
            std::thread::spawn(move || relay(stream, upstream, matches, &rewritten));
        }
    });
    Endpoint { address }
}

fn relay(mut client: TcpStream, upstream: SocketAddr, matches: Matches, rewritten: &str) {
    let Some((head, body)) = read_request(&mut client) else {
        return;
    };
    let Ok(mut server) = TcpStream::connect(upstream) else {
        return;
    };
    if server.write_all(head.as_bytes()).is_err() || server.write_all(&body).is_err() {
        return;
    }
    let (method, target) = request_line(&head);
    if !matches(&method, &target) {
        let _ = std::io::copy(&mut server, &mut client);
        return;
    }
    let mut answered = Vec::new();
    let _ = server.read_to_end(&mut answered);
    let written = if answered.starts_with(b"HTTP/1.1 404") {
        rewritten.as_bytes().to_vec()
    } else {
        answered
    };
    let _ = client.write_all(&written);
}

/// How a stub answers a request, from its method and target.
pub type Answers = Arc<dyn Fn(&str, &str) -> String + Send + Sync>;

/// An endpoint that answers every request with what `answers` makes of its method and target.
pub fn stub(answers: Answers) -> Endpoint {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind port 0");
    let address = listener.local_addr().expect("bound address");
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let answers = Arc::clone(&answers);
            std::thread::spawn(move || {
                let Some((head, _)) = read_request(&mut stream) else {
                    return;
                };
                let (method, target) = request_line(&head);
                let _ = stream.write_all(answers(&method, &target).as_bytes());
            });
        }
    });
    Endpoint { address }
}

/// A `text/event-stream` answer carrying one event per item of `events`, then closing.
pub fn sse(events: &[&str]) -> String {
    let mut body = String::new();
    for event in events {
        body.push_str("data: ");
        body.push_str(event);
        body.push_str("\n\n");
    }
    format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n{body}")
}
