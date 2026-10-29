//! Adversary cases on the gate over `a2a-server`'s own ports: obligations broken in ways the
//! synthesized suite, run through the gate with `unsupported.tsv`, does not notice.
//!
//! Each mutant is the server's ports with one obligation kept otherwise ([`Mutant`]); the suite
//! is synthesized with the `ess` on `PATH`, as `task conform` and `planted_defect` do. A case is
//! red while the gate lets the mutant through.

use std::collections::BTreeSet;
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
/// The `TaskPushNotificationConfigs` view answers no rows.
const CONFIGS_VIEW_EMPTY: u8 = 1;
/// The `Tasks` view answers the tasks of every tenant, not only the reader's.
const TASKS_ACROSS_TENANTS: u8 = 2;
/// `GetTask` answers the whole history whatever `history_length` asks for.
const HISTORY_LENGTH_IGNORED: u8 = 3;
/// The `Tasks` view answers tasks in every state whatever `status` asks for.
const TASKS_STATUS_IGNORED: u8 = 4;
/// `ListTaskPushNotificationConfigs` answers every configuration whatever `page_size` asks for.
const CONFIG_PAGE_SIZE_IGNORED: u8 = 5;

/// The server's ports with the obligation `K` names kept otherwise.
struct Mutant<const K: u8>(ServerPorts);

impl<const K: u8> TaskPushNotificationConfigsQuery for Mutant<K> {
    fn task_push_notification_configs(
        &self,
        task_id: TaskId,
        tenant: Option<String>,
    ) -> Result<Vec<TaskPushNotificationConfigs>, UnmetObligation> {
        if K == CONFIGS_VIEW_EMPTY {
            return Ok(Vec::new());
        }
        self.0.task_push_notification_configs(task_id, tenant)
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
        match K {
            TASKS_ACROSS_TENANTS => {
                let tenants: BTreeSet<Option<String>> = TaskStorage::list(&self.0)
                    .into_iter()
                    .map(|held| held.data.tenant)
                    .collect();
                let mut rows = Vec::new();
                for tenant in tenants {
                    rows.extend(self.0.tasks(
                        context_id.clone(),
                        status,
                        status_timestamp_after.clone(),
                        tenant,
                    )?);
                }
                Ok(rows)
            }
            TASKS_STATUS_IGNORED => self
                .0
                .tasks(context_id, None, status_timestamp_after, tenant),
            _ => self
                .0
                .tasks(context_id, status, status_timestamp_after, tenant),
        }
    }
}

impl<const K: u8> GetTaskBehavior for Mutant<K> {
    fn get_task(&mut self, input: GetTask) -> Result<GetTaskOutcome, UnmetObligation> {
        if K == HISTORY_LENGTH_IGNORED {
            return self.0.get_task(GetTask {
                history_length: None,
                ..input
            });
        }
        self.0.get_task(input)
    }
}

impl<const K: u8> ListTaskPushNotificationConfigsBehavior for Mutant<K> {
    fn list_task_push_notification_configs(
        &mut self,
        input: ListTaskPushNotificationConfigs,
    ) -> Result<ListTaskPushNotificationConfigsOutcome, UnmetObligation> {
        if K == CONFIG_PAGE_SIZE_IGNORED {
            return self
                .0
                .list_task_push_notification_configs(ListTaskPushNotificationConfigs {
                    page_size: None,
                    ..input
                });
        }
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
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join("adv-server");
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
        "the gate let a2a-server through with {mutant}: {held:?}"
    );
}

/// The control: the server's own ports pass the gate this file holds mutants to.
#[test]
fn the_servers_own_ports_pass_the_gate() {
    let held = gate_over(&mut ServerTarget::wrapped(Mutant::<NONE>));
    assert!(held.is_ok(), "{held:?}");
}

/// Four scenarios read the `TaskPushNotificationConfigs` view: CreateTaskPushNotificationConfig
/// `outcome/assigned` and `outcome/created`, and the view's `grant/read/admitted` and
/// `grant/read/denied`. `ACROSS_TENANTS` (src/server.rs) makes all four `unsupported` at a
/// CreateTaskPushNotificationConfig that synthesis sends under another tenant, before any of them
/// reads the view. So a view that answers no rows, or rows without their token and
/// authentication, passes the gate.
#[test]
fn a_push_config_view_that_answers_no_rows_is_caught() {
    caught(
        "a TaskPushNotificationConfigs view that answers no rows",
        ServerTarget::wrapped(Mutant::<CONFIGS_VIEW_EMPTY>),
    );
}

/// No scenario reads the `Tasks` view (ListTasks) under one tenant after a task was created
/// under another, so a view that answers every tenant's tasks passes the gate (A2A v1.0.1
/// section 13.1).
#[test]
fn a_tasks_view_that_answers_other_tenants_tasks_is_caught() {
    caught(
        "a Tasks view that answers the tasks of every tenant",
        ServerTarget::wrapped(Mutant::<TASKS_ACROSS_TENANTS>),
    );
}

/// No scenario reads the `Tasks` view with a `status` that a held task is not in.
#[test]
fn a_tasks_view_that_ignores_its_status_filter_is_caught() {
    caught(
        "a Tasks view that ignores `status`",
        ServerTarget::wrapped(Mutant::<TASKS_STATUS_IGNORED>),
    );
}

/// No scenario sends GetTask a `history_length` shorter than the task's history.
#[test]
fn a_get_task_that_ignores_history_length_is_caught() {
    caught(
        "a GetTask that ignores `history_length`",
        ServerTarget::wrapped(Mutant::<HISTORY_LENGTH_IGNORED>),
    );
}

/// No scenario lists a task's configurations with a `page_size` smaller than their number.
#[test]
fn a_config_list_that_ignores_page_size_is_caught() {
    caught(
        "a ListTaskPushNotificationConfigs that ignores `page_size`",
        ServerTarget::wrapped(Mutant::<CONFIG_PAGE_SIZE_IGNORED>),
    );
}
