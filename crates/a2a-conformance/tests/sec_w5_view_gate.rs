//! Security review of story:view-tenant-scope (wave 5): the conformance gate measures the tenant
//! scope of the `a2a.tasks.TaskPushNotificationConfigs` view and the tenant a configuration is
//! stored under, with `REARRANGED` (`crates/a2a-conformance/src/server.rs`) in place.
//!
//! `REARRANGED` sends the arranging runs of the Get and Delete configuration scenarios under the
//! tenant the harness created the task under, whatever tenant they name. Each case here runs the
//! synthesized suite against the server's ports with one defect planted (the view reads past the
//! tenant it is read under, or a configuration is stored under a tenant other than the one it was
//! created under) and asserts that the gate turns red: some scenario's status or reason differs
//! from the run over the server's own ports, which the gate holds to 0 failed, 0 error and exactly
//! the unsupported scenarios the allowance file names.
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

/// The view answers every configuration of the task, whatever tenant it is read under (0.5.0).
const VIEW_UNSCOPED: u8 = 0;
/// The view read without a tenant answers every configuration of the task.
const VIEW_ABSENT_READS_ALL: u8 = 1;
/// The view read under a tenant answers every configuration of the task.
const VIEW_TENANT_READS_ALL: u8 = 2;
/// A configuration is stored without the tenant it was created under.
const CREATE_DROPS_TENANT: u8 = 3;
/// A configuration is stored under a tenant other than the one it was created under.
const CREATE_STORES_ANOTHER_TENANT: u8 = 4;

/// The server's ports, every obligation kept as they keep it, except the defect `D` names.
struct Planted<const D: u8>(ServerPorts);

impl<const D: u8> Planted<D> {
    /// Every stored configuration of `task`, as a row of the view, whatever its tenant.
    fn every_row_of(&self, task: &TaskId) -> Vec<TaskPushNotificationConfigs> {
        PushNotificationConfigStorage::list(&self.0)
            .into_iter()
            .filter(|snapshot| snapshot.data.task_id == *task)
            .map(|snapshot| TaskPushNotificationConfigs {
                config_id: snapshot.data.config_id,
                task_id: snapshot.data.task_id,
                tenant: snapshot.data.tenant,
                url: snapshot.data.url,
                token: snapshot.data.token,
                authentication: snapshot.data.authentication,
            })
            .collect()
    }
}

impl<const D: u8> TaskPushNotificationConfigsQuery for Planted<D> {
    fn task_push_notification_configs(
        &self,
        task_id: TaskId,
        tenant: Option<String>,
    ) -> Result<Vec<TaskPushNotificationConfigs>, UnmetObligation> {
        let reads_all = match D {
            VIEW_UNSCOPED => true,
            VIEW_ABSENT_READS_ALL => tenant.is_none(),
            VIEW_TENANT_READS_ALL => tenant.is_some(),
            _ => false,
        };
        if reads_all {
            return Ok(self.every_row_of(&task_id));
        }
        self.0.task_push_notification_configs(task_id, tenant)
    }
}

impl<const D: u8> CreateTaskPushNotificationConfigBehavior for Planted<D> {
    fn create_task_push_notification_config(
        &mut self,
        input: CreateTaskPushNotificationConfig,
    ) -> Result<CreateTaskPushNotificationConfigOutcome, UnmetObligation> {
        let outcome = self.0.create_task_push_notification_config(input)?;
        let stored = match D {
            CREATE_DROPS_TENANT => Some(None),
            CREATE_STORES_ANOTHER_TENANT => Some(Some("tenant-elsewhere".to_owned())),
            _ => None,
        };
        if let Some(tenant) = stored {
            let row = match &outcome {
                CreateTaskPushNotificationConfigOutcome::Created {
                    push_notification_config_created,
                    ..
                }
                | CreateTaskPushNotificationConfigOutcome::Assigned {
                    push_notification_config_created,
                    ..
                } => Some(push_notification_config_created.row.clone()),
                _ => None,
            };
            if let Some(row) = row
                && let Some(mut snapshot) = PushNotificationConfigStorage::get(&self.0, &row)
            {
                snapshot.data.tenant = tenant;
                PushNotificationConfigStorage::put(&mut self.0, snapshot);
            }
        }
        Ok(outcome)
    }
}

impl<const D: u8> GetTaskPushNotificationConfigBehavior for Planted<D> {
    fn get_task_push_notification_config(
        &mut self,
        input: GetTaskPushNotificationConfig,
    ) -> Result<GetTaskPushNotificationConfigOutcome, UnmetObligation> {
        self.0.get_task_push_notification_config(input)
    }
}

impl<const D: u8> DeleteTaskPushNotificationConfigBehavior for Planted<D> {
    fn delete_task_push_notification_config(
        &mut self,
        input: DeleteTaskPushNotificationConfig,
    ) -> Result<DeleteTaskPushNotificationConfigOutcome, UnmetObligation> {
        self.0.delete_task_push_notification_config(input)
    }
}

