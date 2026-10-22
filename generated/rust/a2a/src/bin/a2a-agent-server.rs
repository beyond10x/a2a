// generated from a2a v1
// model digest d4cf404c98ec99676cdd8fe8e411c38768d3acb9560e07676018a3ce6a70e28d
// contract digest f51f3ef221113fd930cbd423a63039558132d9f28674116bc714692bd0a5ad44
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
