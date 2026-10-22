// generated from a2a v1
// model digest d675090b4779d71e3671099ac4d5efbaa17d7a58ae4e0ddaa6cafd8d5a23f002
// contract digest 3dea9905e0cd94c1255be15d5619996d104b54b00e2b8097da86f0bf80643b7f
// do not edit: regenerate with `ess synthesize --layout crate`

//! Ephemeral stores for the generated network entry points; no durability.

/// Ephemeral storage of `a2a.tasks.PushNotificationConfig`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryPushNotificationConfigStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::tasks::PushNotificationConfigSnapshot>>>);
impl crate::behaviour::PushNotificationConfigStorage for MemoryPushNotificationConfigStorage {
fn get(&self, identity: &crate::tasks::PushNotificationConfigRow) -> Option<crate::tasks::PushNotificationConfigSnapshot> { let key = memory_key_28(identity); self.0.borrow().iter().find(|row| memory_key_28(&row.data.row) == key).cloned() }
fn put(&mut self, snapshot: crate::tasks::PushNotificationConfigSnapshot) { self.delete(&snapshot.data.row); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_28(&row.data.row).cmp(&memory_key_28(&other.data.row))); }
fn delete(&mut self, identity: &crate::tasks::PushNotificationConfigRow) { let key = memory_key_28(identity); self.0.borrow_mut().retain(|row| memory_key_28(&row.data.row) != key); }
fn list(&self) -> Vec<crate::tasks::PushNotificationConfigSnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::PushNotificationConfigStorage for MemoryPorts {
fn get(&self, identity: &crate::tasks::PushNotificationConfigRow) -> Option<crate::tasks::PushNotificationConfigSnapshot> { crate::behaviour::PushNotificationConfigStorage::get(&self.push_notification_config_storage, identity) }
fn put(&mut self, snapshot: crate::tasks::PushNotificationConfigSnapshot) { crate::behaviour::PushNotificationConfigStorage::put(&mut self.push_notification_config_storage, snapshot); }
fn delete(&mut self, identity: &crate::tasks::PushNotificationConfigRow) { crate::behaviour::PushNotificationConfigStorage::delete(&mut self.push_notification_config_storage, identity); }
fn list(&self) -> Vec<crate::tasks::PushNotificationConfigSnapshot> { crate::behaviour::PushNotificationConfigStorage::list(&self.push_notification_config_storage) }
}

/// Ephemeral storage of `a2a.tasks.Task`, shared by clones.
#[derive(Clone, Default)]
pub struct MemoryTaskStorage(std::rc::Rc<std::cell::RefCell<Vec<crate::tasks::TaskEntitySnapshot>>>);
impl crate::behaviour::TaskStorage for MemoryTaskStorage {
fn get(&self, identity: &crate::tasks::TaskId) -> Option<crate::tasks::TaskEntitySnapshot> { let key = memory_key_33(identity); self.0.borrow().iter().find(|row| memory_key_33(&row.data.id) == key).cloned() }
fn put(&mut self, snapshot: crate::tasks::TaskEntitySnapshot) { self.delete(&snapshot.data.id); let mut rows = self.0.borrow_mut(); rows.push(snapshot); rows.sort_by(|row, other| memory_key_33(&row.data.id).cmp(&memory_key_33(&other.data.id))); }
fn delete(&mut self, identity: &crate::tasks::TaskId) { let key = memory_key_33(identity); self.0.borrow_mut().retain(|row| memory_key_33(&row.data.id) != key); }
fn list(&self) -> Vec<crate::tasks::TaskEntitySnapshot> { self.0.borrow().clone() }
}
impl crate::behaviour::TaskStorage for MemoryPorts {
fn get(&self, identity: &crate::tasks::TaskId) -> Option<crate::tasks::TaskEntitySnapshot> { crate::behaviour::TaskStorage::get(&self.task_storage, identity) }
fn put(&mut self, snapshot: crate::tasks::TaskEntitySnapshot) { crate::behaviour::TaskStorage::put(&mut self.task_storage, snapshot); }
fn delete(&mut self, identity: &crate::tasks::TaskId) { crate::behaviour::TaskStorage::delete(&mut self.task_storage, identity); }
fn list(&self) -> Vec<crate::tasks::TaskEntitySnapshot> { crate::behaviour::TaskStorage::list(&self.task_storage) }
}

/// Ports shared by generated components. Values disappear when the process exits.
#[derive(Clone, Default)]
pub struct MemoryPorts {
/// Storage of `a2a.tasks.PushNotificationConfig`.
pub push_notification_config_storage: MemoryPushNotificationConfigStorage,
/// Storage of `a2a.tasks.Task`.
pub task_storage: MemoryTaskStorage,
}

impl crate::behaviour::TryContext for MemoryPorts {
fn try_generate_optional_timestamp(&mut self) -> Result<Option<crate::primitives::Timestamp>, crate::obligation::UnmetObligation> { Ok(None) }
}

fn memory_key_28(value: &crate::tasks::PushNotificationConfigRow) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

fn memory_key_33(value: &crate::tasks::TaskId) -> MemoryKey { let _ = value; MemoryKey::Text((&value.0).clone()) }

