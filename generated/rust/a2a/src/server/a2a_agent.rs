// generated from a2a v1
// model digest d761a3fbbc9413d58b2073e5f15bbced42229623312d9101f245c68b57f5021d
// contract digest 91b9b3965c71b5aeb8e0b091924c0a887fe1b698155e248eea0b77ad3859a42a
// do not edit: regenerate with `ess synthesize --layout crate`

//! The `a2a-agent` component of `a2a` v1, on the wire.
//!
//! The specification says this component's callers are not deployed with it, so its surface
//! exists on a wire. Which wire is derived rather than chosen: the one contract this model
//! projects for a command surface is the `OpenAPI` document, and an `OpenAPI` document is an
//! HTTP contract. The document is beside this file, served verbatim at `/openapi.json`.

use crate::server::{entry, http, json, wire};

/// The contract this surface answers, byte for byte as `generated/` commits it.
///
/// Embedded rather than rebuilt at run time: a server that regenerated its own contract could
/// publish one the repository never reviewed.
pub const OPENAPI: &str = include_str!("a2a-agent.openapi.json");

/// The prose the same model produced, byte for byte as the documentation projection wrote it.
pub const DOCS: &str = include_str!("a2a-agent.docs.md");

/// Every route this surface answers, in path order.
///
/// The same set the `OpenAPI` document declares, plus the two documents about the surface
/// itself, which no specification construct names and nothing can therefore derive. A path
/// absent from this table is answered with `404`, including one the document declares and this
/// table forgot — which is the failure a table computed twice would hide.
pub const ROUTES: &[(&str, &str)] = &[
    ("GET", "/docs"),
    ("GET", "/extendedAgentCard"),
    ("POST", "/message:send"),
    ("POST", "/message:stream"),
    ("GET", "/openapi.json"),
    ("GET", "/tasks"),
    ("POST", "/tasks/commands/tasks:cancel"),
    ("POST", "/tasks/commands/tasks:complete"),
    ("POST", "/tasks/commands/tasks:fail"),
    ("POST", "/tasks/commands/tasks:reject"),
    ("POST", "/tasks/commands/tasks:require_auth"),
    ("POST", "/tasks/commands/tasks:require_input"),
    ("POST", "/tasks/commands/tasks:subscribe"),
    ("POST", "/tasks/commands/tasks:update_artifact"),
    ("POST", "/tasks/commands/tasks:work"),
    ("GET", "/tasks/views/allPushNotificationConfigs"),
    ("GET", "/tasks/views/allTasks"),
    ("GET", "/tasks/{id}"),
    ("GET", "/tasks/{taskId}/pushNotificationConfigs"),
    ("POST", "/tasks/{taskId}/pushNotificationConfigs"),
    ("DELETE", "/tasks/{taskId}/pushNotificationConfigs/{id}"),
    ("GET", "/tasks/{taskId}/pushNotificationConfigs/{id}"),
    ("GET", "/views/tasks/pushNotificationConfigs"),
];

/// What this process says about itself as it starts, before it answers anything.
///
/// Three lines of JSON on standard output, in this order, every member of them derived from the
/// specification — except `runtime`, which is appended by the emitted code below and holds what
/// is true of *this process*: the language it was synthesised into, and the address it bound.
/// Everything outside `runtime` is the same in every language this plan is emitted into, and
/// `cargo xtask synth --check` starts both and compares them.
pub const STARTUP: &[&str] = &[
    "{\"log\":\"ess/1\",\"event\":\"system.starting\",\"system\":\"a2a\",\"version\":\"v1\",\"model_digest\":\"d761a3fbbc9413d58b2073e5f15bbced42229623312d9101f245c68b57f5021d\",\"contract_digest\":\"91b9b3965c71b5aeb8e0b091924c0a887fe1b698155e248eea0b77ad3859a42a\",\"components\":[\"a2a-agent\"],\"capabilities\":{\"generated\":99,\"obligations\":12,\"refused\":0}",
    "{\"log\":\"ess/1\",\"event\":\"surface.serving\",\"component\":\"a2a-agent\",\"reached_by\":\"network\",\"transport\":\"http/1.1\",\"routes\":23,\"paths\":[{\"method\":\"GET\",\"path\":\"/docs\",\"serves\":\"documentation\",\"name\":\"docs\"},{\"method\":\"GET\",\"path\":\"/extendedAgentCard\",\"serves\":\"command\",\"name\":\"a2a.discovery.GetExtendedAgentCard\"},{\"method\":\"POST\",\"path\":\"/message:send\",\"serves\":\"command\",\"name\":\"a2a.tasks.SendMessage\"},{\"method\":\"POST\",\"path\":\"/message:stream\",\"serves\":\"command\",\"name\":\"a2a.tasks.SendStreamingMessage\"},{\"method\":\"GET\",\"path\":\"/openapi.json\",\"serves\":\"contract\",\"name\":\"openapi\"},{\"method\":\"GET\",\"path\":\"/tasks\",\"serves\":\"view\",\"name\":\"a2a.tasks.Tasks\"},{\"method\":\"POST\",\"path\":\"/tasks/commands/tasks:cancel\",\"serves\":\"command\",\"name\":\"a2a.tasks.CancelTask\"},{\"method\":\"POST\",\"path\":\"/tasks/commands/tasks:complete\",\"serves\":\"command\",\"name\":\"a2a.tasks.CompleteTask\"},{\"method\":\"POST\",\"path\":\"/tasks/commands/tasks:fail\",\"serves\":\"command\",\"name\":\"a2a.tasks.FailTask\"},{\"method\":\"POST\",\"path\":\"/tasks/commands/tasks:reject\",\"serves\":\"command\",\"name\":\"a2a.tasks.RejectTask\"},{\"method\":\"POST\",\"path\":\"/tasks/commands/tasks:require_auth\",\"serves\":\"command\",\"name\":\"a2a.tasks.RequireAuth\"},{\"method\":\"POST\",\"path\":\"/tasks/commands/tasks:require_input\",\"serves\":\"command\",\"name\":\"a2a.tasks.RequireInput\"},{\"method\":\"POST\",\"path\":\"/tasks/commands/tasks:subscribe\",\"serves\":\"command\",\"name\":\"a2a.tasks.SubscribeToTask\"},{\"method\":\"POST\",\"path\":\"/tasks/commands/tasks:update_artifact\",\"serves\":\"command\",\"name\":\"a2a.tasks.UpdateArtifact\"},{\"method\":\"POST\",\"path\":\"/tasks/commands/tasks:work\",\"serves\":\"command\",\"name\":\"a2a.tasks.StartWorking\"},{\"method\":\"GET\",\"path\":\"/tasks/views/allPushNotificationConfigs\",\"serves\":\"view\",\"name\":\"a2a.tasks.PushNotificationConfigs\"},{\"method\":\"GET\",\"path\":\"/tasks/views/allTasks\",\"serves\":\"view\",\"name\":\"a2a.tasks.AllTasks\"},{\"method\":\"GET\",\"path\":\"/tasks/{id}\",\"serves\":\"command\",\"name\":\"a2a.tasks.GetTask\"},{\"method\":\"GET\",\"path\":\"/tasks/{taskId}/pushNotificationConfigs\",\"serves\":\"command\",\"name\":\"a2a.tasks.ListTaskPushNotificationConfigs\"},{\"method\":\"POST\",\"path\":\"/tasks/{taskId}/pushNotificationConfigs\",\"serves\":\"command\",\"name\":\"a2a.tasks.CreateTaskPushNotificationConfig\"},{\"method\":\"DELETE\",\"path\":\"/tasks/{taskId}/pushNotificationConfigs/{id}\",\"serves\":\"command\",\"name\":\"a2a.tasks.DeleteTaskPushNotificationConfig\"},{\"method\":\"GET\",\"path\":\"/tasks/{taskId}/pushNotificationConfigs/{id}\",\"serves\":\"command\",\"name\":\"a2a.tasks.GetTaskPushNotificationConfig\"},{\"method\":\"GET\",\"path\":\"/views/tasks/pushNotificationConfigs\",\"serves\":\"view\",\"name\":\"a2a.tasks.TaskPushNotificationConfigs\"}]",
    "{\"log\":\"ess/1\",\"event\":\"system.ready\",\"system\":\"a2a\",\"surfaces\":1",
];

