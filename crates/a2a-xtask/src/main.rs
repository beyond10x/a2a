use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::{Parser, Subcommand};

const SPECIFICATION: &str = "ess";
const GENERATED: &str = "generated/rust/a2a";
/// The ess-http/1 binding the synthesized `server` feature routes A2A's HTTP+JSON paths by.
const BINDING: &str = "ess/http.yaml";
/// `.ess-output/` carries a per-run anchor; `Cargo.lock` is held by `--locked`, not by synthesis.
const IGNORED: [&str; 2] = [".ess-output", "Cargo.lock"];

#[derive(Parser)]
#[command(about = "Repository checks for a2a")]
struct Cli {
    #[command(subcommand)]
    command: Task,
}

#[derive(Subcommand)]
enum Task {
    /// Rewrite generated/rust/a2a from a fresh synthesis of ess/.
    Generate,
    /// Fail when generated/rust/a2a differs from a fresh synthesis of ess/.
    Drift,
    /// Fail when the (method, path) pairs of the ess-http/1 projection differ from A2A's
    /// HTTP+JSON binding in ways the allow-list does not name, or the allow-list names a pair
    /// that no longer differs.
    HttpPaths(HttpPaths),
}

#[derive(clap::Args)]
struct HttpPaths {
    /// OpenAPI document projected from ess/ through the ess-http/1 binding.
    #[arg(long, default_value = PROJECTED)]
    projected: PathBuf,
    /// OpenAPI document of A2A's HTTP+JSON binding, derived from upstream/a2a.proto.
    #[arg(long, default_value = UPSTREAM)]
    upstream: PathBuf,
    /// Tab-separated `side method path reason` lines naming every accepted difference.
    #[arg(long, default_value = ALLOWED)]
    allowed: PathBuf,
}

const PROJECTED: &str = "generated/openapi-http/openapi/a2a-agent.yaml";
const UPSTREAM: &str = "upstream/openapi/a2a.openapi.yaml";
const ALLOWED: &str = "ess/http-unmapped.tsv";
const METHODS: [&str; 8] = [
    "get", "put", "post", "delete", "patch", "head", "options", "trace",
];