// A common structural order; individual specifications use only a subset of the variants.
#[allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum MemoryKey {
    Null,
    Boolean(bool),
    Integer(i64),
    Number(String),
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<MemoryKey>),
    Map(Vec<(MemoryKey, MemoryKey)>),
    Optional(Option<Box<MemoryKey>>),
}

impl crate::discovery::obligations::GetExtendedAgentCardBehavior for MemoryPorts {
fn get_extended_agent_card(&mut self, input: crate::discovery::GetExtendedAgentCard) -> Result<crate::discovery::GetExtendedAgentCardOutcome, crate::obligation::UnmetObligation> { crate::discovery::obligations::GetExtendedAgentCardBehavior::get_extended_agent_card(&mut crate::discovery::obligations::Unimplemented, input) }
}
impl crate::tasks::obligations::CancelTaskBehavior for MemoryPorts {
fn cancel_task(&mut self, input: crate::tasks::CancelTask) -> Result<crate::tasks::CancelTaskOutcome, crate::obligation::UnmetObligation> { crate::tasks::obligations::CancelTaskBehavior::cancel_task(&mut crate::tasks::obligations::Unimplemented, input) }
}
impl crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior for MemoryPorts {
fn create_task_push_notification_config(&mut self, input: crate::tasks::CreateTaskPushNotificationConfig) -> Result<crate::tasks::CreateTaskPushNotificationConfigOutcome, crate::obligation::UnmetObligation> { crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior::create_task_push_notification_config(&mut crate::tasks::obligations::Unimplemented, input) }
}
impl crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior for MemoryPorts {
fn delete_task_push_notification_config(&mut self, input: crate::tasks::DeleteTaskPushNotificationConfig) -> Result<crate::tasks::DeleteTaskPushNotificationConfigOutcome, crate::obligation::UnmetObligation> { crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior::delete_task_push_notification_config(&mut crate::tasks::obligations::Unimplemented, input) }
}
impl crate::tasks::obligations::GetTaskBehavior for MemoryPorts {
fn get_task(&mut self, input: crate::tasks::GetTask) -> Result<crate::tasks::GetTaskOutcome, crate::obligation::UnmetObligation> { crate::tasks::obligations::GetTaskBehavior::get_task(&mut crate::tasks::obligations::Unimplemented, input) }
}
impl crate::tasks::obligations::GetTaskPushNotificationConfigBehavior for MemoryPorts {
fn get_task_push_notification_config(&mut self, input: crate::tasks::GetTaskPushNotificationConfig) -> Result<crate::tasks::GetTaskPushNotificationConfigOutcome, crate::obligation::UnmetObligation> { crate::tasks::obligations::GetTaskPushNotificationConfigBehavior::get_task_push_notification_config(&mut crate::tasks::obligations::Unimplemented, input) }
}
impl crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior for MemoryPorts {
fn list_task_push_notification_configs(&mut self, input: crate::tasks::ListTaskPushNotificationConfigs) -> Result<crate::tasks::ListTaskPushNotificationConfigsOutcome, crate::obligation::UnmetObligation> { crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior::list_task_push_notification_configs(&mut crate::tasks::obligations::Unimplemented, input) }
}
impl crate::tasks::obligations::SendMessageBehavior for MemoryPorts {
fn send_message(&mut self, input: crate::tasks::SendMessage) -> Result<crate::tasks::SendMessageOutcome, crate::obligation::UnmetObligation> { crate::tasks::obligations::SendMessageBehavior::send_message(&mut crate::tasks::obligations::Unimplemented, input) }
}
impl crate::tasks::obligations::SendStreamingMessageBehavior for MemoryPorts {
fn send_streaming_message(&mut self, input: crate::tasks::SendStreamingMessage) -> Result<crate::tasks::SendStreamingMessageOutcome, crate::obligation::UnmetObligation> { crate::tasks::obligations::SendStreamingMessageBehavior::send_streaming_message(&mut crate::tasks::obligations::Unimplemented, input) }
}
impl crate::tasks::obligations::SubscribeToTaskBehavior for MemoryPorts {
fn subscribe_to_task(&mut self, input: crate::tasks::SubscribeToTask) -> Result<crate::tasks::SubscribeToTaskOutcome, crate::obligation::UnmetObligation> { crate::tasks::obligations::SubscribeToTaskBehavior::subscribe_to_task(&mut crate::tasks::obligations::Unimplemented, input) }
}
impl crate::tasks::obligations::TaskPushNotificationConfigsQuery for MemoryPorts {
fn task_push_notification_configs(&self, task_id: crate::tasks::TaskId) -> Result<Vec<crate::tasks::TaskPushNotificationConfigs>, crate::obligation::UnmetObligation> { crate::tasks::obligations::TaskPushNotificationConfigsQuery::task_push_notification_configs(&crate::tasks::obligations::Unimplemented, task_id) }
}
impl crate::tasks::obligations::TasksQuery for MemoryPorts {
fn tasks(&self, context_id: Option<crate::tasks::ContextId>, status: Option<crate::tasks::TaskState>, status_timestamp_after: Option<crate::primitives::Timestamp>) -> Result<Vec<crate::tasks::Tasks>, crate::obligation::UnmetObligation> { crate::tasks::obligations::TasksQuery::tasks(&crate::tasks::obligations::Unimplemented, context_id, status, status_timestamp_after) }
}