/// Writes the startup record, with this process's own facts closing each line.
fn announce(address: &std::net::SocketAddr) {
    for facts in STARTUP {
        let mut line = String::from(*facts);
        line.push_str(",\"runtime\":{\"address\":");
        json::push_text(&mut line, &address.to_string());
        line.push_str(",\"language\":\"rust\",\"port\":");
        json::push_integer(&mut line, i64::from(address.port()));
        line.push_str("}}");
        println!("{line}");
    }
}

/// Serves `a2a-agent` at `address`, and does not return while it can answer.
///
/// `address` may name port `0`, which binds an ephemeral port; the startup record says which one
/// was taken, because a caller that cannot learn the port cannot make a request.
///
/// It chooses no realization. Every command reaches the port, and a port over unimplemented
/// obligations answers the typed refusal this surface reports as `501` — the honest empty
/// state rather than a server that pretends.
///
/// `authenticate` is the realization's: it says who each request was sent by, or `None`, and
/// [`dispatch`] checks that caller's grant before the command runs. Nothing here reads an
/// actor from the request itself.
///
/// One connection at a time: each is dropped after [`http::READ_TIMEOUT`] without a byte, or
/// [`http::WRITE_TIMEOUT`] of a stalled write, and whatever fails on one connection — a caller
/// that hung up before reading its answer, a reset, a failed accept — ends that connection only.
///
/// # Errors
///
/// What binding the address refuses: the address is taken, or the port is privileged.
pub fn serve<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, address: &str, authenticate: impl Fn(&http::Request) -> Option<crate::actor::Caller>) -> std::io::Result<()>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{ serve_with_static(system, address, authenticate, None) }

/// Serves the declared surface, with files only for paths outside its route table.
///
/// # Errors
/// Returns a listener or static-root error before announcing readiness.
pub fn serve_with_static<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, address: &str, authenticate: impl Fn(&http::Request) -> Option<crate::actor::Caller>, static_root: Option<&std::path::Path>) -> std::io::Result<()>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let static_root = static_root.map(std::fs::canonicalize).transpose()?;
    if static_root.as_ref().is_some_and(|root| !root.is_dir()) { return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "static root is not a directory")); }
    let listener = std::net::TcpListener::bind(address)?;
    announce(&listener.local_addr()?);
    for connection in listener.incoming() {
        // What fails on one connection ends that connection, never this loop: a caller that
        // gave up before it was accepted, or before it read its answer, is that caller's affair.
        let Ok(connection) = connection else {
            // An accept the listener itself failed (no descriptor left) would fail again at
            // once: pause before the next.
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        };
        let _ = connection.set_read_timeout(Some(http::READ_TIMEOUT));
        let _ = connection.set_write_timeout(Some(http::WRITE_TIMEOUT));
        let mut reader = std::io::BufReader::new(connection);
        let (answer, refused) = match http::read(&mut reader) {
            Ok(request) => {
                if !ROUTES.iter().any(|(_, path)| http::matched(path, &request.path).is_some()) {
                    if let Some(root) = &static_root {
                        let _ = crate::server::static_assets::answer(reader.get_mut(), root, &request);
                        continue;
                    }
                }
                (dispatch(system, authenticate(&request).as_ref(), &request), false)
            },
            Err(refusal) => (refusal, true),
        };
        let mut stream = reader.into_inner();
        if http::write(&mut stream, &answer).is_ok() && refused {
            http::linger(&mut stream);
        }
    }
    Ok(())
}

/// Answers one request.
///
/// The routes are templates an `ess-http/1` binding declares, beside the routes it does not name.
/// A path no route matches is a `404` naming where the whole table is published; a path some
/// route matches under another method is a `405` naming every method its routes answer, in `Allow` too.
///
/// Public so a caller can hand it a request it built itself: [`serve`] is this function behind a
/// socket, and nothing else.
///
/// `caller` is who the realization authenticated the request as, or `None`. Every command
/// checks its grant before it runs.
pub fn dispatch<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, caller: Option<&crate::actor::Caller>, request: &http::Request) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    dispatch_with_events(system, caller, request).0
}

/// Answers one request as [`dispatch`] does, and hands back every event published while it
/// ran, in log order: what the command published, what the bindings that reacted to it
/// published in turn, and what a held-back delivery retried during the call published. An
/// event already on the system's log when the call started is not the call's and is never
/// in the list. The list is empty where no command ran — a refusal before the port, a view,
/// a document or a path no route matches — and holds what was published where delivering
/// it failed after the command took effect. An event a component's port published through a
/// direct call is in that component's outbox until a pump collects it, so one no pump has
/// collected yet is listed as this call's: call `pump` after calling a port directly.
pub fn dispatch_with_events<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, caller: Option<&crate::actor::Caller>, request: &http::Request) -> (http::Response, Vec<crate::system::SystemEvent>)
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let mut taken = Vec::new();
    let answer = route_request(system, caller, request, &mut taken);
    (answer, taken)
}