fn main() -> ExitCode {
    let cli = Cli::parse();
    let root = repository_root();
    let result = match cli.command {
        Task::Generate => generate(&root),
        Task::Drift => drift(&root),
        Task::HttpPaths(args) => http_paths(&root, &args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    manifest_dir
        .ancestors()
        .nth(2)
        .expect("crates/a2a-xtask sits two levels below the repository root")
        .to_path_buf()
}

fn generate(root: &Path) -> Result<(), String> {
    let lock = root.join(GENERATED).join("Cargo.lock");
    let held = fs::read(&lock).ok();
    synthesize(root, &root.join(GENERATED))?;
    match held {
        Some(bytes) => {
            fs::write(&lock, bytes).map_err(|e| format!("write {}: {e}", lock.display()))
        }
        None => Ok(()),
    }
}

fn synthesize(root: &Path, out: &Path) -> Result<(), String> {
    if out.exists() {
        fs::remove_dir_all(out).map_err(|e| format!("remove {}: {e}", out.display()))?;
    }
    let status = Command::new("ess")
        .current_dir(root)
        .args(["generate", "synthesize", "--path", SPECIFICATION])
        .args(["--http", BINDING])
        .args(["--target", "rust", "--layout", "crate", "--out"])
        .arg(out)
        .status()
        .map_err(|e| format!("run ess: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("ess generate synthesize exited with {status}"))
    }
}

fn drift(root: &Path) -> Result<(), String> {
    let scratch = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"))
        .join("a2a-xtask-drift");
    synthesize(root, &scratch)?;
    let committed = root.join(GENERATED);
    let fresh = files(&scratch)?;
    let held = files(&committed)?;
    let mut differences = Vec::new();
    for path in fresh.union(&held) {
        match (fresh.contains(path), held.contains(path)) {
            (true, false) => differences.push(format!("missing: {}", path.display())),
            (false, true) => differences.push(format!("not generated: {}", path.display())),
            _ => {
                let a = fs::read(scratch.join(path)).map_err(|e| e.to_string())?;
                let b = fs::read(committed.join(path)).map_err(|e| e.to_string())?;
                if a != b {
                    differences.push(format!("edited: {}", path.display()));
                }
            }
        }
    }
    if differences.is_empty() {
        println!("{GENERATED}: no drift from {SPECIFICATION}/");
        Ok(())
    } else {
        Err(format!(
            "{GENERATED} drifted from a fresh synthesis of {SPECIFICATION}/ (run `task generate`):\n  {}",
            differences.join("\n  ")
        ))
    }
}

type Route = (String, String);

/// The side of a difference, and the document `operations` reads: only upstream's path
/// parameters stand for their own field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Side {
    /// In upstream, absent from the projection.
    Upstream,
    /// In the projection, absent from upstream.
    Projected,
}

impl Side {
    fn name(self) -> &'static str {
        match self {
            Side::Upstream => "upstream",
            Side::Projected => "projected",
        }
    }
}

fn http_paths(root: &Path, args: &HttpPaths) -> Result<(), String> {
    let read = |path: &Path| {
        let path = root.join(path);
        fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))
    };
    let projected_operations = operations(&read(&args.projected)?, Side::Projected)
        .map_err(|e| format!("{}: {e}", args.projected.display()))?;
    let upstream_operations = operations(&read(&args.upstream)?, Side::Upstream)
        .map_err(|e| format!("{}: {e}", args.upstream.display()))?;
    let projected: BTreeSet<Route> = projected_operations.keys().cloned().collect();
    let upstream: BTreeSet<Route> = upstream_operations.keys().cloned().collect();
    let allowed = allow_list(&read(&args.allowed)?)
        .map_err(|e| format!("{}: {e}", args.allowed.display()))?;
    let mut problems = compare(&projected, &upstream, &allowed);
    problems.extend(bindings(&projected_operations, &upstream_operations));
    if problems.is_empty() {
        let equal = projected.intersection(&upstream).count();
        println!(
            "{}: {equal} (method, path) pairs equal to {}, each bound to upstream's operation and path fields, {} differences named in {}",
            args.projected.display(),
            args.upstream.display(),
            allowed.len(),
            args.allowed.display()
        );
        Ok(())
    } else {
        Err(format!(
            "{} drifted from {} (fix ess/http.yaml and run `task projections`, or name the difference in {}):\n  {}",
            args.projected.display(),
            args.upstream.display(),
            args.allowed.display(),
            problems.join("\n  ")
        ))
    }
}

fn compare(
    projected: &BTreeSet<Route>,
    upstream: &BTreeSet<Route>,
    allowed: &BTreeSet<(Side, Route)>,
) -> Vec<String> {
    let mut differences = BTreeSet::new();
    for route in upstream.difference(projected) {
        differences.insert((Side::Upstream, route.clone()));
    }
    for route in projected.difference(upstream) {
        differences.insert((Side::Projected, route.clone()));
    }
    let mut problems = Vec::new();
    for (side, (method, path)) in differences.difference(allowed) {
        let where_ = match side {
            Side::Upstream => "in upstream, not projected",
            Side::Projected => "projected, not in upstream",
        };
        problems.push(format!(
            "{method} {path}: {where_}, and not in the allow-list"
        ));
    }
    for (side, (method, path)) in allowed.difference(&differences) {
        problems.push(format!(
            "{method} {path}: the allow-list names it on side `{}`, but it is no such difference",
            side.name()
        ));
    }
    problems
}

/// What one operation binds: its `operationId` and, per path parameter, the field it fills.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Operation {
    operation_id: Option<String>,
    /// Path parameter name to the field it fills. Projected: `x-ess-field` where ESS writes one,
    /// else the field its description names (a command's "The input field `<field>`, carried in
    /// the path", a view's "Read by the view's filter as `param.<field>`"), else `None`, which the
    /// comparison refuses. Upstream: the parameter's own name, its proto field camel-cased.
    path_parameters: BTreeMap<String, Option<String>>,
}

