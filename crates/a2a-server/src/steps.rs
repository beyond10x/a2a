//! The steps an agent's [`Turn`](crate::Turn) asks for, issued as commands of the system's
//! `a2a-agent` component, whose `agent` actor the model declares `system: true`
//! (`ess/domains/tasks.yaml`). Each step is one command: it runs through its generated behaviour,
//! the system's pump collects what it published, and the stream frames come from that log, so a
//! follower sees exactly the events the system published.

use a2a::behaviour::Generated;
use a2a::json::Value;
use a2a::server::wire;
use a2a::system::{System, SystemEvent};
use a2a::tasks::{
    CompleteTask, CompleteTaskOutcome, ContextId, FailTask, FailTaskOutcome, MessageStreamResponse,
    RejectTask, RejectTaskOutcome, RequireAuth, RequireAuthOutcome, RequireInput,
    RequireInputOutcome, StartWorking, StartWorkingOutcome, TaskArtifactUpdateEvent, TaskId,
    TaskState, TaskStatusUpdateEvent, UpdateArtifact, UpdateArtifactOutcome,
};

use crate::agent::Step;
use crate::ports::{self, ServerPorts, Work};

/// The system the server runs.
pub(crate) type Running = System<Generated<ServerPorts>>;

/// One stream frame for the followers of a task.
pub(crate) type Frame = (TaskId, MessageStreamResponse);

/// Issues the steps of `work` in order and answers the frames the system published for them. A
/// step whose move the task's state does not allow is refused by its behaviour and publishes
/// nothing; a task that reaches a terminal state takes no further step.
pub(crate) fn run(system: &mut Running, ports: &ServerPorts, work: Work) -> Vec<Frame> {
    let task = work.task;
    let mut frames = Vec::new();
    for step in work.steps {
        let Some((state, context)) = ports.state_of(&task) else {
            break;
        };
        if ports::terminal(state) {
            break;
        }
        if matches!(step, Step::Work(_)) && state == TaskState::Working {
            continue;
        }
        ports.next_command();
        issue(system, ports, &task, &context, step);
        if let Err(unmet) = system.pump() {
            eprintln!("a2a-server: delivering what a step published: {unmet}");
        }
        frames.extend(system.take_published().into_iter().filter_map(frame));
    }
    frames
}

/// Issues one step as its command, and keeps what the generated behaviour leaves to the server:
/// the status message in the history, the artifact on the task.
fn issue(
    system: &mut Running,
    ports: &ServerPorts,
    task: &TaskId,
    context: &ContextId,
    step: Step,
) {
    let agent = &mut system.a2a_agent;
    let id = task.clone();
    let (message, moved) = match step {
        Step::Emit {
            artifact,
            append,
            last_chunk,
        } => {
            let updated = agent.update_artifact(UpdateArtifact {
                id,
                artifact: artifact.clone(),
                append,
                last_chunk,
            });
            if matches!(updated, Ok(UpdateArtifactOutcome::Updated { .. })) {
                ports.keep_artifact(task, artifact, append);
            }
            return;
        }
        Step::Work(message) => {
            let status_message = ports::attached(message, task, context);
            let moved = agent.start_working(StartWorking {
                id,
                status_message: status_message.clone(),
            });
            (
                status_message,
                matches!(moved, Ok(StartWorkingOutcome::Moved { .. })),
            )
        }
        Step::Complete(message) => {
            let status_message = ports::attached(message, task, context);
            let moved = agent.complete_task(CompleteTask {
                id,
                status_message: status_message.clone(),
            });
            (
                status_message,
                matches!(moved, Ok(CompleteTaskOutcome::Moved { .. })),
            )
        }
        Step::Fail(message) => {
            let status_message = ports::attached(message, task, context);
            let moved = agent.fail_task(FailTask {
                id,
                status_message: status_message.clone(),
            });
            (
                status_message,
                matches!(moved, Ok(FailTaskOutcome::Moved { .. })),
            )
        }
        Step::Reject(message) => {
            let status_message = ports::attached(message, task, context);
            let moved = agent.reject_task(RejectTask {
                id,
                status_message: status_message.clone(),
            });
            (
                status_message,
                matches!(moved, Ok(RejectTaskOutcome::Moved { .. })),
            )
        }
        Step::RequireInput(message) => {
            let status_message = ports::attached(message, task, context);
            let moved = agent.require_input(RequireInput {
                id,
                status_message: status_message.clone(),
            });
            (
                status_message,
                matches!(moved, Ok(RequireInputOutcome::Moved { .. })),
            )
        }
        Step::RequireAuth(message) => {
            let status_message = ports::attached(message, task, context);
            let moved = agent.require_auth(RequireAuth {
                id,
                status_message: status_message.clone(),
            });
            (
                status_message,
                matches!(moved, Ok(RequireAuthOutcome::Moved { .. })),
            )
        }
    };
    if let (Some(message), true) = (message, moved) {
        ports.record(task, message);
    }
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
