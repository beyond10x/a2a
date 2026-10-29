//! The steps an agent's [`Turn`](crate::Turn) asks for, issued as commands of the system's
//! `a2a-agent` component, whose `agent` actor the model declares `system: true`
//! (`ess/domains/tasks.yaml`). Each step is one command, run as that actor through the generated
//! layer (`a2a::server::a2a_agent::handle_with_events`): its grant is checked, it runs through its
//! generated behaviour, the system's pump delivers what it published, and the stream frames come
//! from that log, so a follower sees exactly the events the system published. The server loop
//! ([`crate::serve_with`]) and [`crate::InProcess`] issue steps here and nowhere else.

use a2a::actor::{Actor, Caller};
use a2a::behaviour::Generated;
use a2a::json::{self, Value};
use a2a::server::entry::Refused;
use a2a::server::{a2a_agent, wire};
use a2a::system::{System, SystemEvent};
use a2a::tasks::{
    Artifact, ContextId, Message, MessageStreamResponse, TaskArtifactUpdateEvent, TaskId,
    TaskState, TaskStatusUpdateEvent,
};

use crate::Obligations;
use crate::agent::Step;
use crate::ports::{self, ServerPorts, Work};

/// One stream frame for the followers of a task.
pub(crate) type Frame = (TaskId, MessageStreamResponse);

/// One step, issued as its command.
pub(crate) struct Issued {
    /// The input the command was issued with, wire-named.
    pub(crate) input: Value,
    /// The generated layer's answer: the outcome the command took, or why it took none.
    pub(crate) answer: Result<Value, Refused>,
    /// Every event the system published while the command ran, in log order.
    pub(crate) events: Vec<SystemEvent>,
}

/// What the server keeps of a step once its command took effect.
enum Kept {
    Artifact(Artifact, bool),
    Status(Option<Message>),
}

/// Issues the steps of `work` in order and answers every event the system published for them, in
/// log order. A step whose move the task's state does not allow is refused by its behaviour and
/// publishes nothing; a task that is gone or in a terminal state takes no further step.
pub(crate) fn run<P: Obligations>(
    system: &mut System<Generated<P>>,
    ports: &ServerPorts,
    work: Work,
) -> Vec<SystemEvent> {
    let mut events = Vec::new();
    for step in work.steps {
        let Some(issued) = issue(system, ports, &work.task, step) else {
            continue;
        };
        if let Err(refused) = &issued.answer {
            eprintln!("a2a-server: issuing a step on {}: {refused:?}", work.task.0);
        }
        events.extend(issued.events);
    }
    events
}

/// Issues `step` on `task` as its `a2a.tasks.Agent` command, and keeps what the generated
/// behaviour leaves to the server: a status message it moved the task with, in the history; an
/// artifact it updated, on the task. `None`, issuing nothing, where the task is gone or in a
/// terminal state, or the step is `Work` and the task is working.
pub(crate) fn issue<P: Obligations>(
    system: &mut System<Generated<P>>,
    ports: &ServerPorts,
    task: &TaskId,
    step: Step,
) -> Option<Issued> {
    let (state, context) = ports.state_of(task)?;
    if ports::terminal(state) || (matches!(step, Step::Work(_)) && state == TaskState::Working) {
        return None;
    }
    let (command, input, kept) = command_of(task, &context, step)?;
    ports.next_command();
    let agent = Caller {
        actor: Actor::TasksAgent,
    };
    let (answer, events) =
        a2a_agent::handle_with_events(system, Some(&agent), command, input.clone());
    let outcome = match answer
        .as_ref()
        .ok()
        .and_then(|answer| answer.member("outcome"))
    {
        Some(Value::Text(outcome)) => outcome.as_str(),
        _ => "",
    };
    match (kept, outcome) {
        (Kept::Artifact(artifact, append), "updated") => {
            ports.keep_artifact(task, artifact, append)
        }
        (Kept::Status(Some(message)), "moved") => ports.record(task, message),
        _ => {}
    }
    Some(Issued {
        input,
        answer,
        events,
    })
}