/// ESS writes `x-ess-field` only where a field's wire name differs from its field name, so a
/// parameter bound to `input.id` carries none whatever the parameter is called, and a view's
/// path parameter carries none at all; the description ESS writes names the field either way.
#[derive(Default)]
struct Parameter {
    name: String,
    in_path: bool,
    marked: Option<String>,
    described: Option<String>,
}

const COMMAND_FIELD: &str = "description: The input field `";
const VIEW_FIELD: &str = "description: Read by the view's filter as `param.";

/// Upstream `operationId`s whose RPC is bound to an ESS operation of another local name.
/// Every other RPC `A2AService_<Rpc>` binds the ESS operation `a2a.<domain>.<Rpc>`.
const RENAMED: [(&str, &str); 1] = [("ListTasks", "a2a.tasks.Tasks")];
const UPSTREAM_PREFIX: &str = "A2AService_";

/// For each (method, path) pair both documents carry: upstream's RPC must be the operation the
/// projection binds there, and each upstream path parameter, snake-cased, the field it fills.
fn bindings(
    projected: &BTreeMap<Route, Operation>,
    upstream: &BTreeMap<Route, Operation>,
) -> Vec<String> {
    let mut problems = Vec::new();
    for (route @ (method, path), theirs) in upstream {
        let Some(ours) = projected.get(route) else {
            continue;
        };
        let rpc = theirs
            .operation_id
            .as_deref()
            .and_then(|id| id.strip_prefix(UPSTREAM_PREFIX));
        let bound = ours.operation_id.as_deref();
        match (rpc, bound) {
            (Some(rpc), Some(bound)) => {
                let expected = RENAMED
                    .iter()
                    .find(|(from, _)| *from == rpc)
                    .map(|(_, to)| bound == *to)
                    .unwrap_or_else(|| bound.rsplit('.').next() == Some(rpc));
                if !expected {
                    problems.push(format!(
                        "{method} {path}: upstream runs {UPSTREAM_PREFIX}{rpc}, the projection binds {bound}"
                    ));
                }
            }
            _ => problems.push(format!(
                "{method} {path}: operationId upstream {:?}, projected {:?}; both are required and upstream's starts with {UPSTREAM_PREFIX}",
                theirs.operation_id, ours.operation_id
            )),
        }
        for name in theirs.path_parameters.keys() {
            let want = snake_case(name);
            match ours.path_parameters.get(name) {
                Some(Some(field)) if *field == want => {}
                Some(Some(field)) => problems.push(format!(
                    "{method} {path}: path parameter `{{{name}}}` fills upstream field `{want}`, the projection fills `{field}`"
                )),
                Some(None) => problems.push(format!(
                    "{method} {path}: projected path parameter `{{{name}}}` names no field: no `x-ess-field`, and no description \"The input field `<field>`\" or \"Read by the view's filter as `param.<field>`\""
                )),
                None => problems.push(format!(
                    "{method} {path}: path parameter `{{{name}}}` is not a projected path parameter"
                )),
            }
        }
        for name in ours.path_parameters.keys() {
            if !theirs.path_parameters.contains_key(name) {
                problems.push(format!(
                    "{method} {path}: projected path parameter `{{{name}}}` is no upstream path parameter"
                ));
            }
        }
    }
    problems
}

