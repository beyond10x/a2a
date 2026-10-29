//! Adversary pass 1 on story:view-tenant-scope: the conformance gate over `a2a-server`'s own
//! ports, with the `TaskPushNotificationConfigs` view's tenant filter broken one way at a time.
//!
//! The view's filter (`ess/domains/tasks.yaml`) has two branches: a configuration made under a
//! tenant is a row only of a read under that tenant, and one made without a tenant only of a read
//! without one. Each mutant keeps every obligation as the server keeps it except the view, and a
//! case is red while the gate (`unsupported.tsv`, 0 failed, 0 error) lets the mutant through. The
//! suite is synthesized with the `ess` on `PATH`, as `task conform` synthesizes it.

use std::path::Path;
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
use a2a_conformance::{Results, ServerTarget, Status, Suite, gate, run};
use a2a_server::ServerPorts;

/// The server's own ports.
const NONE: u8 = 0;
/// A read without a tenant answers no row: a tenant-less client never reads its own configuration.
const ABSENT_BLIND: u8 = 1;
/// The view answers the task's configurations whatever tenant it is read under.
const TENANT_IGNORED: u8 = 2;
/// A read without a tenant answers the task's configurations of any tenant.
const ABSENT_WILDCARD: u8 = 3;
/// A read under a tenant answers no row: only tenant-less configurations are ever rows.
const DEFINED_BLIND: u8 = 4;

/// The server's ports with the view's tenant filter broken as `K` names.
struct Mutant<const K: u8>(ServerPorts);

impl<const K: u8> Mutant<K> {
    /// The tenant of the stored task `task`, which every configuration of it was created under.
    fn owner(&self, task: &TaskId) -> Option<Option<String>> {
        TaskStorage::get(&self.0, task).map(|held| held.data.tenant)
    }
}

impl<const K: u8> TaskPushNotificationConfigsQuery for Mutant<K> {
    fn task_push_notification_configs(
        &self,
        task_id: TaskId,
        tenant: Option<String>,
    ) -> Result<Vec<TaskPushNotificationConfigs>, UnmetObligation> {
        let owner = self.owner(&task_id);
        match K {
            ABSENT_BLIND if tenant.is_none() => Ok(Vec::new()),
            DEFINED_BLIND if tenant.is_some() => Ok(Vec::new()),
            TENANT_IGNORED => {
                let tenant = owner.unwrap_or(tenant);
                self.0.task_push_notification_configs(task_id, tenant)
            }
            ABSENT_WILDCARD if tenant.is_none() => {
                let tenant = owner.unwrap_or(tenant);
                self.0.task_push_notification_configs(task_id, tenant)
            }
            _ => self.0.task_push_notification_configs(task_id, tenant),
        }
    }
}

