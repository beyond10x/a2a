//! Adversarial cases against the `a2a-server` binary's start-up (story:server, wave 3).

use std::process::Command;

fn run(arguments: &[&str]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_a2a-server"))
        .args(arguments)
        .output()
        .expect("run a2a-server");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn adv_the_binary_refuses_a_listen_value_that_is_no_address() {
    let (succeeded, stderr) = run(&["--listen", "not-an-address"]);
    assert!(!succeeded, "{stderr}");
    assert!(stderr.contains("not-an-address"), "{stderr}");
}

#[test]
fn adv_the_binary_refuses_a_card_path_that_does_not_exist() {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("adv-no-such-card.json");
    let _ = std::fs::remove_file(&path);
    let (succeeded, stderr) = run(&[
        "--listen",
        "127.0.0.1:0",
        "--card",
        path.to_str().expect("a UTF-8 path"),
    ]);
    assert!(!succeeded, "{stderr}");
    assert!(stderr.contains("adv-no-such-card.json"), "{stderr}");
}

#[test]
fn adv_the_binary_refuses_a_card_that_is_not_json() {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("adv-not-json-card.json");
    std::fs::write(&path, "name: yaml").expect("write");
    let (succeeded, stderr) = run(&[
        "--listen",
        "127.0.0.1:0",
        "--card",
        path.to_str().expect("a UTF-8 path"),
    ]);
    assert!(!succeeded, "{stderr}");
    assert!(stderr.contains("adv-not-json-card.json"), "{stderr}");
}
