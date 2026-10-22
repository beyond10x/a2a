//! Adversary cases, pass 2, for `a2a-xtask http-paths` (story:http-binding).
//!
//! Pass 2 reads the field a projected path parameter fills from `x-ess-field`, else from the
//! description "The input field `<field>`, carried in the path…", else from the parameter's name.
//! ESS 0.67.0 writes neither of the first two for a *view* path parameter: binding the view
//! `a2a.tasks.TaskPushNotificationConfigs` with `parameters: {taskId: param.task_id}` projects
//!
//! ```yaml
//! - name: taskId
//!   in: path
//!   description: Read by the view's filter as `param.task_id`; carried in the path rather than the query.
//! ```
//!
//! (scratch probe `p2-view-param-probe`). So on a view route the check compares the parameter's
//! name with itself. The cases below put that projected parameter, as ESS writes it, on an equal
//! route and assert what the check claims: the field is compared, not the name.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PROJECTED: &str = "generated/openapi-http/openapi/a2a-agent.yaml";
const OPERATION: &str = "      operationId: a2a.tasks.GetTaskPushNotificationConfig\n";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn http_paths(name: &str, projected: &str) -> Output {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("adv-http-binding-pass2");
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

/// The committed projection with one parameter block of `get
/// /tasks/{taskId}/pushNotificationConfigs/{id}` replaced.
fn with_parameter(original: &str, replacement: &str) -> String {
    let text = std::fs::read_to_string(root().join(PROJECTED)).unwrap();
    let at = text.find(OPERATION).unwrap();
    let (head, tail) = text.split_at(at);
    assert!(tail.contains(original), "fixture lost the parameter block");
    format!("{head}{}", tail.replacen(original, replacement, 1))
}

/// `{id}` filling the task id, written as ESS writes a view path parameter: no `x-ess-field`,
/// the field named only as `param.task_id`. Upstream's `{id}` is the config id, so this is the
/// pass-1 parameter swap, and the name fallback passes it.
#[test]
fn a_view_style_path_parameter_filling_the_wrong_field_is_drift() {
    let projected = with_parameter(
        "      - name: id
        in: path
        description: The input field `id`, carried in the path rather than the request body.
",
        "      - name: id
        in: path
        description: Read by the view's filter as `param.task_id`; carried in the path rather than the query.
",
    );
    let out = http_paths("view-style-wrong-field", &projected);
    assert!(
        !out.status.success(),
        "{{id}} fills `task_id` (as ESS writes a view path parameter), and http-paths passed:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// `{taskId}` filling `task_id`, written as ESS writes a view path parameter: the right field,
/// refused because the check compares the name `taskId` with `task_id`.
#[test]
fn a_view_style_path_parameter_filling_the_right_field_passes() {
    let projected = with_parameter(
        "      - name: taskId
        in: path
        description: The input field `task_id`, carried in the path rather than the request body.
        required: true
        schema:
          $ref: '#/components/schemas/a2a.tasks.TaskId'
          x-ess-field: task_id
",
        "      - name: taskId
        in: path
        description: Read by the view's filter as `param.task_id`; carried in the path rather than the query.
        required: true
        schema:
          $ref: '#/components/schemas/a2a.tasks.TaskId'
",
    );
    let out = http_paths("view-style-right-field", &projected);
    assert!(
        out.status.success(),
        "{{taskId}} fills `task_id` (as ESS writes a view path parameter), and http-paths refused it:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
