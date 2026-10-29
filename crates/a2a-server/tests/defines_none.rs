//! "Defines none": `crates/a2a-server/src` declares no `struct`, `enum`, `union`, `trait` or
//! `type` named as a type the generated crate exports, at any visibility (`pub`, `pub(crate)`,
//! `pub(in …)`, private) and with any qualifier (`unsafe`, `auto`). Every model type comes from
//! `generated/rust/a2a`; only server-only types (the agent, router state, settings) are declared
//! here.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const KINDS: [&str; 5] = ["struct ", "enum ", "union ", "trait ", "type "];
const QUALIFIERS: [&str; 2] = ["unsafe ", "auto "];

fn sources(root: &Path, skip: Option<&str>, into: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(root)
        .unwrap_or_else(|error| panic!("{}: {error}", root.display()))
        .map(|entry| entry.expect("a directory entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            if skip.is_some_and(|skip| path.file_name().is_some_and(|name| name == skip)) {
                continue;
            }
            sources(&path, skip, into);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            into.push(path);
        }
    }
}

/// The names `text` declares by a type item; `public` keeps only those a plain `pub` exports.
fn declared_in(text: &str, public: bool) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in text.lines() {
        let line = line.trim_start();
        let item = match line.strip_prefix("pub") {
            Some(rest) if rest.starts_with(' ') => rest.trim_start(),
            Some(_) if public => continue,
            Some(rest) if rest.starts_with('(') => match rest.split_once(')') {
                Some((_, item)) => item.trim_start(),
                None => continue,
            },
            _ if public => continue,
            _ => line,
        };
        let mut item = item;
        while let Some(rest) = QUALIFIERS
            .iter()
            .find_map(|qualifier| item.strip_prefix(qualifier))
        {
            item = rest.trim_start();
        }
        for kind in KINDS {
            if let Some(rest) = item.strip_prefix(kind) {
                let name: String = rest
                    .trim_start()
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    names.insert(name);
                }
            }
        }
    }
    names
}

fn declared(files: &[PathBuf], public: bool) -> BTreeSet<String> {
    files
        .iter()
        .flat_map(|file| {
            declared_in(
                &std::fs::read_to_string(file).expect("a source file"),
                public,
            )
        })
        .collect()
}

#[test]
fn the_server_declares_no_type_the_generated_crate_exports() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut generated_files = Vec::new();
    sources(
        &manifest.join("../../generated/rust/a2a/src"),
        Some("bin"),
        &mut generated_files,
    );
    let generated = declared(&generated_files, true);
    assert!(
        generated.len() > 100,
        "the generated crate's types were not read: {generated:?}"
    );
    for model in [
        "Task",
        "Message",
        "AgentCard",
        "TaskSnapshot",
        "SendMessageOutcome",
    ] {
        assert!(
            generated.contains(model),
            "{model} is not read as generated"
        );
    }

    let mut server_files = Vec::new();
    sources(&manifest.join("src"), None, &mut server_files);
    let server = declared(&server_files, false);
    assert!(
        server.contains("EchoAgent"),
        "the server's own types were not read: {server:?}"
    );

    let both: Vec<_> = server.intersection(&generated).collect();
    assert!(
        both.is_empty(),
        "crates/a2a-server/src declares types the generated crate exports: {both:?}"
    );
}

/// The forms the wave-3 adversary planted (an `unsafe` trait, a `pub(in …)` struct), each other
/// visibility and qualifier, and what is no declaration.
#[test]
fn every_visibility_and_qualifier_is_read() {
    let planted = "\
/// Planted: a hand-declared model type.
pub unsafe trait AgentCard {}
pub(in crate::agent) struct Task(pub(in crate::agent) u8);
pub(crate) enum Message { A }
pub(super) type TaskId = String;
pub(self) union Part { a: u8 }
unsafe auto trait Artifact {}
struct TaskStatus;
pub fn planted() -> u8 { Task(0).0 }
let r#type = 1;
";
    let all: Vec<_> = declared_in(planted, false).into_iter().collect();
    assert_eq!(
        all,
        [
            "AgentCard",
            "Artifact",
            "Message",
            "Part",
            "Task",
            "TaskId",
            "TaskStatus"
        ]
    );
    let exported: Vec<_> = declared_in(planted, true).into_iter().collect();
    assert_eq!(exported, ["AgentCard"]);
}
