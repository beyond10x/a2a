//! Adversarial cases against the `Agent` seam of `a2a-server` (story:server, wave 3): an agent
//! must not drive a task through a move its lifecycle forbids, and artifacts append or replace as
//! `TaskArtifactUpdateEvent.append` says.

mod common;

use a2a::json::Value;
use a2a::tasks::{Artifact, Message, Part, TaskSnapshot};
use a2a_server::{Agent, Step, Turn};
use common::{at, card, count, message, start, text};

struct Scripted(Vec<Vec<Step>>);

impl Agent for Scripted {
    fn turn(&mut self, _message: &Message, _task: Option<&TaskSnapshot>) -> Turn {
        Turn::Steps(if self.0.is_empty() {
            Vec::new()
        } else {
            self.0.remove(0)
        })
    }
}

fn part(text: &str) -> Part {
    Part {
        text: Some(text.to_owned()),
        raw: None,
        url: None,
        data: None,
        metadata: None,
        filename: None,
        media_type: None,
    }
}

fn artifact(id: &str, text: &str) -> Artifact {
    Artifact {
        artifact_id: id.to_owned(),
        name: None,
        description: None,
        parts: vec![part(text)],
        metadata: None,
        extensions: None,
    }
}

fn emit(id: &str, text: &str, append: bool) -> Step {
    Step::Emit {
        artifact: artifact(id, text),
        append,
        last_chunk: false,
    }
}

fn state_of(frame: &Value) -> String {
    let task = text(frame, &["task", "status", "state"]);
    if task.is_empty() {
        text(frame, &["statusUpdate", "status", "state"])
    } else {
        task
    }
}

#[test]
fn adv_an_agent_cannot_move_a_completed_task_on() {
    let steps = vec![
        Step::Work(None),
        emit("first", "one", false),
        Step::Complete(None),
        Step::Work(None),
        emit("second", "two", false),
        Step::Fail(None),
        Step::RequireInput(None),
    ];
    let server = start(card(|_| {}), move || Box::new(Scripted(vec![steps])));
    let reply = server.call("POST", "/message:stream", Some(&message("go", None, None)));
    assert_eq!(reply.status, 200, "{}", reply.body);
    let frames = reply.frames();
    let states: Vec<String> = frames
        .iter()
        .map(state_of)
        .filter(|state| !state.is_empty())
        .collect();
    assert_eq!(
        states,
        [
            "TASK_STATE_SUBMITTED",
            "TASK_STATE_WORKING",
            "TASK_STATE_COMPLETED"
        ],
        "{}",
        reply.body
    );
    let id = text(&frames[0], &["task", "id"]);
    let task = server.call("GET", &format!("/tasks/{id}"), None).json();
    assert_eq!(text(&task, &["status", "state"]), "TASK_STATE_COMPLETED");
    assert_eq!(count(&task, &["artifacts"]), 1, "{task:?}");
    assert_eq!(text(&task, &["artifacts", "0", "artifactId"]), "first");
}

#[test]
fn adv_an_appended_chunk_extends_its_artifact_and_a_replacement_replaces_it() {
    let appended = vec![
        Step::Work(None),
        emit("x", "one", false),
        emit("x", "two", true),
        Step::Complete(None),
    ];
    let replaced = vec![
        Step::Work(None),
        emit("x", "one", false),
        emit("x", "two", false),
        Step::Complete(None),
    ];
    let server = start(card(|_| {}), move || {
        Box::new(Scripted(vec![appended, replaced]))
    });
    let first = server.send_text("a");
    let parts = |task: &Value| -> Vec<String> {
        match at(task, &["task", "artifacts", "0", "parts"]) {
            Some(Value::Array(parts)) => parts.iter().map(|part| text(part, &["text"])).collect(),
            _ => Vec::new(),
        }
    };
    assert_eq!(count(&first, &["task", "artifacts"]), 1, "{first:?}");
    assert_eq!(parts(&first), ["one", "two"], "{first:?}");
    let second = server.send_text("b");
    assert_eq!(count(&second, &["task", "artifacts"]), 1, "{second:?}");
    assert_eq!(parts(&second), ["two"], "{second:?}");
}