/// The route match behind [`dispatch_with_events`]: each command pushes what it took off the
/// log onto `taken`.
fn route_request<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, caller: Option<&crate::actor::Caller>, request: &http::Request, taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let mut allowed: Vec<&str> = Vec::new();
    if request.path == "/docs" {
        if request.method == "GET" {
            return http::Response::new(200, http::MARKDOWN, DOCS);
        }
        allowed.push("GET");
    }
    if http::matched("/extendedAgentCard", &request.path).is_some() {
        if request.method == "GET" {
            if let Err(actor) = admit(caller, "a2a.discovery.GetExtendedAgentCard") {
                return refusal_not_granted(actor);
            }
            return serve_bound_a2a_discovery_get_extended_agent_card(system, request, &[], taken);
        }
        allowed.push("GET");
    }
    if http::matched("/message:send", &request.path).is_some() {
        if request.method == "POST" {
            if let Err(actor) = admit(caller, "a2a.tasks.SendMessage") {
                return refusal_not_granted(actor);
            }
            return serve_bound_a2a_tasks_send_message(system, request, &[], taken);
        }
        allowed.push("POST");
    }
    if http::matched("/message:stream", &request.path).is_some() {
        if request.method == "POST" {
            if let Err(actor) = admit(caller, "a2a.tasks.SendStreamingMessage") {
                return refusal_not_granted(actor);
            }
            return serve_bound_a2a_tasks_send_streaming_message(system, request, &[], taken);
        }
        allowed.push("POST");
    }
    if request.path == "/openapi.json" {
        if request.method == "GET" {
            return http::Response::new(200, http::JSON, OPENAPI);
        }
        allowed.push("GET");
    }
    if http::matched("/tasks", &request.path).is_some() {
        if request.method == "GET" {
            if let Err(actor) = admit_read(caller, "a2a.tasks.Tasks") {
                return refusal_not_granted(actor);
            }
            return serve_bound_a2a_tasks_tasks(system, request, &[]);
        }
        allowed.push("GET");
    }
    if request.path == "/tasks/commands/tasks:cancel" {
        if request.method == "POST" {
            if let Err(actor) = admit(caller, "a2a.tasks.CancelTask") {
                return not_granted(actor);
            }
            return serve_a2a_tasks_cancel_task(system, &request.body, taken);
        }
        allowed.push("POST");
    }
    if request.path == "/tasks/commands/tasks:complete" {
        if request.method == "POST" {
            if let Err(actor) = admit(caller, "a2a.tasks.CompleteTask") {
                return not_granted(actor);
            }
            return serve_a2a_tasks_complete_task(system, &request.body, taken);
        }
        allowed.push("POST");
    }
    if request.path == "/tasks/commands/tasks:fail" {
        if request.method == "POST" {
            if let Err(actor) = admit(caller, "a2a.tasks.FailTask") {
                return not_granted(actor);
            }
            return serve_a2a_tasks_fail_task(system, &request.body, taken);
        }
        allowed.push("POST");
    }
    if request.path == "/tasks/commands/tasks:reject" {
        if request.method == "POST" {
            if let Err(actor) = admit(caller, "a2a.tasks.RejectTask") {
                return not_granted(actor);
            }
            return serve_a2a_tasks_reject_task(system, &request.body, taken);
        }
        allowed.push("POST");
    }
    if request.path == "/tasks/commands/tasks:require_auth" {
        if request.method == "POST" {
            if let Err(actor) = admit(caller, "a2a.tasks.RequireAuth") {
                return not_granted(actor);
            }
            return serve_a2a_tasks_require_auth(system, &request.body, taken);
        }
        allowed.push("POST");
    }
    if request.path == "/tasks/commands/tasks:require_input" {
        if request.method == "POST" {
            if let Err(actor) = admit(caller, "a2a.tasks.RequireInput") {
                return not_granted(actor);
            }
            return serve_a2a_tasks_require_input(system, &request.body, taken);
        }
        allowed.push("POST");
    }
    if request.path == "/tasks/commands/tasks:subscribe" {
        if request.method == "POST" {
            if let Err(actor) = admit(caller, "a2a.tasks.SubscribeToTask") {
                return not_granted(actor);
            }
            return serve_a2a_tasks_subscribe_to_task(system, &request.body, taken);
        }
        allowed.push("POST");
    }
    if request.path == "/tasks/commands/tasks:update_artifact" {
        if request.method == "POST" {
            if let Err(actor) = admit(caller, "a2a.tasks.UpdateArtifact") {
                return not_granted(actor);
            }
            return serve_a2a_tasks_update_artifact(system, &request.body, taken);
        }
        allowed.push("POST");
    }
    if request.path == "/tasks/commands/tasks:work" {
        if request.method == "POST" {
            if let Err(actor) = admit(caller, "a2a.tasks.StartWorking") {
                return not_granted(actor);
            }
            return serve_a2a_tasks_start_working(system, &request.body, taken);
        }
        allowed.push("POST");
    }
    if request.path == "/tasks/views/allPushNotificationConfigs" {
        if request.method == "GET" {
            if let Err(actor) = admit_read(caller, "a2a.tasks.PushNotificationConfigs") {
                return not_granted(actor);
            }
            return http::answer(run_a2a_tasks_push_notification_configs(system));
        }
        allowed.push("GET");
    }
    if request.path == "/tasks/views/allTasks" {
        if request.method == "GET" {
            if let Err(actor) = admit_read(caller, "a2a.tasks.AllTasks") {
                return not_granted(actor);
            }
            return http::answer(run_a2a_tasks_all_tasks(system));
        }
        allowed.push("GET");
    }
    if let Some(path) = http::matched("/tasks/{id}", &request.path) {
        if request.method == "GET" {
            if let Err(actor) = admit(caller, "a2a.tasks.GetTask") {
                return refusal_not_granted(actor);
            }
            return serve_bound_a2a_tasks_get_task(system, request, &path, taken);
        }
        allowed.push("GET");
    }
    if let Some(path) = http::matched("/tasks/{taskId}/pushNotificationConfigs", &request.path) {
        if request.method == "GET" {
            if let Err(actor) = admit(caller, "a2a.tasks.ListTaskPushNotificationConfigs") {
                return refusal_not_granted(actor);
            }
            return serve_bound_a2a_tasks_list_task_push_notification_configs(system, request, &path, taken);
        }
        allowed.push("GET");
    }
    if let Some(path) = http::matched("/tasks/{taskId}/pushNotificationConfigs", &request.path) {
        if request.method == "POST" {
            if let Err(actor) = admit(caller, "a2a.tasks.CreateTaskPushNotificationConfig") {
                return refusal_not_granted(actor);
            }
            return serve_bound_a2a_tasks_create_task_push_notification_config(system, request, &path, taken);
        }
        allowed.push("POST");
    }
    if let Some(path) = http::matched("/tasks/{taskId}/pushNotificationConfigs/{id}", &request.path) {
        if request.method == "DELETE" {
            if let Err(actor) = admit(caller, "a2a.tasks.DeleteTaskPushNotificationConfig") {
                return refusal_not_granted(actor);
            }
            return serve_bound_a2a_tasks_delete_task_push_notification_config(system, request, &path, taken);
        }
        allowed.push("DELETE");
    }
    if let Some(path) = http::matched("/tasks/{taskId}/pushNotificationConfigs/{id}", &request.path) {
        if request.method == "GET" {
            if let Err(actor) = admit(caller, "a2a.tasks.GetTaskPushNotificationConfig") {
                return refusal_not_granted(actor);
            }
            return serve_bound_a2a_tasks_get_task_push_notification_config(system, request, &path, taken);
        }
        allowed.push("GET");
    }
    if http::matched("/views/tasks/pushNotificationConfigs", &request.path).is_some() {
        if request.method == "GET" {
            if let Err(actor) = admit_read(caller, "a2a.tasks.TaskPushNotificationConfigs") {
                return refusal_not_granted(actor);
            }
            return serve_bound_a2a_tasks_task_push_notification_configs(system, request, &[]);
        }
        allowed.push("GET");
    }
    if allowed.is_empty() {
        return http::Response::refusal(
            404,
            &format!("`{}` is not a path this surface declares; `GET /openapi.json` publishes every one that is", request.path),
        );
    }
    // RFC 9110 §15.5.6: a `405` names the methods the path answers in `Allow`.
    allowed.sort_unstable();
    allowed.dedup();
    let allow = allowed.join(", ");
    let mut answer = http::method_not_allowed(&allow);
    answer.headers.push(("Allow".to_owned(), allow));
    answer
}

/// Runs one command or view of `a2a-agent` by its qualified name, with no transport.
///
/// The same decoding, refusals and rendering the HTTP routes use — each route and this function call
/// one `run_*` function — so a conformance runner or an in-process caller drives the system
/// without a socket and without a dispatch table of its own. `Ok` is the declared outcome, as the
/// route's body renders it; a view without parameters ignores `input`, as its `GET` route ignores a body, and a view with
/// parameters reads them from `input`, an object keyed by their wire names, as its route reads
/// them from the query string.
///
/// # Errors
///
/// [`entry::Refused::Unknown`] naming `name` when this surface declares no command or view
/// by it; [`entry::Refused::Input`] when `input` is not the command's declared input (the
/// route's `400`); [`entry::Refused::Unmet`] when the port reports an unmet obligation, and
/// [`entry::Refused::Undelivered`] when the command took effect and delivering what it published
/// failed (the route's `501`, with `committed` `false` and `true`).
/// [`entry::Refused::NotGranted`] when `caller` — who the realization authenticated the call
/// as — is none, or is an actor the specification does not grant the command; checked before
/// the command runs (the route's `403`).
/// A view some actor's grant names answers the same refusal, checked before it is read.
///
/// `input` is held to what the route's reader admits: nested no deeper, every number spelled
/// as JSON spells one, or [`entry::Refused::Input`]. The answer is read back at any depth; one holding a
/// `Json` value the realization built with a number JSON does not spell is not JSON, and is
/// [`entry::Refused::Undelivered`] for a command, whose effect stands, and [`entry::Refused::Unmet`]
/// for a view.
pub fn handle<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, caller: Option<&crate::actor::Caller>, name: &str, input: json::Value) -> Result<json::Value, entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    handle_with_events(system, caller, name, input).0
}