impl<const K: u8> TasksQuery for Mutant<K> {
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

impl<const K: u8> GetTaskBehavior for Mutant<K> {
    fn get_task(&mut self, input: GetTask) -> Result<GetTaskOutcome, UnmetObligation> {
        self.0.get_task(input)
    }
}

impl<const K: u8> ListTaskPushNotificationConfigsBehavior for Mutant<K> {
    fn list_task_push_notification_configs(
        &mut self,
        input: ListTaskPushNotificationConfigs,
    ) -> Result<ListTaskPushNotificationConfigsOutcome, UnmetObligation> {
        self.0.list_task_push_notification_configs(input)
    }
}

impl<const K: u8> CancelTaskBehavior for Mutant<K> {
    fn cancel_task(&mut self, input: CancelTask) -> Result<CancelTaskOutcome, UnmetObligation> {
        self.0.cancel_task(input)
    }
}

impl<const K: u8> Context for Mutant<K> {
    fn generate_optional_timestamp(&mut self) -> Option<Timestamp> {
        self.0.generate_optional_timestamp()
    }
}

impl<const K: u8> TaskStorage for Mutant<K> {
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

impl<const K: u8> PushNotificationConfigStorage for Mutant<K> {
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

impl<const K: u8> GetExtendedAgentCardBehavior for Mutant<K> {
    fn get_extended_agent_card(
        &mut self,
        input: GetExtendedAgentCard,
    ) -> Result<GetExtendedAgentCardOutcome, UnmetObligation> {
        self.0.get_extended_agent_card(input)
    }
}

impl<const K: u8> CreateTaskPushNotificationConfigBehavior for Mutant<K> {
    fn create_task_push_notification_config(
        &mut self,
        input: CreateTaskPushNotificationConfig,
    ) -> Result<CreateTaskPushNotificationConfigOutcome, UnmetObligation> {
        self.0.create_task_push_notification_config(input)
    }
}

impl<const K: u8> DeleteTaskPushNotificationConfigBehavior for Mutant<K> {
    fn delete_task_push_notification_config(
        &mut self,
        input: DeleteTaskPushNotificationConfig,
    ) -> Result<DeleteTaskPushNotificationConfigOutcome, UnmetObligation> {
        self.0.delete_task_push_notification_config(input)
    }
}

impl<const K: u8> GetTaskPushNotificationConfigBehavior for Mutant<K> {
    fn get_task_push_notification_config(
        &mut self,
        input: GetTaskPushNotificationConfig,
    ) -> Result<GetTaskPushNotificationConfigOutcome, UnmetObligation> {
        self.0.get_task_push_notification_config(input)
    }
}

impl<const K: u8> SendMessageBehavior for Mutant<K> {
    fn send_message(&mut self, input: SendMessage) -> Result<SendMessageOutcome, UnmetObligation> {
        self.0.send_message(input)
    }
}

impl<const K: u8> SendStreamingMessageBehavior for Mutant<K> {
    fn send_streaming_message(
        &mut self,
        input: SendStreamingMessage,
    ) -> Result<SendStreamingMessageOutcome, UnmetObligation> {
        self.0.send_streaming_message(input)
    }
}

impl<const K: u8> SubscribeToTaskBehavior for Mutant<K> {
    fn subscribe_to_task(
        &mut self,
        input: SubscribeToTask,
    ) -> Result<SubscribeToTaskOutcome, UnmetObligation> {
        self.0.subscribe_to_task(input)
    }
}

/// The suite of `ess/`, synthesized once with the `ess` on `PATH`.
fn suite() -> &'static Suite {
    static SUITE: OnceLock<Suite> = OnceLock::new();
    SUITE.get_or_init(|| {
        let specification = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ess");
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join("adv-w5-view-tenant");
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
        Suite::parse(&std::fs::read_to_string(&suite).expect("the suite was written"))
            .expect("the synthesized suite reads")
    })
}

/// The `ess-conformance-report/2` counts and outcomes `ess verify conform report` writes for
/// `results`.
fn report_of(results: &Results) -> String {
    let ids = |status: Status| {
        let quoted: Vec<String> = results
            .results
            .iter()
            .filter(|result| result.status == status)
            .map(|result| format!("\"{}\"", result.scenario_id))
            .collect();
        format!("[{}]", quoted.join(","))
    };
    format!(
        r#"{{"format":"ess-conformance-report/2","conformance_status":"failed","counts":{{"error":{},"failed":{},"passed":{},"skipped":0,"total":{},"unsupported":{}}},"outcomes":{{"error":{},"failed":{},"passed":{},"skipped":[],"unsupported":{}}}}}"#,
        results.count(Status::Error),
        results.count(Status::Failed),
        results.count(Status::Passed),
        results.results.len(),
        results.count(Status::Unsupported),
        ids(Status::Error),
        ids(Status::Failed),
        ids(Status::Passed),
        ids(Status::Unsupported),
    )
}

const ALLOWANCES: &str = include_str!("../unsupported.tsv");

/// What the gate says of the suite run against `target`, held to `unsupported.tsv`.
fn gate_over<P: a2a_server::Obligations>(target: &mut ServerTarget<P>) -> Result<String, String> {
    let results = run(suite(), target);
    gate::hold(&report_of(&results), &results, ALLOWANCES)
}

fn caught<P: a2a_server::Obligations>(mutant: &str, mut target: ServerTarget<P>) {
    let held = gate_over(&mut target);
    assert!(
        held.is_err(),
        "story:view-tenant-scope: the gate let a2a-server through with {mutant}: {held:?}"
    );
}

/// The control: the server's own ports pass the gate this file holds mutants to.
#[test]
fn adv_w5_the_servers_own_ports_pass_the_gate() {
    let held = gate_over(&mut ServerTarget::wrapped(Mutant::<NONE>));
    assert!(held.is_ok(), "{held:?}");
}

/// The filter's second branch (`not defined(param.tenant)`, `not defined(tenant)`): a
/// configuration made without a tenant is a row of a read without one. No scenario reads the view
/// without a tenant and expects a row, so a view that never answers a tenant-less client passes.
#[test]
fn adv_w5_a_view_that_answers_a_tenant_less_read_no_row_is_caught() {
    caught(
        "a TaskPushNotificationConfigs view that answers a read without a tenant no row",
        ServerTarget::wrapped(Mutant::<ABSENT_BLIND>),
    );
}

#[test]
fn adv_w5_a_view_that_ignores_its_tenant_is_caught() {
    caught(
        "a TaskPushNotificationConfigs view that ignores `tenant`",
        ServerTarget::wrapped(Mutant::<TENANT_IGNORED>),
    );
}

#[test]
fn adv_w5_a_view_that_answers_a_tenant_less_read_every_tenants_rows_is_caught() {
    caught(
        "a TaskPushNotificationConfigs view that answers a read without a tenant every row",
        ServerTarget::wrapped(Mutant::<ABSENT_WILDCARD>),
    );
}

#[test]
fn adv_w5_a_view_that_answers_a_tenant_no_row_is_caught() {
    caught(
        "a TaskPushNotificationConfigs view that answers a read under a tenant no row",
        ServerTarget::wrapped(Mutant::<DEFINED_BLIND>),
    );
}