fn snake_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            if !out.is_empty() {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// The (method, path) pairs under the top-level `paths:` of an OpenAPI document in block YAML:
/// a path key at indent 2, plain or quoted, and its operations at indent 4.
#[cfg(test)]
fn routes(document: &str) -> Result<BTreeSet<Route>, String> {
    Ok(operations(document, Side::Upstream)?.into_keys().collect())
}

/// Every operation under the top-level `paths:`, with its `operationId` (indent 6) and the
/// entries of its `parameters:` list (indent 6) that are `in: path`.
fn operations(document: &str, side: Side) -> Result<BTreeMap<Route, Operation>, String> {
    let mut found: BTreeMap<Route, Operation> = BTreeMap::new();
    let mut in_paths = false;
    let mut seen_paths = false;
    let mut path: Option<String> = None;
    let mut current: Option<Route> = None;
    let mut in_parameters = false;
    let mut parameter: Option<Parameter> = None;
    let finish = |found: &mut BTreeMap<Route, Operation>,
                  current: &Option<Route>,
                  parameter: &mut Option<Parameter>| {
        if let (Some(route), Some(p)) = (current, parameter.take())
            && p.in_path
        {
            let field = match side {
                Side::Projected => p.marked.or(p.described),
                Side::Upstream => Some(p.name.clone()),
            };
            found
                .entry(route.clone())
                .or_default()
                .path_parameters
                .insert(p.name, field);
        }
    };
    for (number, line) in document.lines().enumerate() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if in_parameters && indent <= 6 && !(indent == 6 && trimmed.starts_with("- ")) {
            finish(&mut found, &current, &mut parameter);
            in_parameters = false;
        }
        if indent == 0 {
            in_paths = trimmed == "paths:";
            seen_paths |= in_paths;
            path = None;
            current = None;
            continue;
        }
        if !in_paths {
            continue;
        }
        let key = trimmed.strip_suffix(':');
        match (indent, key) {
            (2, Some(key)) => {
                path = Some(unquote(key).to_string());
                current = None;
                continue;
            }
            (2, None) => return Err(format!("line {}: a path key ends in `:`", number + 1)),
            (4, Some(key)) if METHODS.contains(&key) => {
                let Some(path) = &path else {
                    return Err(format!("line {}: an operation outside a path", number + 1));
                };
                let route = (key.to_string(), path.clone());
                found.insert(route.clone(), Operation::default());
                current = Some(route);
                continue;
            }
            (4, _) => {
                current = None;
                continue;
            }
            _ => {}
        }
        let Some(route) = &current else {
            continue;
        };
        if indent == 6 {
            if let Some(id) = trimmed.strip_prefix("operationId:") {
                found.entry(route.clone()).or_default().operation_id =
                    Some(unquote(id.trim()).to_string());
            } else if trimmed == "parameters:" {
                in_parameters = true;
            }
        }
        if !in_parameters {
            continue;
        }
        let item = trimmed.strip_prefix("- ");
        if let Some(name) = item.and_then(|rest| rest.strip_prefix("name:")) {
            finish(&mut found, &current, &mut parameter);
            parameter = Some(Parameter {
                name: unquote(name.trim()).to_string(),
                ..Parameter::default()
            });
        } else if let Some(p) = parameter.as_mut() {
            if trimmed == "in: path" {
                p.in_path = true;
            } else if let Some(value) = trimmed.strip_prefix("x-ess-field:") {
                p.marked = Some(unquote(value.trim()).to_string());
            } else if let Some(field) = trimmed
                .strip_prefix(COMMAND_FIELD)
                .or_else(|| trimmed.strip_prefix(VIEW_FIELD))
                .and_then(|rest| rest.split_once('`'))
                .map(|(field, _)| field.to_string())
            {
                p.described = Some(field);
            }
        }
    }
    finish(&mut found, &current, &mut parameter);
    if !seen_paths {
        return Err("no top-level `paths:`".to_string());
    }
    Ok(found)
}

fn unquote(key: &str) -> &str {
    for quote in ['"', '\''] {
        if let Some(inner) = key
            .strip_prefix(quote)
            .and_then(|rest| rest.strip_suffix(quote))
        {
            return inner;
        }
    }
    key
}

