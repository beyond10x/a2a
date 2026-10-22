// generated from a2a v1
// model digest 19f81099dbb59817313f63273ff10776f7d98070eb5de7dda53e5d67514ef5bf
// contract digest b81ca6ca77a699d33385c2f8b48b31221205b1879fc0f96d44d1a695ab4bff58
// do not edit: regenerate with `ess synthesize --layout crate`

//! Ephemeral generated server. Durable storage and production authentication remain ports.
use clap::Parser;

#[derive(Parser)]
struct Options {
#[arg(long, default_value = "127.0.0.1:8080")]
listen: String,
#[arg(long, default_value = "none", value_parser = ["none", "actor-header"])]
callers: String,
#[arg(long = "static")]
static_root: Option<std::path::PathBuf>,
}

fn main() -> std::process::ExitCode {
let options = Options::parse();
let _ = options;
eprintln!("unmet startup obligations:\ncommand behaviour: a2a.discovery.GetExtendedAgentCard\ncommand behaviour: a2a.tasks.CancelTask\ncommand behaviour: a2a.tasks.CreateTaskPushNotificationConfig\ncommand behaviour: a2a.tasks.DeleteTaskPushNotificationConfig\ncommand behaviour: a2a.tasks.GetTask\ncommand behaviour: a2a.tasks.GetTaskPushNotificationConfig\ncommand behaviour: a2a.tasks.ListTaskPushNotificationConfigs\ncommand behaviour: a2a.tasks.SendMessage\ncommand behaviour: a2a.tasks.SendStreamingMessage\ncommand behaviour: a2a.tasks.SubscribeToTask\nview query: a2a.tasks.TaskPushNotificationConfigs\nview query: a2a.tasks.Tasks");
std::process::ExitCode::FAILURE
}