impl<const D: u8> ListTaskPushNotificationConfigsBehavior for Planted<D> {
    fn list_task_push_notification_configs(
        &mut self,
        input: ListTaskPushNotificationConfigs,
    ) -> Result<ListTaskPushNotificationConfigsOutcome, UnmetObligation> {
        self.0.list_task_push_notification_configs(input)
    }
}

impl<const D: u8> Context for Planted<D> {
    fn generate_optional_timestamp(&mut self) -> Option<Timestamp> {
        self.0.generate_optional_timestamp()
    }
}

impl<const D: u8> TaskStorage for Planted<D> {
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

impl<const D: u8> PushNotificationConfigStorage for Planted<D> {
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

impl<const D: u8> GetExtendedAgentCardBehavior for Planted<D> {
    fn get_extended_agent_card(
        &mut self,
        input: GetExtendedAgentCard,
    ) -> Result<GetExtendedAgentCardOutcome, UnmetObligation> {
        self.0.get_extended_agent_card(input)
    }
}

impl<const D: u8> CancelTaskBehavior for Planted<D> {
    fn cancel_task(&mut self, input: CancelTask) -> Result<CancelTaskOutcome, UnmetObligation> {
        self.0.cancel_task(input)
    }
}

impl<const D: u8> GetTaskBehavior for Planted<D> {
    fn get_task(&mut self, input: GetTask) -> Result<GetTaskOutcome, UnmetObligation> {
        self.0.get_task(input)
    }
}

impl<const D: u8> SendMessageBehavior for Planted<D> {
    fn send_message(&mut self, input: SendMessage) -> Result<SendMessageOutcome, UnmetObligation> {
        self.0.send_message(input)
    }
}

impl<const D: u8> SendStreamingMessageBehavior for Planted<D> {
    fn send_streaming_message(
        &mut self,
        input: SendStreamingMessage,
    ) -> Result<SendStreamingMessageOutcome, UnmetObligation> {
        self.0.send_streaming_message(input)
    }
}

impl<const D: u8> SubscribeToTaskBehavior for Planted<D> {
    fn subscribe_to_task(
        &mut self,
        input: SubscribeToTask,
    ) -> Result<SubscribeToTaskOutcome, UnmetObligation> {
        self.0.subscribe_to_task(input)
    }
}

impl<const D: u8> TasksQuery for Planted<D> {
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

/// The suite of `ess/`, synthesized once with the `ess` on `PATH` into this target's temporary
/// directory.
fn suite() -> Suite {
    static SYNTHESIZED: OnceLock<PathBuf> = OnceLock::new();
    let path = SYNTHESIZED.get_or_init(|| {
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sec-w5-view-gate");
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

/// Asserts that the run over the planted ports comes out otherwise than the run over the
/// server's own ports in at least one scenario, so the gate that holds the latter turns red.
fn the_gate_sees(defect: &str, planted: &Results) {
    let suite = suite();
    let kept = outcomes(&run(&suite, &mut ServerTarget::new()));
    for (id, (status, message)) in &kept {
        assert!(
            !matches!(status, Status::Failed | Status::Error),
            "{id} is {status:?} over the server's own ports: {message:?}"
        );
    }
    let planted = outcomes(planted);
    let changed: Vec<(&String, &Status)> = kept
        .iter()
        .filter(|(id, kept)| planted.get(*id) != Some(kept))
        .map(|(id, _)| (id, &planted[id].0))
        .collect();
    println!("{defect} changes: {changed:?}");
    assert!(
        !changed.is_empty(),
        "{defect}: every scenario comes out as over the server's own ports, so the conformance \
         gate stays green"
    );
}

#[test]
fn sec_w5_the_gate_turns_red_when_the_view_ignores_its_tenant() {
    let results = run(
        &suite(),
        &mut ServerTarget::wrapped(Planted::<VIEW_UNSCOPED>),
    );
    the_gate_sees("the view answering every tenant's configurations", &results);
}

#[test]
fn sec_w5_the_gate_turns_red_when_the_view_read_without_a_tenant_reads_every_tenant() {
    let results = run(
        &suite(),
        &mut ServerTarget::wrapped(Planted::<VIEW_ABSENT_READS_ALL>),
    );
    the_gate_sees(
        "the view read without a tenant answering every configuration",
        &results,
    );
}

#[test]
fn sec_w5_the_gate_turns_red_when_the_view_read_under_a_tenant_reads_every_tenant() {
    let results = run(
        &suite(),
        &mut ServerTarget::wrapped(Planted::<VIEW_TENANT_READS_ALL>),
    );
    the_gate_sees(
        "the view read under a tenant answering every configuration",
        &results,
    );
}

#[test]
fn sec_w5_the_gate_turns_red_when_a_configuration_drops_its_tenant() {
    let results = run(
        &suite(),
        &mut ServerTarget::wrapped(Planted::<CREATE_DROPS_TENANT>),
    );
    the_gate_sees("a configuration stored without its tenant", &results);
}

#[test]
fn sec_w5_the_gate_turns_red_when_a_configuration_is_stored_under_another_tenant() {
    let results = run(
        &suite(),
        &mut ServerTarget::wrapped(Planted::<CREATE_STORES_ANOTHER_TENANT>),
    );
    the_gate_sees("a configuration stored under another tenant", &results);
}
