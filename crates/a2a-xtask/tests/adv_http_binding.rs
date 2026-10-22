//! Adversary cases for `a2a-xtask http-paths` (story:http-binding).
//!
//! The acceptance asks for a binding that maps every command and view to A2A's HTTP+JSON method
//! and path. Each case below takes the committed projection, applies a mutation that
//! `ess generate --kind openapi --http` really emits for a mutated `ess/http.yaml` (the binding
//! validates, `ESS-HTTP` raises nothing), and asserts that `http-paths` refuses it. The control
//! case asserts the committed projection passes, so a red case is red because of its mutation.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PROJECTED: &str = "generated/openapi-http/openapi/a2a-agent.yaml";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn committed() -> String {
    std::fs::read_to_string(root().join(PROJECTED)).unwrap()
}

fn http_paths(name: &str, projected: &str) -> Output {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("adv-http-binding");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("{name}.yaml"));
    std::fs::write(&file, projected).unwrap();
    Command::new(env!("CARGO_BIN_EXE_a2a-xtask"))
        .env("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR"))
        .args(["http-paths", "--projected"])
        .arg(&file)
        .output()
        .unwrap()
}

fn swap(text: &str, a: &str, b: &str) -> String {
    assert!(
        text.contains(a) && text.contains(b),
        "fixture lost {a:?} or {b:?}"
    );
    text.replace(a, "\u{0}PLACEHOLDER\u{0}")
        .replace(b, a)
        .replace("\u{0}PLACEHOLDER\u{0}", b)
}

#[test]
fn control_the_committed_projection_passes() {
    let out = http_paths("control", &committed());
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// `ess/http.yaml` with the commands of `post /message:send` and `post /message:stream`
/// exchanged validates and projects; the (method, path) set is unchanged, so A2A's
/// SendMessage is now served by the streaming command and the other way round.
#[test]
fn a_route_bound_to_the_wrong_command_is_drift() {
    let projected = swap(
        &committed(),
        "      operationId: a2a.tasks.SendMessage\n",
        "      operationId: a2a.tasks.SendStreamingMessage\n",
    );
    let out = http_paths("swapped-commands", &projected);
    assert!(
        !out.status.success(),
        "post /message:send now runs a2a.tasks.SendStreamingMessage (upstream operationId \
         A2AService_SendMessage), and http-paths passed:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// `ess/http.yaml` with `parameters: {taskId: input.id, id: input.task_id}` on
/// GetTaskPushNotificationConfig validates and projects exactly this operation: upstream's
/// `{taskId}` (a2a.proto `task_id`) now fills the config id and `{id}` the task id.
#[test]
fn path_variables_bound_to_the_wrong_input_fields_are_drift() {
    let original = "      - name: taskId
        in: path
        description: The input field `task_id`, carried in the path rather than the request body.
        required: true
        schema:
          $ref: '#/components/schemas/a2a.tasks.TaskId'
          x-ess-field: task_id
      - name: id
        in: path
        description: The input field `id`, carried in the path rather than the request body.
        required: true
        schema:
          $ref: '#/components/schemas/a2a.tasks.PushNotificationConfigId'
";
    let swapped = "      - name: taskId
        in: path
        description: The input field `id`, carried in the path rather than the request body.
        required: true
        schema:
          $ref: '#/components/schemas/a2a.tasks.PushNotificationConfigId'
      - name: id
        in: path
        description: The input field `task_id`, carried in the path rather than the request body.
        required: true
        schema:
          $ref: '#/components/schemas/a2a.tasks.TaskId'
          x-ess-field: task_id
";
    let text = committed();
    let at = text
        .find("      operationId: a2a.tasks.GetTaskPushNotificationConfig\n")
        .unwrap();
    let (head, tail) = text.split_at(at);
    assert!(tail.contains(original), "fixture lost the parameter block");
    let projected = format!("{head}{}", tail.replacen(original, swapped, 1));
    let out = http_paths("swapped-parameters", &projected);
    assert!(
        !out.status.success(),
        "get /tasks/{{taskId}}/pushNotificationConfigs/{{id}} now binds {{taskId}} to input.id \
         and {{id}} to input.task_id, and http-paths passed:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
}