/// The command `step` is issued as on `task` of `context`, its wire-named input, and what the
/// server keeps of it once it took effect.
fn command_of(
    task: &TaskId,
    context: &ContextId,
    step: Step,
) -> Option<(&'static str, Value, Kept)> {
    let mut input = String::from("{");
    json::member(&mut input, "id");
    json::push_text(&mut input, &task.0);
    let (command, kept) = match step {
        Step::Emit {
            artifact,
            append,
            last_chunk,
        } => {
            json::member(&mut input, "artifact");
            wire::encode_a2a_tasks_artifact(&artifact, &mut input);
            json::member(&mut input, "append");
            json::push_bool(&mut input, append);
            json::member(&mut input, "last_chunk");
            json::push_bool(&mut input, last_chunk);
            ("a2a.tasks.UpdateArtifact", Kept::Artifact(artifact, append))
        }
        Step::Work(message) => ("a2a.tasks.StartWorking", Kept::Status(message)),
        Step::Complete(message) => ("a2a.tasks.CompleteTask", Kept::Status(message)),
        Step::Fail(message) => ("a2a.tasks.FailTask", Kept::Status(message)),
        Step::Reject(message) => ("a2a.tasks.RejectTask", Kept::Status(message)),
        Step::RequireInput(message) => ("a2a.tasks.RequireInput", Kept::Status(message)),
        Step::RequireAuth(message) => ("a2a.tasks.RequireAuth", Kept::Status(message)),
    };
    let kept = match kept {
        Kept::Status(message) => {
            let message = ports::attached(message, task, context);
            if let Some(message) = &message {
                json::member(&mut input, "status_message");
                wire::encode_a2a_tasks_message(message, &mut input);
            }
            Kept::Status(message)
        }
        artifact @ Kept::Artifact(..) => artifact,
    };
    input.push('}');
    let input = json::parse_written(&input).ok()?;
    Some((command, input, kept))
}

/// The stream frames of `events`, for the events a stream carries, in order.
pub(crate) fn frames(events: Vec<SystemEvent>) -> Vec<Frame> {
    events.into_iter().filter_map(frame).collect()
}

/// The stream frame a published event is, for the events a stream carries.
fn frame(event: SystemEvent) -> Option<Frame> {
    match event {
        SystemEvent::TaskStatusUpdated(moved) => Some((
            moved.task_id.clone(),
            MessageStreamResponse {
                task: None,
                message: None,
                status_update: Some(TaskStatusUpdateEvent {
                    task_id: moved.task_id,
                    context_id: moved.context_id,
                    status: moved.status,
                    metadata: None,
                }),
                artifact_update: None,
            },
        )),
        SystemEvent::TaskArtifactUpdated(updated) => Some((
            updated.task_id.clone(),
            MessageStreamResponse {
                task: None,
                message: None,
                status_update: None,
                artifact_update: Some(TaskArtifactUpdateEvent {
                    task_id: updated.task_id,
                    context_id: updated.context_id,
                    artifact: updated.artifact,
                    append: updated.append,
                    last_chunk: updated.last_chunk,
                    metadata: None,
                }),
            },
        )),
        _ => None,
    }
}

/// The status frames a command the generated layer ran published, read from the `published`
/// list of its envelope. The layer pumps and takes the system's log itself, so the envelope is
/// where those events remain. A dispatched command publishes no artifact event: only
/// `UpdateArtifact` does, and it runs as a step.
// ESS gap (ess 0.67.0): the generated `dispatch` and `handle` return a JSON envelope and no typed
// `SystemEvent`s, so the events a command published are decoded back from its `published` list.
pub(crate) fn published(envelope: &Value) -> Vec<Frame> {
    let Some(Value::Array(events)) = envelope.member("published") else {
        return Vec::new();
    };
    events
        .iter()
        .filter(|event| {
            matches!(event.member("event"), Some(Value::Text(name)) if name == "a2a.tasks.TaskStatusUpdated")
        })
        .filter_map(|event| {
            let payload = event.member("payload")?;
            let text = |name: &str| match payload.member(name) {
                Some(Value::Text(text)) => Some(text.clone()),
                _ => None,
            };
            let status =
                wire::decode_a2a_tasks_task_status(payload.member("status")?, "status").ok()?;
            frame(SystemEvent::TaskStatusUpdated(
                a2a::tasks::TaskStatusUpdated {
                    task_id: TaskId(text("task_id")?),
                    context_id: ContextId(text("context_id")?),
                    status,
                },
            ))
        })
        .collect()
}