/// Runs one command or view as [`handle`] does, and hands back every event published while it
/// ran, in log order: what the command published, what the bindings that reacted to it
/// published in turn, and what a held-back delivery retried during the call published. An
/// event already on the system's log when the call started is not the call's and is never
/// in the list. The list is empty where no command ran — an unknown name, a refusal before
/// the port, a view — and holds what was published where delivering it failed after the
/// command took effect. An event a component's port published through a direct call is in
/// that component's outbox until a pump collects it, so one no pump has collected yet is
/// listed as this call's: call `pump` after calling a port directly.
pub fn handle_with_events<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, caller: Option<&crate::actor::Caller>, name: &str, input: json::Value) -> (Result<json::Value, entry::Refused>, Vec<crate::system::SystemEvent>)
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let mut taken = Vec::new();
    let checked = |input: &json::Value| {
        json::admitted(input).map_err(|expected| {
            entry::Refused::Input(format!("the input is not JSON: expected {expected}"))
        })
    };
    let answered = match name {
        "a2a.discovery.GetExtendedAgentCard" => match admit(caller, "a2a.discovery.GetExtendedAgentCard") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_discovery_get_extended_agent_card(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.AllTasks" => match admit_read(caller, "a2a.tasks.AllTasks") {
            Ok(()) => run_a2a_tasks_all_tasks(system),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.CancelTask" => match admit(caller, "a2a.tasks.CancelTask") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_cancel_task(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.CompleteTask" => match admit(caller, "a2a.tasks.CompleteTask") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_complete_task(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.CreateTaskPushNotificationConfig" => match admit(caller, "a2a.tasks.CreateTaskPushNotificationConfig") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_create_task_push_notification_config(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.DeleteTaskPushNotificationConfig" => match admit(caller, "a2a.tasks.DeleteTaskPushNotificationConfig") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_delete_task_push_notification_config(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.FailTask" => match admit(caller, "a2a.tasks.FailTask") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_fail_task(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.GetTask" => match admit(caller, "a2a.tasks.GetTask") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_get_task(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.GetTaskPushNotificationConfig" => match admit(caller, "a2a.tasks.GetTaskPushNotificationConfig") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_get_task_push_notification_config(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.ListTaskPushNotificationConfigs" => match admit(caller, "a2a.tasks.ListTaskPushNotificationConfigs") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_list_task_push_notification_configs(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.PushNotificationConfigs" => match admit_read(caller, "a2a.tasks.PushNotificationConfigs") {
            Ok(()) => run_a2a_tasks_push_notification_configs(system),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.RejectTask" => match admit(caller, "a2a.tasks.RejectTask") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_reject_task(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.RequireAuth" => match admit(caller, "a2a.tasks.RequireAuth") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_require_auth(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.RequireInput" => match admit(caller, "a2a.tasks.RequireInput") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_require_input(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.SendMessage" => match admit(caller, "a2a.tasks.SendMessage") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_send_message(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.SendStreamingMessage" => match admit(caller, "a2a.tasks.SendStreamingMessage") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_send_streaming_message(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.StartWorking" => match admit(caller, "a2a.tasks.StartWorking") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_start_working(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.SubscribeToTask" => match admit(caller, "a2a.tasks.SubscribeToTask") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_subscribe_to_task(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.TaskPushNotificationConfigs" => match admit_read(caller, "a2a.tasks.TaskPushNotificationConfigs") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_task_push_notification_configs(system, &input)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.Tasks" => match admit_read(caller, "a2a.tasks.Tasks") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_tasks(system, &input)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        "a2a.tasks.UpdateArtifact" => match admit(caller, "a2a.tasks.UpdateArtifact") {
            Ok(()) => checked(&input).and_then(|()| run_a2a_tasks_update_artifact(system, &input, &mut taken)),
            Err(actor) => Err(entry::Refused::NotGranted(actor.map(str::to_owned))),
        },
        other => return (Err(entry::Refused::Unknown(other.to_owned())), taken),
    };
    let body = match answered {
        Ok((_, body)) => body,
        Err(refused) => return (Err(refused), taken),
    };
    let read = entry::read(&body).map_err(|error| {
        if matches!(name, "a2a.discovery.GetExtendedAgentCard" | "a2a.tasks.CancelTask" | "a2a.tasks.CompleteTask" | "a2a.tasks.CreateTaskPushNotificationConfig" | "a2a.tasks.DeleteTaskPushNotificationConfig" | "a2a.tasks.FailTask" | "a2a.tasks.GetTask" | "a2a.tasks.GetTaskPushNotificationConfig" | "a2a.tasks.ListTaskPushNotificationConfigs" | "a2a.tasks.RejectTask" | "a2a.tasks.RequireAuth" | "a2a.tasks.RequireInput" | "a2a.tasks.SendMessage" | "a2a.tasks.SendStreamingMessage" | "a2a.tasks.StartWorking" | "a2a.tasks.SubscribeToTask" | "a2a.tasks.UpdateArtifact") {
            entry::Refused::Undelivered(format!(
                "delivering what the command published: its answer is not JSON this surface reads back: {error}"
            ))
        } else {
            entry::Refused::Unmet(format!(
                "unmet obligation: view_query `{name}` answered rows this surface cannot read back: {error}"
            ))
        }
    });
    (read, taken)
}

/// Nothing, where `caller` may invoke `command`; otherwise the actor the standard refusal names,
/// `None` where the request was authenticated as no actor.
///
/// Checked before the command runs, on the caller the realization authenticated the request
/// as and never on anything the request says about itself. Public, so code that drives the system
/// in process checks the grant exactly as every route does.
pub fn admit(caller: Option<&crate::actor::Caller>, command: &str) -> Result<(), Option<&'static str>> {
    match caller {
        Some(caller) if caller.may(command) => Ok(()),
        Some(caller) => Err(Some(caller.actor.name())),
        None => Err(None),
    }
}

/// The standard refusal for an actor no grant admits, as the contract declares it: `403`,
/// `{"refused": "not granted", "actor": <name or null>}`.
fn not_granted(actor: Option<&str>) -> http::Response {
    let mut body = String::from("{");
    json::member(&mut body, "refused");
    json::push_text(&mut body, "not granted");
    json::member(&mut body, "actor");
    match actor {
        Some(actor) => json::push_text(&mut body, actor),
        None => body.push_str("null"),
    }
    body.push('}');
    http::Response::new(403, http::JSON, body)
}

/// Nothing, where `caller` may read `view`, a view some actor's grant names; otherwise the actor
/// the standard refusal names, `None` where the request was authenticated as no actor.
///
/// Checked before the view is read, on the caller the realization authenticated the request as.
/// Public, so code that reads the system in process checks the grant exactly as every route does.
pub fn admit_read(caller: Option<&crate::actor::Caller>, view: &str) -> Result<(), Option<&'static str>> {
    match caller {
        Some(caller) if caller.may_read(view) => Ok(()),
        Some(caller) => Err(Some(caller.actor.name())),
        None => Err(None),
    }
}

/// `a2a.discovery.GetExtendedAgentCard` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_discovery_get_extended_agent_card<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_discovery_get_extended_agent_card(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.get_extended_agent_card(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_discovery_get_extended_agent_card(&outcome))
}

/// One declared outcome of `a2a.discovery.GetExtendedAgentCard`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_discovery_get_extended_agent_card(outcome: &crate::discovery::GetExtendedAgentCardOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::discovery::GetExtendedAgentCardOutcome::NotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.discovery.UnsupportedOperation");
            502
        }
        crate::discovery::GetExtendedAgentCardOutcome::NotConfigured { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-configured");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.discovery.ExtendedAgentCardNotConfigured");
            502
        }
        crate::discovery::GetExtendedAgentCardOutcome::Returned { response, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "returned");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_discovery_get_extended_agent_card(response, &mut body);
            200
        }
        crate::discovery::GetExtendedAgentCardOutcome::VersionNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "version-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.discovery.VersionNotSupported");
            502
        }
        crate::discovery::GetExtendedAgentCardOutcome::ExtensionRequired { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "extension-required");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.discovery.ExtensionSupportRequired");
            502
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `/extendedAgentCard`: `a2a.discovery.GetExtendedAgentCard`, read and answered as its `ess-http/1` binding declares.
fn serve_bound_a2a_discovery_get_extended_agent_card<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, request: &http::Request, _path: &[String], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let read = || -> Result<json::Value, String> {
        let value = http::flat_object(&request.query, &[("tenant", http::Flat::String, http::Shape::Optional)], &[], true, "query")?;
        http::with_path(value, Vec::new(), "query")
    };
    let value = match read() {
        Ok(value) => value,
        Err(detail) => return refusal_malformed(&detail),
    };
    match run_a2a_discovery_get_extended_agent_card(system, &value, taken) {
        Ok((status, envelope)) => http::Response::new(status, http::JSON, envelope),
        Err(refused) => refused_bound(&refused),
    }
}

/// `a2a.tasks.SendMessage` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_send_message<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_send_message(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.send_message(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_send_message(&outcome))
}