fn allow_list(text: &str) -> Result<BTreeSet<(Side, Route)>, String> {
    let mut allowed = BTreeSet::new();
    for (number, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let [side, method, path, reason] = fields[..] else {
            return Err(format!(
                "line {}: expected four tab-separated fields `side method path reason`",
                number + 1
            ));
        };
        let side = match side {
            "upstream" => Side::Upstream,
            "projected" => Side::Projected,
            other => {
                return Err(format!(
                    "line {}: side `{other}` is neither `upstream` nor `projected`",
                    number + 1
                ));
            }
        };
        if !METHODS.contains(&method) {
            return Err(format!("line {}: `{method}` is no HTTP method", number + 1));
        }
        if !path.starts_with('/') {
            return Err(format!(
                "line {}: path `{path}` does not start with `/`",
                number + 1
            ));
        }
        if reason.trim().is_empty() {
            return Err(format!("line {}: the reason is empty", number + 1));
        }
        if !allowed.insert((side, (method.to_string(), path.to_string()))) {
            return Err(format!(
                "line {}: {method} {path} is named twice",
                number + 1
            ));
        }
    }
    Ok(allowed)
}

fn files(dir: &Path) -> Result<BTreeSet<PathBuf>, String> {
    let mut found = BTreeSet::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(current) = pending.pop() {
        let entries =
            fs::read_dir(&current).map_err(|e| format!("read {}: {e}", current.display()))?;
        for entry in entries {
            let path = entry.map_err(|e| e.to_string())?.path();
            let relative = path.strip_prefix(dir).map_err(|e| e.to_string())?;
            if IGNORED.iter().any(|ignored| relative == Path::new(ignored)) {
                continue;
            }
            if path.is_dir() {
                pending.push(path);
            } else {
                found.insert(relative.to_path_buf());
            }
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    const QUOTED: &str = "openapi: 3.0.0\npaths:\n  /tasks:\n    get:\n      operationId: List\n  \"/tasks/{id}\":\n    get:\n      parameters:\n        - name: id\n    post:\n      summary: x\ncomponents:\n  schemas:\n    get:\n      type: object\n";
    const PLAIN: &str = "openapi: 3.1.0\npaths:\n  /tasks:\n    get:\n      operationId: a\n  /tasks/{id}:\n    get:\n      operationId: b\n  /extra:\n    delete:\n      operationId: c\n";

    fn route(method: &str, path: &str) -> Route {
        (method.to_string(), path.to_string())
    }

    #[test]
    fn reads_plain_and_quoted_path_keys_only_under_paths() {
        let quoted = routes(QUOTED).unwrap();
        assert_eq!(
            quoted,
            BTreeSet::from([
                route("get", "/tasks"),
                route("get", "/tasks/{id}"),
                route("post", "/tasks/{id}")
            ])
        );
        let plain = routes(PLAIN).unwrap();
        assert!(plain.contains(&route("get", "/tasks/{id}")));
        assert!(plain.contains(&route("delete", "/extra")));
        assert_eq!(plain.len(), 3);
    }

    #[test]
    fn refuses_a_document_without_paths() {
        assert!(routes("openapi: 3.0.0\ninfo:\n  title: x\n").is_err());
    }

    #[test]
    fn an_unnamed_difference_on_either_side_is_drift() {
        let problems = compare(
            &routes(PLAIN).unwrap(),
            &routes(QUOTED).unwrap(),
            &BTreeSet::new(),
        );
        assert_eq!(
            problems,
            vec![
                "post /tasks/{id}: in upstream, not projected, and not in the allow-list",
                "delete /extra: projected, not in upstream, and not in the allow-list",
            ]
        );
    }

    #[test]
    fn named_differences_pass_and_stale_names_fail() {
        let allowed = allow_list(
            "upstream\tpost\t/tasks/{id}\tUNMAPPED: reason\nprojected\tdelete\t/extra\tnot A2A\n",
        )
        .unwrap();
        let projected = routes(PLAIN).unwrap();
        let upstream = routes(QUOTED).unwrap();
        assert!(compare(&projected, &upstream, &allowed).is_empty());
        let stale = allow_list(
            "upstream\tpost\t/tasks/{id}\tUNMAPPED: reason\nprojected\tdelete\t/extra\tnot A2A\nprojected\tget\t/tasks\tstale\n",
        )
        .unwrap();
        assert_eq!(
            compare(&projected, &upstream, &stale),
            vec![
                "get /tasks: the allow-list names it on side `projected`, but it is no such difference"
            ]
        );
    }

    const UPSTREAM_OPS: &str = "paths:\n  /message:send:\n    post:\n      operationId: A2AService_SendMessage\n  /tasks:\n    get:\n      operationId: A2AService_ListTasks\n  \"/tasks/{taskId}/configs/{id}\":\n    get:\n      operationId: A2AService_GetConfig\n      responses:\n        \"200\":\n          description: ok\n      parameters:\n        - name: taskId\n          in: path\n          required: true\n        - name: id\n          in: path\n        - name: tenant\n          in: query\n    delete:\n      operationId: A2AService_DeleteConfig\n      parameters:\n        - name: taskId\n          in: path\n        - name: id\n          in: path\n";

    fn projected_ops(
        send: &str,
        get: &str,
        delete: &str,
        task_field: &str,
        id_field: &str,
    ) -> String {
        // As ESS writes it: a description naming the field on every input parameter, and
        // `x-ess-field` only where the field's wire name differs from its name (`task_id`).
        let field = |f: &str| {
            let marked = if f.contains('_') {
                format!("          x-ess-field: {f}\n")
            } else {
                String::new()
            };
            format!(
                "        description: The input field `{f}`, carried in the path rather than the request body.\n        schema:\n          $ref: '#/x'\n{marked}"
            )
        };
        format!(
            "paths:\n  /message:send:\n    post:\n      operationId: {send}\n  /tasks:\n    get:\n      operationId: a2a.tasks.Tasks\n  /tasks/{{taskId}}/configs/{{id}}:\n    get:\n      operationId: {get}\n      parameters:\n      - name: taskId\n        in: path\n{}      - name: id\n        in: path\n{}      - name: tenant\n        in: query\n        schema:\n          x-ess-field: tenant_other\n      responses:\n        '200':\n          description: ok\n    delete:\n      operationId: {delete}\n      parameters:\n      - name: taskId\n        in: path\n        schema:\n          x-ess-field: task_id\n      - name: id\n        in: path\n        description: The input field `id`, carried in the path rather than the request body.\n",
            field(task_field),
            field(id_field)
        )
    }

    fn binding_problems(projected: &str) -> Vec<String> {
        bindings(
            &operations(projected, Side::Projected).unwrap(),
            &operations(UPSTREAM_OPS, Side::Upstream).unwrap(),
        )
    }

    #[test]
    fn reads_operation_ids_and_path_fields_in_both_styles() {
        let up = operations(UPSTREAM_OPS, Side::Upstream).unwrap();
        let get = &up[&route("get", "/tasks/{taskId}/configs/{id}")];
        assert_eq!(get.operation_id.as_deref(), Some("A2AService_GetConfig"));
        assert_eq!(
            get.path_parameters,
            BTreeMap::from([
                ("id".to_string(), Some("id".to_string())),
                ("taskId".to_string(), Some("taskId".to_string()))
            ])
        );
        let ours = operations(
            &projected_ops(
                "a2a.tasks.SendMessage",
                "a2a.tasks.GetConfig",
                "a2a.tasks.DeleteConfig",
                "task_id",
                "id",
            ),
            Side::Projected,
        )
        .unwrap();
        assert_eq!(
            ours[&route("get", "/tasks/{taskId}/configs/{id}")].path_parameters,
            BTreeMap::from([
                ("id".to_string(), Some("id".to_string())),
                ("taskId".to_string(), Some("task_id".to_string()))
            ])
        );
    }

    #[test]
    fn the_right_operations_and_fields_pass() {
        let ok = projected_ops(
            "a2a.tasks.SendMessage",
            "a2a.tasks.GetConfig",
            "a2a.tasks.DeleteConfig",
            "task_id",
            "id",
        );
        assert_eq!(binding_problems(&ok), Vec::<String>::new());
    }

    #[test]
    fn a_route_bound_to_another_command_is_refused() {
        let swapped = projected_ops(
            "a2a.tasks.SendStreamingMessage",
            "a2a.tasks.DeleteConfig",
            "a2a.tasks.GetConfig",
            "task_id",
            "id",
        );
        assert_eq!(
            binding_problems(&swapped),
            vec![
                "delete /tasks/{taskId}/configs/{id}: upstream runs A2AService_DeleteConfig, the projection binds a2a.tasks.GetConfig",
                "get /tasks/{taskId}/configs/{id}: upstream runs A2AService_GetConfig, the projection binds a2a.tasks.DeleteConfig",
                "post /message:send: upstream runs A2AService_SendMessage, the projection binds a2a.tasks.SendStreamingMessage",
            ]
        );
    }

    #[test]
    fn the_renamed_view_must_be_the_named_one() {
        let other = projected_ops(
            "a2a.tasks.SendMessage",
            "a2a.tasks.GetConfig",
            "a2a.tasks.DeleteConfig",
            "task_id",
            "id",
        )
        .replace("a2a.tasks.Tasks", "a2a.tasks.AllTasks");
        assert_eq!(
            binding_problems(&other),
            vec![
                "get /tasks: upstream runs A2AService_ListTasks, the projection binds a2a.tasks.AllTasks"
            ]
        );
    }

    #[test]
    fn swapped_path_fields_are_refused() {
        let swapped = projected_ops(
            "a2a.tasks.SendMessage",
            "a2a.tasks.GetConfig",
            "a2a.tasks.DeleteConfig",
            "id",
            "task_id",
        );
        assert_eq!(
            binding_problems(&swapped),
            vec![
                "get /tasks/{taskId}/configs/{id}: path parameter `{id}` fills upstream field `id`, the projection fills `task_id`",
                "get /tasks/{taskId}/configs/{id}: path parameter `{taskId}` fills upstream field `task_id`, the projection fills `id`",
            ]
        );
    }

    fn ok_ops() -> String {
        projected_ops(
            "a2a.tasks.SendMessage",
            "a2a.tasks.GetConfig",
            "a2a.tasks.DeleteConfig",
            "task_id",
            "id",
        )
    }

    const COMMAND_TASK: &str = "        description: The input field `task_id`, carried in the path rather than the request body.\n        schema:\n          $ref: '#/x'\n          x-ess-field: task_id\n";

    #[test]
    fn a_view_path_parameter_is_read_from_its_filter_description() {
        let view = |field: &str| {
            ok_ops().replacen(
                COMMAND_TASK,
                &format!(
                    "        description: Read by the view's filter as `param.{field}`; carried in the path rather than the query.\n        schema:\n          $ref: '#/x'\n"
                ),
                1,
            )
        };
        assert_eq!(binding_problems(&view("task_id")), Vec::<String>::new());
        assert_eq!(
            binding_problems(&view("id")),
            vec![
                "get /tasks/{taskId}/configs/{id}: path parameter `{taskId}` fills upstream field `task_id`, the projection fills `id`"
            ]
        );
    }

    #[test]
    fn a_projected_path_parameter_naming_no_field_is_refused() {
        let unnamed = ok_ops().replacen(
            COMMAND_TASK,
            "        description: Some other wording.\n        schema:\n          $ref: '#/x'\n",
            1,
        );
        assert_eq!(
            binding_problems(&unnamed),
            vec![
                "get /tasks/{taskId}/configs/{id}: projected path parameter `{taskId}` names no field: no `x-ess-field`, and no description \"The input field `<field>`\" or \"Read by the view's filter as `param.<field>`\""
            ]
        );
    }

    #[test]
    fn snake_cases_camel_names() {
        assert_eq!(snake_case("taskId"), "task_id");
        assert_eq!(snake_case("id"), "id");
        assert_eq!(snake_case("statusTimestampAfter"), "status_timestamp_after");
    }

    #[test]
    fn refuses_malformed_allow_list_lines() {
        for line in [
            "upstream\tpost\t/x",
            "elsewhere\tpost\t/x\twhy",
            "upstream\tfetch\t/x\twhy",
            "upstream\tpost\tx\twhy",
            "upstream\tpost\t/x\t ",
            "upstream\tpost\t/x\twhy\nupstream\tpost\t/x\tagain",
        ] {
            assert!(allow_list(line).is_err(), "{line:?} was accepted");
        }
    }
}
