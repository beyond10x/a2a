//! The suite catches a broken obligation: `a2a-server`'s ports with CancelTask answering
//! `canceled` for a task already in a terminal state.
//!
//! The suite is synthesized here with the `ess` on `PATH`, as `task conform` synthesizes it, into
//! this test target's temporary directory.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use a2a::behaviour::{Context, PushNotificationConfigStorage, TaskStorage};
use a2a::discovery::obligations::GetExtendedAgentCardBehavior;
use a2a::discovery::{GetExtendedAgentCard, GetExtendedAgentCardOutcome};
use a2a::obligation::UnmetObligation;
use a2a::primitives::Timestamp;
use a2a::tasks::obligations::{
    CancelTaskBehavior, CreateTaskPushNotificationConfigBehavior,
    DeleteTaskPushNotificationConfigBehavior, GetTaskBehavior,
    GetTaskPushNotificationConfigBehavior, ListTaskPushNotificationConfigsBehavior,
    SendMessageBehavior, SendStreamingMessageBehavior, SubscribeToTaskBehavior,
    TaskPushNotificationConfigsQuery, TasksQuery,
};
use a2a::tasks::{
    CancelTask, CancelTaskOutcome, CancelTaskResponse, ContextId, CreateTaskPushNotificationConfig,
    CreateTaskPushNotificationConfigOutcome, DeleteTaskPushNotificationConfig,
    DeleteTaskPushNotificationConfigOutcome, GetTask, GetTaskOutcome,
    GetTaskPushNotificationConfig, GetTaskPushNotificationConfigOutcome,
    ListTaskPushNotificationConfigs, ListTaskPushNotificationConfigsOutcome,
    PushNotificationConfigRow, PushNotificationConfigSnapshot, SendMessage, SendMessageOutcome,
    SendStreamingMessage, SendStreamingMessageOutcome, SubscribeToTask, SubscribeToTaskOutcome,
    TaskEntitySnapshot, TaskId, TaskPushNotificationConfigs, TaskSnapshot, TaskState, TaskStatus,
    TaskStatusUpdated, Tasks, WireTaskState,
};
use a2a_conformance::{Results, ServerTarget, Status, Suite, run};
use a2a_server::ServerPorts;

/// The server's ports, every obligation kept as they keep it, except that CancelTask answers
/// `canceled` for a task in a terminal state instead of refusing it.
struct Broken(ServerPorts);

impl CancelTaskBehavior for Broken {
    fn cancel_task(&mut self, input: CancelTask) -> Result<CancelTaskOutcome, UnmetObligation> {
        let held = TaskStorage::get(&self.0, &input.id);
        match held {
            Some(held)
                if matches!(
                    held.state,
                    TaskState::Completed
                        | TaskState::Failed
                        | TaskState::Canceled
                        | TaskState::Rejected
                ) =>
            {
                let status = TaskStatus {
                    state: WireTaskState::TASKSTATECANCELED,
                    message: None,
                    timestamp: held.data.status_timestamp.clone(),
                };
                Ok(CancelTaskOutcome::Canceled {
                    response: CancelTaskResponse {
                        task: TaskSnapshot {
                            id: held.data.id.clone(),
                            context_id: Some(held.data.context_id.clone()),
                            status: status.clone(),
                            artifacts: held.data.artifacts.clone(),
                            history: None,
                            metadata: held.data.metadata.clone(),
                        },
                    },
                    task_status_updated: TaskStatusUpdated {
                        task_id: held.data.id,
                        context_id: held.data.context_id,
                        status,
                    },
                })
            }
            _ => self.0.cancel_task(input),
        }
    }
}

impl Context for Broken {
    fn generate_optional_timestamp(&mut self) -> Option<Timestamp> {
        self.0.generate_optional_timestamp()
    }
}

impl TaskStorage for Broken {
    fn get(&self, identity: &TaskId) -> Option<TaskEntitySnapshot> {
        TaskStorage::get(&self.0, identity)
    }

    fn put(&mut self, snapshot: TaskEntitySnapshot) {
        TaskStorage::put(&mut self.0, snapshot);
    }

    fn delete(&mut self, identity: &TaskId) {
        TaskStorage::delete(&mut self.0, identity);
    }

    fn list(&self) -> Vec<TaskEntitySnapshot> {
        TaskStorage::list(&self.0)
    }
}

impl PushNotificationConfigStorage for Broken {
    fn get(&self, identity: &PushNotificationConfigRow) -> Option<PushNotificationConfigSnapshot> {
        PushNotificationConfigStorage::get(&self.0, identity)
    }

    fn put(&mut self, snapshot: PushNotificationConfigSnapshot) {
        PushNotificationConfigStorage::put(&mut self.0, snapshot);
    }

    fn delete(&mut self, identity: &PushNotificationConfigRow) {
        PushNotificationConfigStorage::delete(&mut self.0, identity);
    }

    fn list(&self) -> Vec<PushNotificationConfigSnapshot> {
        PushNotificationConfigStorage::list(&self.0)
    }
}

impl GetExtendedAgentCardBehavior for Broken {
    fn get_extended_agent_card(
        &mut self,
        input: GetExtendedAgentCard,
    ) -> Result<GetExtendedAgentCardOutcome, UnmetObligation> {
        self.0.get_extended_agent_card(input)
    }
}

impl CreateTaskPushNotificationConfigBehavior for Broken {
    fn create_task_push_notification_config(
        &mut self,
        input: CreateTaskPushNotificationConfig,
    ) -> Result<CreateTaskPushNotificationConfigOutcome, UnmetObligation> {
        self.0.create_task_push_notification_config(input)
    }
}