/// One declared outcome of `a2a.tasks.SendMessage`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_send_message(outcome: &crate::tasks::SendMessageOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::SendMessageOutcome::TaskNotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "task-not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            502
        }
        crate::tasks::SendMessageOutcome::TaskTerminal { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "task-terminal");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.UnsupportedOperation");
            502
        }
        crate::tasks::SendMessageOutcome::ContextMismatch { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "context-mismatch");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ContextMismatch");
            502
        }
        crate::tasks::SendMessageOutcome::ContextNotAccepted { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "context-not-accepted");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ContextNotAccepted");
            502
        }
        crate::tasks::SendMessageOutcome::ContentTypeNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "content-type-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ContentTypeNotSupported");
            502
        }
        crate::tasks::SendMessageOutcome::TaskLimit { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "task-limit");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ResourceExhausted");
            502
        }
        crate::tasks::SendMessageOutcome::HistoryLimit { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "history-limit");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ResourceExhausted");
            502
        }
        crate::tasks::SendMessageOutcome::Continued { response, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "continued");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_tasks_send_message(response, &mut body);
            200
        }
        crate::tasks::SendMessageOutcome::DirectMessage { response, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "direct-message");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_tasks_send_message(response, &mut body);
            200
        }
        crate::tasks::SendMessageOutcome::TaskCreated { response, task_created, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "task-created");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.TaskCreated");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_task_created(task_created, &mut body);
            body.push('}');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_tasks_send_message(response, &mut body);
            200
        }
        crate::tasks::SendMessageOutcome::VersionNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "version-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.VersionNotSupported");
            502
        }
        crate::tasks::SendMessageOutcome::ExtensionRequired { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "extension-required");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ExtensionSupportRequired");
            502
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `/message:send`: `a2a.tasks.SendMessage`, read and answered as its `ess-http/1` binding declares.
fn serve_bound_a2a_tasks_send_message<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, request: &http::Request, _path: &[String], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let read = || -> Result<json::Value, String> {
        let value = http::json_body(&request.body, true)?;
        http::with_path(value, Vec::new(), "body")
    };
    let value = match read() {
        Ok(value) => value,
        Err(detail) => return refusal_malformed(&detail),
    };
    match run_a2a_tasks_send_message(system, &value, taken) {
        Ok((status, envelope)) => http::Response::new(status, http::JSON, envelope),
        Err(refused) => refused_bound(&refused),
    }
}

/// `a2a.tasks.SendStreamingMessage` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_send_streaming_message<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_send_streaming_message(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.send_streaming_message(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_send_streaming_message(&outcome))
}

/// One declared outcome of `a2a.tasks.SendStreamingMessage`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_send_streaming_message(outcome: &crate::tasks::SendStreamingMessageOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::SendStreamingMessageOutcome::TaskNotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "task-not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            502
        }
        crate::tasks::SendStreamingMessageOutcome::TaskTerminal { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "task-terminal");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.UnsupportedOperation");
            502
        }
        crate::tasks::SendStreamingMessageOutcome::ContextMismatch { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "context-mismatch");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ContextMismatch");
            502
        }
        crate::tasks::SendStreamingMessageOutcome::ContextNotAccepted { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "context-not-accepted");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ContextNotAccepted");
            502
        }
        crate::tasks::SendStreamingMessageOutcome::ContentTypeNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "content-type-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ContentTypeNotSupported");
            502
        }
        crate::tasks::SendStreamingMessageOutcome::TaskLimit { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "task-limit");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ResourceExhausted");
            502
        }
        crate::tasks::SendStreamingMessageOutcome::HistoryLimit { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "history-limit");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ResourceExhausted");
            502
        }
        crate::tasks::SendStreamingMessageOutcome::Continued { response, message_stream_response, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "continued");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.MessageStreamResponse");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_message_stream_response(message_stream_response, &mut body);
            body.push('}');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_tasks_send_streaming_message(response, &mut body);
            200
        }
        crate::tasks::SendStreamingMessageOutcome::DirectMessage { response, message_stream_response, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "direct-message");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.MessageStreamResponse");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_message_stream_response(message_stream_response, &mut body);
            body.push('}');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_tasks_send_streaming_message(response, &mut body);
            200
        }
        crate::tasks::SendStreamingMessageOutcome::TaskCreated { response, task_created, message_stream_response, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "task-created");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.TaskCreated");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_task_created(task_created, &mut body);
            body.push('}');
            body.push(',');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.MessageStreamResponse");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_message_stream_response(message_stream_response, &mut body);
            body.push('}');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_tasks_send_streaming_message(response, &mut body);
            200
        }
        crate::tasks::SendStreamingMessageOutcome::VersionNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "version-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.VersionNotSupported");
            502
        }
        crate::tasks::SendStreamingMessageOutcome::ExtensionRequired { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "extension-required");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ExtensionSupportRequired");
            502
        }
        crate::tasks::SendStreamingMessageOutcome::StreamingNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "streaming-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.UnsupportedOperation");
            502
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `/message:stream`: `a2a.tasks.SendStreamingMessage`, read and answered as its `ess-http/1` binding declares.
fn serve_bound_a2a_tasks_send_streaming_message<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, request: &http::Request, _path: &[String], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let read = || -> Result<json::Value, String> {
        let value = http::json_body(&request.body, true)?;
        http::with_path(value, Vec::new(), "body")
    };
    let value = match read() {
        Ok(value) => value,
        Err(detail) => return refusal_malformed(&detail),
    };
    match run_a2a_tasks_send_streaming_message(system, &value, taken) {
        Ok((status, envelope)) => http::Response::new(status, http::JSON, envelope),
        Err(refused) => refused_bound(&refused),
    }
}

/// `GET` `a2a.tasks.Tasks` at `read_your_writes` consistency: every row the owed projection holds for the declared
/// parameters, read from `value`, an object keyed by their wire names.
///
/// The one path the `GET` route and [`handle`] share. A decoding failure is located under
/// `query`, as the route reports it.
fn run_a2a_tasks_tasks<A2aAgentBehaviors>(system: &crate::system::System<A2aAgentBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let (param0, param1, param2, param3) = match wire::decode_params_a2a_tasks_tasks(value, "query") {
        Ok(params) => params,
        Err(error) => return Err(entry::Refused::Input(format!("{error}"))),
    };
    match system.a2a_agent.tasks(param0, param1, param2, param3) {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_a2a_tasks_tasks(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `/tasks`: `a2a.tasks.Tasks`, read and answered as its `ess-http/1` binding declares.
fn serve_bound_a2a_tasks_tasks<A2aAgentBehaviors>(system: &crate::system::System<A2aAgentBehaviors>, request: &http::Request, _path: &[String]) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let read = || -> Result<json::Value, String> {
        let value = http::query_object(&request.query, &[("contextId", http::Scalar::Text), ("status", http::Scalar::Text), ("statusTimestampAfter", http::Scalar::Text), ("tenant", http::Scalar::Text)], "query")?;
        http::with_path(value, Vec::new(), "query")
    };
    let value = match read() {
        Ok(value) => value,
        Err(detail) => return refusal_malformed(&detail),
    };
    let result = run_a2a_tasks_tasks(system, &value);
    match result {
        Ok((status, body)) => http::Response::new(status, http::JSON, body),
        Err(refused) => refused_bound(&refused),
    }
}

/// `POST` `a2a.tasks.CancelTask`: reads the declared input, runs the port, answers the declared outcome.
fn serve_a2a_tasks_cancel_task<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, body: &[u8], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_a2a_tasks_cancel_task(system, &value, taken))
}

/// `a2a.tasks.CancelTask` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_cancel_task<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_cancel_task(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.cancel_task(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_cancel_task(&outcome))
}

/// One declared outcome of `a2a.tasks.CancelTask`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_cancel_task(outcome: &crate::tasks::CancelTaskOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::CancelTaskOutcome::VersionNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "version-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.VersionNotSupported");
            502
        }
        crate::tasks::CancelTaskOutcome::ExtensionRequired { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "extension-required");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ExtensionSupportRequired");
            502
        }
        crate::tasks::CancelTaskOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            404
        }
        crate::tasks::CancelTaskOutcome::OtherTenant { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "other-tenant");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            409
        }
        crate::tasks::CancelTaskOutcome::NotCancelableNow { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-cancelable-now");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotCancelable");
            502
        }
        crate::tasks::CancelTaskOutcome::Canceled { response, task_status_updated, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "canceled");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.TaskStatusUpdated");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_task_status_updated(task_status_updated, &mut body);
            body.push('}');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_tasks_cancel_task(response, &mut body);
            200
        }
        crate::tasks::CancelTaskOutcome::NotCancelable { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-cancelable");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotCancelable");
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `a2a.tasks.CompleteTask`: reads the declared input, runs the port, answers the declared outcome.
fn serve_a2a_tasks_complete_task<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, body: &[u8], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_a2a_tasks_complete_task(system, &value, taken))
}

/// `a2a.tasks.CompleteTask` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_complete_task<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_complete_task(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.complete_task(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_complete_task(&outcome))
}

/// One declared outcome of `a2a.tasks.CompleteTask`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_complete_task(outcome: &crate::tasks::CompleteTaskOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::CompleteTaskOutcome::Moved { task_status_updated, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "moved");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.TaskStatusUpdated");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_task_status_updated(task_status_updated, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::tasks::CompleteTaskOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            404
        }
        crate::tasks::CompleteTaskOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_a2a_tasks_task_state_conflict(error, &mut body);
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `a2a.tasks.FailTask`: reads the declared input, runs the port, answers the declared outcome.
fn serve_a2a_tasks_fail_task<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, body: &[u8], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_a2a_tasks_fail_task(system, &value, taken))
}

