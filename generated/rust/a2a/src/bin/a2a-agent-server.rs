// generated from a2a v1
// model digest 340c397069089debcfcb2cf241e311c06d40ac443ecea2a196508e49c5fb6859
// contract digest f01a7ba8cee02bc352a5a026f681d10fa8fa5237c90807e3a38f323f46cb67ea
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
eprintln!("unmet startup obligations:\ncommand behaviour: a2a.discovery.GetExtendedAgentCard\ncommand behaviour: a2a.tasks.CancelTask\ncommand behaviour: a2a.tasks.CreateTaskPushNotificationConfig\ncommand behaviour: a2a.tasks.DeleteTaskPushNotificationConfig\ncommand behaviour: a2a.tasks.GetTask\ncommand behaviour: a2a.tasks.GetTaskPushNotificationConfig\ncommand behaviour: a2a.tasks.SendMessage\ncommand behaviour: a2a.tasks.SendStreamingMessage\ncommand behaviour: a2a.tasks.SubscribeToTask\nview query: a2a.tasks.TaskPushNotificationConfigs");
std::process::ExitCode::FAILURE
}