impl DeleteTaskPushNotificationConfigBehavior for Broken {
    fn delete_task_push_notification_config(
        &mut self,
        input: DeleteTaskPushNotificationConfig,
    ) -> Result<DeleteTaskPushNotificationConfigOutcome, UnmetObligation> {
        self.0.delete_task_push_notification_config(input)
    }
}

impl GetTaskBehavior for Broken {
    fn get_task(&mut self, input: GetTask) -> Result<GetTaskOutcome, UnmetObligation> {
        self.0.get_task(input)
    }
}

impl GetTaskPushNotificationConfigBehavior for Broken {
    fn get_task_push_notification_config(
        &mut self,
        input: GetTaskPushNotificationConfig,
    ) -> Result<GetTaskPushNotificationConfigOutcome, UnmetObligation> {
        self.0.get_task_push_notification_config(input)
    }
}

impl ListTaskPushNotificationConfigsBehavior for Broken {
    fn list_task_push_notification_configs(
        &mut self,
        input: ListTaskPushNotificationConfigs,
    ) -> Result<ListTaskPushNotificationConfigsOutcome, UnmetObligation> {
        self.0.list_task_push_notification_configs(input)
    }
}

impl SendMessageBehavior for Broken {
    fn send_message(&mut self, input: SendMessage) -> Result<SendMessageOutcome, UnmetObligation> {
        self.0.send_message(input)
    }
}

impl SendStreamingMessageBehavior for Broken {
    fn send_streaming_message(
        &mut self,
        input: SendStreamingMessage,
    ) -> Result<SendStreamingMessageOutcome, UnmetObligation> {
        self.0.send_streaming_message(input)
    }
}

impl SubscribeToTaskBehavior for Broken {
    fn subscribe_to_task(
        &mut self,
        input: SubscribeToTask,
    ) -> Result<SubscribeToTaskOutcome, UnmetObligation> {
        self.0.subscribe_to_task(input)
    }
}

impl TasksQuery for Broken {
    fn tasks(
        &self,
        context_id: Option<ContextId>,
        status: Option<TaskState>,
        status_timestamp_after: Option<Timestamp>,
        tenant: Option<String>,
    ) -> Result<Vec<Tasks>, UnmetObligation> {
        self.0
            .tasks(context_id, status, status_timestamp_after, tenant)
    }
}

impl TaskPushNotificationConfigsQuery for Broken {
    fn task_push_notification_configs(
        &self,
        task_id: TaskId,
    ) -> Result<Vec<TaskPushNotificationConfigs>, UnmetObligation> {
        self.0.task_push_notification_configs(task_id)
    }
}

/// Synthesizes the suite of `ess/` with the `ess` on `PATH` into `directory`, emptied first.
fn synthesize(directory: &Path) -> Suite {
    let specification = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ess");
    if directory.exists() {
        std::fs::remove_dir_all(directory).expect("the old synthesis is removed");
    }
    std::fs::create_dir_all(directory).expect("the synthesis directory is made");
    let suite = directory.join("suite.json");
    let synthesized = Command::new("ess")
        .args(["verify", "conform", "synthesize", "--path"])
        .arg(&specification)
        .arg("--scenarios")
        .arg(&specification)
        .arg("--out")
        .arg(&suite)
        .output()
        .expect("`ess` runs: the pinned ess must be on PATH");
    assert!(
        synthesized.status.success(),
        "ess verify conform synthesize: {}",
        String::from_utf8_lossy(&synthesized.stderr)
    );
    Suite::parse(&std::fs::read_to_string(&suite).expect("the suite was written"))
        .expect("the synthesized suite reads")
}

fn statuses(results: &Results) -> BTreeMap<String, Status> {
    results
        .results
        .iter()
        .map(|result| (result.scenario_id.clone(), result.status))
        .collect()
}

const REFUSALS: [&str; 4] = [
    "a2a.tasks.Task/state/Completed/refuses/a2a.tasks.CancelTask",
    "a2a.tasks.Task/state/Failed/refuses/a2a.tasks.CancelTask",
    "a2a.tasks.Task/state/Canceled/refuses/a2a.tasks.CancelTask",
    "a2a.tasks.Task/state/Rejected/refuses/a2a.tasks.CancelTask",
];

const CANCELED: &str = "a2a.tasks.CancelTask/outcome/canceled";

#[test]
fn canceling_a_task_in_a_terminal_state_is_caught_by_name() {
    let directory: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR")).join("planted-defect");
    let suite = synthesize(&directory);

    let broken = statuses(&run(&suite, &mut ServerTarget::wrapped(Broken)));
    let kept = statuses(&run(&suite, &mut ServerTarget::new()));

    for id in REFUSALS {
        assert_eq!(
            kept.get(id),
            Some(&Status::Passed),
            "{id} with the server's own ports"
        );
        assert_eq!(
            broken.get(id),
            Some(&Status::Failed),
            "{id} with the broken CancelTask"
        );
    }
    assert_eq!(kept.get(CANCELED), Some(&Status::Passed));
    assert_eq!(broken.get(CANCELED), Some(&Status::Passed));

    let changed: Vec<&String> = kept
        .iter()
        .filter(|(id, status)| broken.get(*id) != Some(status))
        .map(|(id, _)| id)
        .collect();
    println!("scenarios the broken CancelTask changes: {changed:?}");
    assert!(
        changed
            .iter()
            .all(|id| broken.get(*id) == Some(&Status::Failed)),
        "the broken CancelTask only fails scenarios: {changed:?}"
    );
}