/// `a2a.tasks.FailTask` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_fail_task<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_fail_task(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.fail_task(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_fail_task(&outcome))
}

/// One declared outcome of `a2a.tasks.FailTask`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_fail_task(outcome: &crate::tasks::FailTaskOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::FailTaskOutcome::Moved { task_status_updated, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "moved");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.TaskStatusUpdated");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_task_status_updated(task_status_updated, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::tasks::FailTaskOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            404
        }
        crate::tasks::FailTaskOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_a2a_tasks_task_state_conflict(error, &mut body);
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `a2a.tasks.RejectTask`: reads the declared input, runs the port, answers the declared outcome.
fn serve_a2a_tasks_reject_task<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, body: &[u8], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_a2a_tasks_reject_task(system, &value, taken))
}

/// `a2a.tasks.RejectTask` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_reject_task<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_reject_task(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.reject_task(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_reject_task(&outcome))
}

/// One declared outcome of `a2a.tasks.RejectTask`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_reject_task(outcome: &crate::tasks::RejectTaskOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::RejectTaskOutcome::Moved { task_status_updated, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "moved");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.TaskStatusUpdated");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_task_status_updated(task_status_updated, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::tasks::RejectTaskOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            404
        }
        crate::tasks::RejectTaskOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_a2a_tasks_task_state_conflict(error, &mut body);
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `a2a.tasks.RequireAuth`: reads the declared input, runs the port, answers the declared outcome.
fn serve_a2a_tasks_require_auth<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, body: &[u8], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_a2a_tasks_require_auth(system, &value, taken))
}

/// `a2a.tasks.RequireAuth` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_require_auth<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_require_auth(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.require_auth(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_require_auth(&outcome))
}

/// One declared outcome of `a2a.tasks.RequireAuth`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_require_auth(outcome: &crate::tasks::RequireAuthOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::RequireAuthOutcome::Moved { task_status_updated, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "moved");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.TaskStatusUpdated");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_task_status_updated(task_status_updated, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::tasks::RequireAuthOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            404
        }
        crate::tasks::RequireAuthOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_a2a_tasks_task_state_conflict(error, &mut body);
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `a2a.tasks.RequireInput`: reads the declared input, runs the port, answers the declared outcome.
fn serve_a2a_tasks_require_input<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, body: &[u8], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_a2a_tasks_require_input(system, &value, taken))
}

/// `a2a.tasks.RequireInput` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_require_input<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_require_input(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.require_input(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_require_input(&outcome))
}

/// One declared outcome of `a2a.tasks.RequireInput`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_require_input(outcome: &crate::tasks::RequireInputOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::RequireInputOutcome::Moved { task_status_updated, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "moved");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.TaskStatusUpdated");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_task_status_updated(task_status_updated, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::tasks::RequireInputOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            404
        }
        crate::tasks::RequireInputOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_a2a_tasks_task_state_conflict(error, &mut body);
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `a2a.tasks.SubscribeToTask`: reads the declared input, runs the port, answers the declared outcome.
fn serve_a2a_tasks_subscribe_to_task<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, body: &[u8], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_a2a_tasks_subscribe_to_task(system, &value, taken))
}

/// `a2a.tasks.SubscribeToTask` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_subscribe_to_task<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_subscribe_to_task(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.subscribe_to_task(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_subscribe_to_task(&outcome))
}

/// One declared outcome of `a2a.tasks.SubscribeToTask`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_subscribe_to_task(outcome: &crate::tasks::SubscribeToTaskOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::SubscribeToTaskOutcome::VersionNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "version-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.VersionNotSupported");
            502
        }
        crate::tasks::SubscribeToTaskOutcome::ExtensionRequired { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "extension-required");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ExtensionSupportRequired");
            502
        }
        crate::tasks::SubscribeToTaskOutcome::StreamingNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "streaming-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.UnsupportedOperation");
            502
        }
        crate::tasks::SubscribeToTaskOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            404
        }
        crate::tasks::SubscribeToTaskOutcome::OtherTenant { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "other-tenant");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            409
        }
        crate::tasks::SubscribeToTaskOutcome::TaskTerminal { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "task-terminal");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.UnsupportedOperation");
            409
        }
        crate::tasks::SubscribeToTaskOutcome::Subscribed { response, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "subscribed");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_tasks_subscribe_to_task(response, &mut body);
            200
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `a2a.tasks.UpdateArtifact`: reads the declared input, runs the port, answers the declared outcome.
fn serve_a2a_tasks_update_artifact<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, body: &[u8], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_a2a_tasks_update_artifact(system, &value, taken))
}

/// `a2a.tasks.UpdateArtifact` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_update_artifact<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_update_artifact(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.update_artifact(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_update_artifact(&outcome))
}

/// One declared outcome of `a2a.tasks.UpdateArtifact`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_update_artifact(outcome: &crate::tasks::UpdateArtifactOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::UpdateArtifactOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            404
        }
        crate::tasks::UpdateArtifactOutcome::Updated { task_artifact_updated, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "updated");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.TaskArtifactUpdated");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_task_artifact_updated(task_artifact_updated, &mut body);
            body.push('}');
            body.push(']');
            202
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `a2a.tasks.StartWorking`: reads the declared input, runs the port, answers the declared outcome.
fn serve_a2a_tasks_start_working<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, body: &[u8], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let text = match std::str::from_utf8(body) {
        Ok(text) => text,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not UTF-8: {error}"));
        }
    };
    let value = match json::parse(text) {
        Ok(value) => value,
        Err(error) => {
            return http::Response::refusal(400, &format!("the body is not JSON: {error}"));
        }
    };
    http::answer(run_a2a_tasks_start_working(system, &value, taken))
}

/// `a2a.tasks.StartWorking` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_start_working<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_start_working(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.start_working(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_start_working(&outcome))
}

/// One declared outcome of `a2a.tasks.StartWorking`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_start_working(outcome: &crate::tasks::StartWorkingOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::StartWorkingOutcome::Moved { task_status_updated, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "moved");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.TaskStatusUpdated");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_task_status_updated(task_status_updated, &mut body);
            body.push('}');
            body.push(']');
            202
        }
        crate::tasks::StartWorkingOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            404
        }
        crate::tasks::StartWorkingOutcome::WrongState { error, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "wrong-state");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskStateConflict");
            json::member(&mut body, "payload");
            wire::encode_error_a2a_tasks_task_state_conflict(error, &mut body);
            409
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `a2a.tasks.PushNotificationConfigs` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_a2a_tasks_push_notification_configs<A2aAgentBehaviors>(system: &crate::system::System<A2aAgentBehaviors>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    match system.a2a_agent.push_notification_configs() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_a2a_tasks_push_notification_configs(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `a2a.tasks.AllTasks` at `read_your_writes` consistency: every row the owed projection holds.
///
/// The one path the `GET` route and [`handle`] share.
fn run_a2a_tasks_all_tasks<A2aAgentBehaviors>(system: &crate::system::System<A2aAgentBehaviors>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    match system.a2a_agent.all_tasks() {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_a2a_tasks_all_tasks(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `a2a.tasks.GetTask` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_get_task<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_get_task(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.get_task(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_get_task(&outcome))
}

/// One declared outcome of `a2a.tasks.GetTask`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_get_task(outcome: &crate::tasks::GetTaskOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::GetTaskOutcome::VersionNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "version-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.VersionNotSupported");
            502
        }
        crate::tasks::GetTaskOutcome::ExtensionRequired { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "extension-required");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ExtensionSupportRequired");
            502
        }
        crate::tasks::GetTaskOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            404
        }
        crate::tasks::GetTaskOutcome::OtherTenant { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "other-tenant");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            409
        }
        crate::tasks::GetTaskOutcome::Found { response, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_tasks_get_task(response, &mut body);
            200
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `/tasks/{id}`: `a2a.tasks.GetTask`, read and answered as its `ess-http/1` binding declares.
fn serve_bound_a2a_tasks_get_task<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, request: &http::Request, path: &[String], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let read = || -> Result<json::Value, String> {
        let value = http::flat_object(&request.query, &[("tenant", http::Flat::String, http::Shape::Optional), ("historyLength", http::Flat::Integer, http::Shape::Optional)], &["id"], true, "query")?;
        http::with_path(value, vec![("id".to_owned(), http::path_value(&path[0], http::Flat::String, "path.id")?)], "query")
    };
    let value = match read() {
        Ok(value) => value,
        Err(detail) => return refusal_malformed(&detail),
    };
    match run_a2a_tasks_get_task(system, &value, taken) {
        Ok((status, envelope)) => http::Response::new(status, http::JSON, envelope),
        Err(refused) => refused_bound(&refused),
    }
}

/// `a2a.tasks.ListTaskPushNotificationConfigs` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_list_task_push_notification_configs<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_list_task_push_notification_configs(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.list_task_push_notification_configs(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_list_task_push_notification_configs(&outcome))
}

