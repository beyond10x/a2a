//! Security review of story:server (wave 3): the binary's default address and what it writes
//! about a card file it cannot use.

use std::process::Command;

fn run(arguments: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_a2a-server"))
        .args(arguments)
        .output()
        .expect("run a2a-server");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Without `--listen`, the binary listens on the loopback interface only.
#[test]
fn sec_the_binary_listens_on_loopback_by_default() {
    let (ok, help, _) = run(&["--help"]);
    assert!(
        ok && help.contains("[default: 127.0.0.1:8080]"),
        "the help does not name 127.0.0.1:8080 as the default address: {help}"
    );
}

/// A card file that does not decode is refused naming the file, and none of its content reaches
/// standard error or standard output.
#[test]
fn sec_a_card_file_that_does_not_decode_is_not_echoed() {
    let directory = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("sec-binary-cards");
    std::fs::create_dir_all(&directory).expect("a directory for the cards");
    let mut echoed = Vec::new();
    for (name, content) in [
        ("not-json", "secret-value-one is not JSON"),
        ("not-a-card", "{\"name\":\"secret-value-two\"}"),
        (
            "wrong-type",
            "{\"name\":\"n\",\"description\":\"d\",\"version\":\"1\",\"supportedInterfaces\":\"secret-value-three\",\"capabilities\":{},\"defaultInputModes\":[],\"defaultOutputModes\":[],\"skills\":[]}",
        ),
    ] {
        let path = directory.join(name);
        std::fs::write(&path, content).expect("write the card");
        let path = path.to_string_lossy().into_owned();
        let (ok, stdout, stderr) = run(&["--listen", "127.0.0.1:0", "--card", &path]);
        if ok || stdout.contains("secret-value") || stderr.contains("secret-value") {
            echoed.push(format!(
                "{name}: success {ok}, stdout {stdout:?}, stderr {stderr:?}"
            ));
        }
    }
    assert!(echoed.is_empty(), "{echoed:#?}");
}
