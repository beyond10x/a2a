//! Security review of story:conformance-gate: the conformance gate measures the tenant guard of
//! every push notification configuration command.
//!
//! `ServerTarget` declares a run unsupported where a command of `ACROSS_TENANTS`
//! (`crates/a2a-conformance/src/server.rs`) answers task-not-found on another tenant's task, and
//! `unsupported.tsv` lists those scenarios. Each case here runs the synthesized suite against the
//! server's ports with one command's tenant guard removed (the command acts on the task as if the
//! request were under the task's own tenant) and asserts that the gate turns red: some scenario's
//! status or reason differs from the run over the server's own ports, which the gate holds to 0
//! failed, 0 error and exactly the unsupported scenarios the allowance file names.
//!
//! The suite is synthesized with the `ess` on `PATH`, as `task conform` synthesizes it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

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
    CancelTask, CancelTaskOutcome, ContextId, CreateTaskPushNotificationConfig,
    CreateTaskPushNotificationConfigOutcome, DeleteTaskPushNotificationConfig,
    DeleteTaskPushNotificationConfigOutcome, GetTask, GetTaskOutcome,
    GetTaskPushNotificationConfig, GetTaskPushNotificationConfigOutcome,
    ListTaskPushNotificationConfigs, ListTaskPushNotificationConfigsOutcome,
    PushNotificationConfigRow, PushNotificationConfigSnapshot, SendMessage, SendMessageOutcome,
    SendStreamingMessage, SendStreamingMessageOutcome, SubscribeToTask, SubscribeToTaskOutcome,
    TaskEntitySnapshot, TaskId, TaskPushNotificationConfigs, TaskState, Tasks,
};
use a2a_conformance::{Results, ServerTarget, Status, Suite, run};
use a2a_server::ServerPorts;

const CREATE: u8 = 0;
const GET: u8 = 1;
const DELETE: u8 = 2;
const LIST: u8 = 3;

/// The server's ports, every obligation kept as they keep it, except that the push notification
/// configuration command `G` names runs under the tenant of the task it names, whatever tenant
/// the request is under: its tenant guard is gone.
struct Unguarded<const G: u8>(ServerPorts);

impl<const G: u8> Unguarded<G> {
    /// The tenant of the stored task `task`, where one is stored.
    fn owner(&self, task: &TaskId) -> Option<Option<String>> {
        TaskStorage::get(&self.0, task).map(|held| held.data.tenant)
    }
}

impl<const G: u8> CreateTaskPushNotificationConfigBehavior for Unguarded<G> {
    fn create_task_push_notification_config(
        &mut self,
        mut input: CreateTaskPushNotificationConfig,
    ) -> Result<CreateTaskPushNotificationConfigOutcome, UnmetObligation> {
        if G == CREATE
            && let Some(owner) = self.owner(&input.task_id)
        {
            input.tenant = owner;
        }
        self.0.create_task_push_notification_config(input)
    }
}

impl<const G: u8> GetTaskPushNotificationConfigBehavior for Unguarded<G> {
    fn get_task_push_notification_config(
        &mut self,
        mut input: GetTaskPushNotificationConfig,
    ) -> Result<GetTaskPushNotificationConfigOutcome, UnmetObligation> {
        if G == GET
            && let Some(owner) = self.owner(&input.task_id)
        {
            input.tenant = owner;
        }
        self.0.get_task_push_notification_config(input)
    }
}

impl<const G: u8> DeleteTaskPushNotificationConfigBehavior for Unguarded<G> {
    fn delete_task_push_notification_config(
        &mut self,
        mut input: DeleteTaskPushNotificationConfig,
    ) -> Result<DeleteTaskPushNotificationConfigOutcome, UnmetObligation> {
        if G == DELETE
            && let Some(owner) = self.owner(&input.task_id)
        {
            input.tenant = owner;
        }
        self.0.delete_task_push_notification_config(input)
    }
}

impl<const G: u8> ListTaskPushNotificationConfigsBehavior for Unguarded<G> {
    fn list_task_push_notification_configs(
        &mut self,
        mut input: ListTaskPushNotificationConfigs,
    ) -> Result<ListTaskPushNotificationConfigsOutcome, UnmetObligation> {
        if G == LIST
            && let Some(owner) = self.owner(&input.task_id)
        {
            input.tenant = owner;
        }
        self.0.list_task_push_notification_configs(input)
    }
}

impl<const G: u8> Context for Unguarded<G> {
    fn generate_optional_timestamp(&mut self) -> Option<Timestamp> {
        self.0.generate_optional_timestamp()
    }
}