/// One declared outcome of `a2a.tasks.ListTaskPushNotificationConfigs`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_list_task_push_notification_configs(outcome: &crate::tasks::ListTaskPushNotificationConfigsOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::ListTaskPushNotificationConfigsOutcome::VersionNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "version-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.VersionNotSupported");
            502
        }
        crate::tasks::ListTaskPushNotificationConfigsOutcome::ExtensionRequired { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "extension-required");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ExtensionSupportRequired");
            502
        }
        crate::tasks::ListTaskPushNotificationConfigsOutcome::PushNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "push-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.PushNotificationNotSupported");
            502
        }
        crate::tasks::ListTaskPushNotificationConfigsOutcome::TaskNotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "task-not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            404
        }
        crate::tasks::ListTaskPushNotificationConfigsOutcome::OtherTenant { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "other-tenant");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            409
        }
        crate::tasks::ListTaskPushNotificationConfigsOutcome::Listed { response, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "listed");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_tasks_list_task_push_notification_configs(response, &mut body);
            200
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `/tasks/{taskId}/pushNotificationConfigs`: `a2a.tasks.ListTaskPushNotificationConfigs`, read and answered as its `ess-http/1` binding declares.
fn serve_bound_a2a_tasks_list_task_push_notification_configs<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, request: &http::Request, path: &[String], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let read = || -> Result<json::Value, String> {
        let value = http::flat_object(&request.query, &[("tenant", http::Flat::String, http::Shape::Optional), ("pageSize", http::Flat::Integer, http::Shape::Optional), ("pageToken", http::Flat::String, http::Shape::Optional)], &["taskId"], true, "query")?;
        http::with_path(value, vec![("taskId".to_owned(), http::path_value(&path[0], http::Flat::String, "path.taskId")?)], "query")
    };
    let value = match read() {
        Ok(value) => value,
        Err(detail) => return refusal_malformed(&detail),
    };
    match run_a2a_tasks_list_task_push_notification_configs(system, &value, taken) {
        Ok((status, envelope)) => http::Response::new(status, http::JSON, envelope),
        Err(refused) => refused_bound(&refused),
    }
}

/// `a2a.tasks.CreateTaskPushNotificationConfig` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_create_task_push_notification_config<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_create_task_push_notification_config(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.create_task_push_notification_config(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_create_task_push_notification_config(&outcome))
}

/// One declared outcome of `a2a.tasks.CreateTaskPushNotificationConfig`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_create_task_push_notification_config(outcome: &crate::tasks::CreateTaskPushNotificationConfigOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::CreateTaskPushNotificationConfigOutcome::VersionNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "version-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.VersionNotSupported");
            502
        }
        crate::tasks::CreateTaskPushNotificationConfigOutcome::ExtensionRequired { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "extension-required");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ExtensionSupportRequired");
            502
        }
        crate::tasks::CreateTaskPushNotificationConfigOutcome::PushNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "push-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.PushNotificationNotSupported");
            502
        }
        crate::tasks::CreateTaskPushNotificationConfigOutcome::TaskNotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "task-not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            409
        }
        crate::tasks::CreateTaskPushNotificationConfigOutcome::ConfigLimit { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "config-limit");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ResourceExhausted");
            502
        }
        crate::tasks::CreateTaskPushNotificationConfigOutcome::Created { response, push_notification_config_created, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "created");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.PushNotificationConfigCreated");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_push_notification_config_created(push_notification_config_created, &mut body);
            body.push('}');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_tasks_create_task_push_notification_config(response, &mut body);
            200
        }
        crate::tasks::CreateTaskPushNotificationConfigOutcome::Assigned { response, push_notification_config_created, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "assigned");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.PushNotificationConfigCreated");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_push_notification_config_created(push_notification_config_created, &mut body);
            body.push('}');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_tasks_create_task_push_notification_config(response, &mut body);
            200
        }
    };
    body.push('}');
    (status, body)
}

/// `POST` `/tasks/{taskId}/pushNotificationConfigs`: `a2a.tasks.CreateTaskPushNotificationConfig`, read and answered as its `ess-http/1` binding declares.
fn serve_bound_a2a_tasks_create_task_push_notification_config<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, request: &http::Request, path: &[String], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let read = || -> Result<json::Value, String> {
        let value = http::json_body(&request.body, true)?;
        http::with_path(value, vec![("taskId".to_owned(), http::path_value(&path[0], http::Flat::String, "path.taskId")?)], "body")
    };
    let value = match read() {
        Ok(value) => value,
        Err(detail) => return refusal_malformed(&detail),
    };
    match run_a2a_tasks_create_task_push_notification_config(system, &value, taken) {
        Ok((status, envelope)) => http::Response::new(status, http::JSON, envelope),
        Err(refused) => refused_bound(&refused),
    }
}

/// `a2a.tasks.DeleteTaskPushNotificationConfig` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_delete_task_push_notification_config<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_delete_task_push_notification_config(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.delete_task_push_notification_config(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_delete_task_push_notification_config(&outcome))
}

/// One declared outcome of `a2a.tasks.DeleteTaskPushNotificationConfig`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_delete_task_push_notification_config(outcome: &crate::tasks::DeleteTaskPushNotificationConfigOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::DeleteTaskPushNotificationConfigOutcome::VersionNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "version-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.VersionNotSupported");
            502
        }
        crate::tasks::DeleteTaskPushNotificationConfigOutcome::ExtensionRequired { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "extension-required");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ExtensionSupportRequired");
            502
        }
        crate::tasks::DeleteTaskPushNotificationConfigOutcome::PushNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "push-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.PushNotificationNotSupported");
            502
        }
        crate::tasks::DeleteTaskPushNotificationConfigOutcome::TaskNotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "task-not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            404
        }
        crate::tasks::DeleteTaskPushNotificationConfigOutcome::Deleted { push_notification_config_deleted, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "deleted");
            json::member(&mut body, "published");
            body.push('[');
            body.push('{');
            json::member(&mut body, "event");
            json::push_text(&mut body, "a2a.tasks.PushNotificationConfigDeleted");
            json::member(&mut body, "payload");
            wire::encode_event_a2a_tasks_push_notification_config_deleted(push_notification_config_deleted, &mut body);
            body.push('}');
            body.push(']');
            202
        }
    };
    body.push('}');
    (status, body)
}

/// `DELETE` `/tasks/{taskId}/pushNotificationConfigs/{id}`: `a2a.tasks.DeleteTaskPushNotificationConfig`, read and answered as its `ess-http/1` binding declares.
fn serve_bound_a2a_tasks_delete_task_push_notification_config<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, request: &http::Request, path: &[String], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let read = || -> Result<json::Value, String> {
        let value = http::json_body(&request.body, false)?;
        http::with_path(value, vec![("taskId".to_owned(), http::path_value(&path[0], http::Flat::String, "path.taskId")?), ("id".to_owned(), http::path_value(&path[1], http::Flat::String, "path.id")?)], "body")
    };
    let value = match read() {
        Ok(value) => value,
        Err(detail) => return refusal_malformed(&detail),
    };
    match run_a2a_tasks_delete_task_push_notification_config(system, &value, taken) {
        Ok((status, envelope)) => http::Response::new(status, http::JSON, envelope),
        Err(refused) => refused_bound(&refused),
    }
}

