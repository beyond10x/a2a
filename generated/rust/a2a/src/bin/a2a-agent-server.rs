// generated from a2a v1
// model digest d675090b4779d71e3671099ac4d5efbaa17d7a58ae4e0ddaa6cafd8d5a23f002
// contract digest 3dea9905e0cd94c1255be15d5619996d104b54b00e2b8097da86f0bf80643b7f
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