impl<const G: u8> TaskStorage for Unguarded<G> {
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

impl<const G: u8> PushNotificationConfigStorage for Unguarded<G> {
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

impl<const G: u8> GetExtendedAgentCardBehavior for Unguarded<G> {
    fn get_extended_agent_card(
        &mut self,
        input: GetExtendedAgentCard,
    ) -> Result<GetExtendedAgentCardOutcome, UnmetObligation> {
        self.0.get_extended_agent_card(input)
    }
}

impl<const G: u8> CancelTaskBehavior for Unguarded<G> {
    fn cancel_task(&mut self, input: CancelTask) -> Result<CancelTaskOutcome, UnmetObligation> {
        self.0.cancel_task(input)
    }
}

impl<const G: u8> GetTaskBehavior for Unguarded<G> {
    fn get_task(&mut self, input: GetTask) -> Result<GetTaskOutcome, UnmetObligation> {
        self.0.get_task(input)
    }
}

impl<const G: u8> SendMessageBehavior for Unguarded<G> {
    fn send_message(&mut self, input: SendMessage) -> Result<SendMessageOutcome, UnmetObligation> {
        self.0.send_message(input)
    }
}

impl<const G: u8> SendStreamingMessageBehavior for Unguarded<G> {
    fn send_streaming_message(
        &mut self,
        input: SendStreamingMessage,
    ) -> Result<SendStreamingMessageOutcome, UnmetObligation> {
        self.0.send_streaming_message(input)
    }
}

impl<const G: u8> SubscribeToTaskBehavior for Unguarded<G> {
    fn subscribe_to_task(
        &mut self,
        input: SubscribeToTask,
    ) -> Result<SubscribeToTaskOutcome, UnmetObligation> {
        self.0.subscribe_to_task(input)
    }
}

impl<const G: u8> TasksQuery for Unguarded<G> {
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

impl<const G: u8> TaskPushNotificationConfigsQuery for Unguarded<G> {
    fn task_push_notification_configs(
        &self,
        task_id: TaskId,
        tenant: Option<String>,
    ) -> Result<Vec<TaskPushNotificationConfigs>, UnmetObligation> {
        self.0.task_push_notification_configs(task_id, tenant)
    }
}

/// The suite of `ess/`, synthesized once with the `ess` on `PATH` into this target's temporary
/// directory.
fn suite() -> Suite {
    static SYNTHESIZED: OnceLock<PathBuf> = OnceLock::new();
    let path = SYNTHESIZED.get_or_init(|| {
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sec-tenant-guards");
        let specification = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ess");
        if directory.exists() {
            std::fs::remove_dir_all(&directory).expect("the old synthesis is removed");
        }
        std::fs::create_dir_all(&directory).expect("the synthesis directory is made");
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
        suite
    });
    Suite::parse(&std::fs::read_to_string(path).expect("the suite was written"))
        .expect("the synthesized suite reads")
}

fn outcomes(results: &Results) -> BTreeMap<String, (Status, Option<String>)> {
    results
        .results
        .iter()
        .map(|result| {
            (
                result.scenario_id.clone(),
                (result.status, result.message.clone()),
            )
        })
        .collect()
}

/// Asserts that the run over `P` comes out otherwise than the run over the server's own ports
/// in at least one scenario, so the gate that holds the latter turns red on it.
fn the_gate_sees(command: &str, unguarded: &Results) {
    let suite = suite();
    let kept = outcomes(&run(&suite, &mut ServerTarget::new()));
    for (id, (status, message)) in &kept {
        assert!(
            !matches!(status, Status::Failed | Status::Error),
            "{id} is {status:?} over the server's own ports: {message:?}"
        );
    }
    let unguarded = outcomes(unguarded);
    let changed: Vec<(&String, &Status)> = kept
        .iter()
        .filter(|(id, kept)| unguarded.get(*id) != Some(kept))
        .map(|(id, _)| (id, &unguarded[id].0))
        .collect();
    println!("{command} without its tenant guard changes: {changed:?}");
    assert!(
        !changed.is_empty(),
        "{command} acting on another tenant's task leaves every scenario as it is over the \
         server's own ports: the conformance gate stays green"
    );
}

#[test]
fn sec_the_gate_turns_red_without_the_tenant_guard_of_create_push_config() {
    let results = run(&suite(), &mut ServerTarget::wrapped(Unguarded::<CREATE>));
    the_gate_sees("CreateTaskPushNotificationConfig", &results);
}

#[test]
fn sec_the_gate_turns_red_without_the_tenant_guard_of_get_push_config() {
    let results = run(&suite(), &mut ServerTarget::wrapped(Unguarded::<GET>));
    the_gate_sees("GetTaskPushNotificationConfig", &results);
}

#[test]
fn sec_the_gate_turns_red_without_the_tenant_guard_of_delete_push_config() {
    let results = run(&suite(), &mut ServerTarget::wrapped(Unguarded::<DELETE>));
    the_gate_sees("DeleteTaskPushNotificationConfig", &results);
}

#[test]
fn sec_the_gate_turns_red_without_the_tenant_guard_of_list_push_configs() {
    let results = run(&suite(), &mut ServerTarget::wrapped(Unguarded::<LIST>));
    the_gate_sees("ListTaskPushNotificationConfigs", &results);
}