/// `a2a.tasks.GetTaskPushNotificationConfig` from its input as a JSON value: decode, run the port, render the declared outcome.
///
/// The one path the `POST` route and [`handle`] share. A decoding failure is located under
/// `body`, as the route reports it. Every event published while the call runs is pushed onto
/// `taken`, in log order, whatever it answers; what was on the log before it is taken off
/// the log but not pushed.
fn run_a2a_tasks_get_task_push_notification_config<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, value: &json::Value, taken: &mut Vec<crate::system::SystemEvent>) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    // What is on the log before this call is not what this call published: it is delivered
    // and taken with the rest, and never pushed onto `taken`.
    let before = system.published().len();
    let input = match wire::decode_command_a2a_tasks_get_task_push_notification_config(value, "body") {
        Ok(input) => input,
        Err(error) => {
            // `400` and not `422`: this is a body the schema decides, which is the difference
            // between fixing a value and fixing a serialiser.
            return Err(entry::Refused::Input(format!("{error}")));
        }
    };
    let outcome = match system.a2a_agent.get_task_push_notification_config(input) {
        Ok(outcome) => outcome,
        Err(unmet) => return Err(entry::Refused::Unmet(format!("{unmet}"))),
    };
    // Deliver what this command published to every binding that reacts to it, then take it
    // off the log: a long-running server keeps nothing from one request to the next.
    let delivered = system.pump();
    taken.extend(system.take_published().into_iter().skip(before));
    if let Err(failure) = delivered {
        return Err(entry::Refused::Undelivered(format!("delivering what the command published: {failure}")));
    }
    Ok(answer_a2a_tasks_get_task_push_notification_config(&outcome))
}

/// One declared outcome of `a2a.tasks.GetTaskPushNotificationConfig`: the branch that was taken, every event it published in
/// publication order, the declared error where there is one, and that error's own payload —
/// with the status the contract declares for that branch.
fn answer_a2a_tasks_get_task_push_notification_config(outcome: &crate::tasks::GetTaskPushNotificationConfigOutcome) -> (u16, String) {
    let mut body = String::from("{");
    let status = match outcome {
        crate::tasks::GetTaskPushNotificationConfigOutcome::VersionNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "version-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.VersionNotSupported");
            502
        }
        crate::tasks::GetTaskPushNotificationConfigOutcome::ExtensionRequired { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "extension-required");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.ExtensionSupportRequired");
            502
        }
        crate::tasks::GetTaskPushNotificationConfigOutcome::PushNotSupported { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "push-not-supported");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.PushNotificationNotSupported");
            502
        }
        crate::tasks::GetTaskPushNotificationConfigOutcome::NotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            404
        }
        crate::tasks::GetTaskPushNotificationConfigOutcome::ConfigNotFound { .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "config-not-found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "error");
            json::push_text(&mut body, "a2a.tasks.TaskNotFound");
            409
        }
        crate::tasks::GetTaskPushNotificationConfigOutcome::Found { response, .. } => {
            json::member(&mut body, "outcome");
            json::push_text(&mut body, "found");
            json::member(&mut body, "published");
            body.push('[');
            body.push(']');
            json::member(&mut body, "response");
            wire::encode_response_a2a_tasks_get_task_push_notification_config(response, &mut body);
            200
        }
    };
    body.push('}');
    (status, body)
}

/// `GET` `/tasks/{taskId}/pushNotificationConfigs/{id}`: `a2a.tasks.GetTaskPushNotificationConfig`, read and answered as its `ess-http/1` binding declares.
fn serve_bound_a2a_tasks_get_task_push_notification_config<A2aAgentBehaviors>(system: &mut crate::system::System<A2aAgentBehaviors>, request: &http::Request, path: &[String], taken: &mut Vec<crate::system::SystemEvent>) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let read = || -> Result<json::Value, String> {
        let value = http::flat_object(&request.query, &[("tenant", http::Flat::String, http::Shape::Optional)], &["taskId", "id"], true, "query")?;
        http::with_path(value, vec![("taskId".to_owned(), http::path_value(&path[0], http::Flat::String, "path.taskId")?), ("id".to_owned(), http::path_value(&path[1], http::Flat::String, "path.id")?)], "query")
    };
    let value = match read() {
        Ok(value) => value,
        Err(detail) => return refusal_malformed(&detail),
    };
    match run_a2a_tasks_get_task_push_notification_config(system, &value, taken) {
        Ok((status, envelope)) => http::Response::new(status, http::JSON, envelope),
        Err(refused) => refused_bound(&refused),
    }
}

/// `GET` `a2a.tasks.TaskPushNotificationConfigs` at `read_your_writes` consistency: every row the owed projection holds for the declared
/// parameters, read from `value`, an object keyed by their wire names.
///
/// The one path the `GET` route and [`handle`] share. A decoding failure is located under
/// `query`, as the route reports it.
fn run_a2a_tasks_task_push_notification_configs<A2aAgentBehaviors>(system: &crate::system::System<A2aAgentBehaviors>, value: &json::Value) -> Result<(u16, String), entry::Refused>
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let (param0,) = match wire::decode_params_a2a_tasks_task_push_notification_configs(value, "query") {
        Ok(params) => params,
        Err(error) => return Err(entry::Refused::Input(format!("{error}"))),
    };
    match system.a2a_agent.task_push_notification_configs(param0) {
        Ok(rows) => {
            let mut body = String::from("{");
            json::member(&mut body, "rows");
            body.push('[');
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    body.push(',');
                }
                wire::encode_view_a2a_tasks_task_push_notification_configs(row, &mut body);
            }
            body.push(']');
            body.push('}');
            Ok((200, body))
        }
        Err(unmet) => Err(entry::Refused::Unmet(format!("{unmet}"))),
    }
}

/// `GET` `/views/tasks/pushNotificationConfigs`: `a2a.tasks.TaskPushNotificationConfigs`, read and answered as its `ess-http/1` binding declares.
fn serve_bound_a2a_tasks_task_push_notification_configs<A2aAgentBehaviors>(system: &crate::system::System<A2aAgentBehaviors>, request: &http::Request, _path: &[String]) -> http::Response
where
    A2aAgentBehaviors: crate::discovery::obligations::GetExtendedAgentCardBehavior + crate::tasks::obligations::CancelTaskBehavior + crate::tasks::obligations::CompleteTaskBehavior + crate::tasks::obligations::CreateTaskPushNotificationConfigBehavior + crate::tasks::obligations::DeleteTaskPushNotificationConfigBehavior + crate::tasks::obligations::FailTaskBehavior + crate::tasks::obligations::GetTaskBehavior + crate::tasks::obligations::GetTaskPushNotificationConfigBehavior + crate::tasks::obligations::ListTaskPushNotificationConfigsBehavior + crate::tasks::obligations::RejectTaskBehavior + crate::tasks::obligations::RequireAuthBehavior + crate::tasks::obligations::RequireInputBehavior + crate::tasks::obligations::SendMessageBehavior + crate::tasks::obligations::SendStreamingMessageBehavior + crate::tasks::obligations::StartWorkingBehavior + crate::tasks::obligations::SubscribeToTaskBehavior + crate::tasks::obligations::UpdateArtifactBehavior + crate::tasks::obligations::AllTasksQuery + crate::tasks::obligations::PushNotificationConfigsQuery + crate::tasks::obligations::TaskPushNotificationConfigsQuery + crate::tasks::obligations::TasksQuery,
{
    let read = || -> Result<json::Value, String> {
        let value = http::query_object(&request.query, &[("task_id", http::Scalar::Text)], "query")?;
        http::with_path(value, Vec::new(), "query")
    };
    let value = match read() {
        Ok(value) => value,
        Err(detail) => return refusal_malformed(&detail),
    };
    let result = run_a2a_tasks_task_push_notification_configs(system, &value);
    match result {
        Ok((status, body)) => http::Response::new(status, http::JSON, body),
        Err(refused) => refused_bound(&refused),
    }
}

/// A request a bound route cannot read as its binding declares: `refusals.malformed`, else `400`.
fn refusal_malformed(detail: &str) -> http::Response {
    http::Response::refusal(400, detail)
}

/// An unfinished realization on a bound route: `refusals.unfinished`, else `501` with `committed`.
fn refusal_unfinished(detail: &str, committed: bool) -> http::Response {
    http::Response::unfinished(detail, committed)
}

/// A caller no grant admits, on a bound route: `refusals.not_granted`, else the standard `403`.
fn refusal_not_granted(actor: Option<&str>) -> http::Response {
    not_granted(actor)
}

/// What a bound route's shared path refused, as its binding answers it.
fn refused_bound(refused: &entry::Refused) -> http::Response {
    match refused {
        entry::Refused::Input(detail) => refusal_malformed(detail),
        entry::Refused::Unmet(_) | entry::Refused::Undelivered(_) => {
            refusal_unfinished(&refused.to_string(), refused.committed())
        }
        other => http::Response::from(other),
    }
}
