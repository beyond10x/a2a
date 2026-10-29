//! The agent behind the server: what decides the branches the specification leaves "externally
//! decided" (direct message or task, content types, contexts, cancelability), and the echo
//! reference agent.
//!
//! The server owns everything the model determines: storage, the task lifecycle, versions,
//! extensions and the card's capabilities. An [`Agent`] only answers a message.

use a2a::json::Value;
use a2a::tasks::{Artifact, Message, Part, Role, TaskSnapshot};

/// Answers messages for the server.
pub trait Agent {
    /// How the agent answers `message`. `task` is the task the message continues, already holding
    /// the message in its history, or `None` when the message starts one.
    fn turn(&mut self, message: &Message, task: Option<&TaskSnapshot>) -> Turn;

    /// Whether `task`, which is in no terminal state, can be canceled at its current stage. A
    /// `false` answers `TaskNotCancelableError` (`not-cancelable-now`).
    fn cancelable(&self, task: &TaskSnapshot) -> bool {
        let _ = task;
        true
    }
}

/// An agent's answer to one message.
#[derive(Debug, Clone, PartialEq)]
pub enum Turn {
    /// Answer with this message and create no task (`direct-message`).
    Reply(Message),
    /// Take the message into a task (`task-created`, or `continued` for an existing task) and
    /// move it through these steps, in order.
    Steps(Vec<Step>),
    /// Refuse the message.
    Refuse(Refusal),
}

/// One step of an agent's work on a task. Each is one generated lifecycle command; a step whose
/// move the task's state does not allow is skipped.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// `StartWorking`, with an optional status message.
    Work(Option<Message>),
    /// `UpdateArtifact`: add the artifact, or append its parts to the artifact of the same id.
    Emit {
        /// The artifact, or the chunk of it.
        artifact: Artifact,
        /// Append to the artifact of the same id instead of replacing it.
        append: bool,
        /// This is the artifact's last chunk.
        last_chunk: bool,
    },
    /// `CompleteTask`.
    Complete(Option<Message>),
    /// `FailTask`.
    Fail(Option<Message>),
    /// `RejectTask`.
    Reject(Option<Message>),
    /// `RequireInput`: the task waits for the client's next message.
    RequireInput(Option<Message>),
    /// `RequireAuth`.
    RequireAuth(Option<Message>),
}

/// Why an agent refuses a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// A part's media type is not one the agent accepts (`ContentTypeNotSupportedError`).
    ContentType,
    /// The agent cannot accept the client's `contextId` (`context-not-accepted`).
    Context,
}

/// The reference agent: completes each task with the message's parts as an artifact.
///
/// - A part whose `mediaType` is not one of the accepted modes is refused.
/// - A message whose `metadata.mode` is `"message"` is answered directly, with its parts.
/// - A message without non-empty content asks for input.
/// - Any other message is worked on, answered with an artifact `echo` holding its parts, and
///   completed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EchoAgent {
    accepted: Vec<String>,
}

impl EchoAgent {
    /// An echo agent accepting parts of these media types, and parts that name none.
    pub fn new(accepted: Vec<String>) -> Self {
        Self { accepted }
    }

    fn accepts(&self, part: &Part) -> bool {
        part.media_type
            .as_ref()
            .is_none_or(|media_type| self.accepted.iter().any(|mode| mode == media_type))
    }
}

impl Agent for EchoAgent {
    fn turn(&mut self, message: &Message, _task: Option<&TaskSnapshot>) -> Turn {
        if !message.parts.iter().all(|part| self.accepts(part)) {
            return Turn::Refuse(Refusal::ContentType);
        }
        let direct = message
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.member("mode"))
            .is_some_and(|mode| *mode == Value::Text("message".to_owned()));
        if direct {
            return Turn::Reply(agent_message(message.parts.clone()));
        }
        if !message.parts.iter().any(has_content) {
            let prompt = Part {
                text: Some("send the text to echo".to_owned()),
                raw: None,
                url: None,
                data: None,
                metadata: None,
                filename: None,
                media_type: None,
            };
            return Turn::Steps(vec![
                Step::Work(None),
                Step::RequireInput(Some(agent_message(vec![prompt]))),
            ]);
        }
        let artifact = Artifact {
            artifact_id: uuid::Uuid::new_v4().to_string(),
            name: Some("echo".to_owned()),
            description: None,
            parts: message.parts.clone(),
            metadata: None,
            extensions: None,
        };
        Turn::Steps(vec![
            Step::Work(None),
            Step::Emit {
                artifact,
                append: false,
                last_chunk: true,
            },
            Step::Complete(None),
        ])
    }
}

fn has_content(part: &Part) -> bool {
    part.text.as_ref().is_some_and(|text| !text.is_empty())
        || part.raw.as_ref().is_some_and(|raw| !raw.is_empty())
        || part.url.is_some()
        || part.data.is_some()
}

/// A message from the agent; the server fills in its task and context.
pub fn agent_message(parts: Vec<Part>) -> Message {
    Message {
        message_id: uuid::Uuid::new_v4().to_string(),
        context_id: None,
        task_id: None,
        role: Role::ROLEAGENT,
        parts,
        metadata: None,
        extensions: None,
        reference_task_ids: None,
    }
}
