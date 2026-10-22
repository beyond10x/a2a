//! The HTTP+JSON route page: which of A2A's (method, path) pairs `ess/http.yaml` serves as they
//! are, which it cannot, and which projected routes are not A2A's, each difference with the reason
//! `ess/http-unmapped.tsv` gives.

use std::{fs, path::Path};

use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;

use crate::{
    Files,
    page::{Meta, code, front_matter, inline, source_link},
};

/// The page, relative to the repository root.
pub const PAGE: &str = "website/docs/reference/http-routes.md";
const BINDING: &str = "ess/http.yaml";
const UNMAPPED: &str = "ess/http-unmapped.tsv";
const UPSTREAM: &str = "upstream/openapi/a2a.openapi.yaml";
const PROJECTED: &str = "generated/openapi-http/openapi/a2a-agent.yaml";

#[derive(Deserialize)]
struct Binding {
    routes: Vec<Route>,
}

#[derive(Deserialize)]
struct Route {
    command: Option<String>,
    view: Option<String>,
    method: String,
    path: String,
}

/// One row of `ess/http-unmapped.tsv`.
struct Difference {
    upstream: bool,
    method: String,
    path: String,
    reason: String,
}

fn differences(tsv: &str) -> Result<Vec<Difference>> {
    let mut rows = Vec::new();
    for (n, line) in tsv.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let [side, method, path, reason] = fields[..] else {
            bail!(
                "{UNMAPPED}:{}: expected side, method, path and reason separated by tabs",
                n + 1
            );
        };
        let upstream = match side {
            "upstream" => true,
            "projected" => false,
            other => bail!(
                "{UNMAPPED}:{}: side is `upstream` or `projected`, not `{other}`",
                n + 1
            ),
        };
        rows.push(Difference {
            upstream,
            method: method.to_owned(),
            path: path.to_owned(),
            reason: reason.to_owned(),
        });
    }
    Ok(rows)
}

/// A link to the entry of the operation `qualified` (`a2a.tasks.SendMessage`) on its domain page,
/// checked against the generated `specification` pages.
fn operation_link(qualified: &str, specification: &Files) -> Result<String> {
    let (domain, local) = qualified
        .rsplit_once('.')
        .with_context(|| format!("{BINDING} names `{qualified}`, which has no domain"))?;
    let file = format!("{}.md", domain.replace('.', "-"));
    let page = specification
        .get(&file)
        .with_context(|| format!("{BINDING} names `{qualified}`, whose domain has no page"))?;
    let heading = format!("\n### `{local}`\n");
    ensure!(
        String::from_utf8_lossy(page).contains(&heading),
        "{BINDING} names `{qualified}`, which {file} has no entry for"
    );
    Ok(format!(
        "[{}](./specification/{file}#{})",
        code(qualified),
        local.to_lowercase()
    ))
}

fn table(rows: &[Difference]) -> String {
    let mut out = String::from("| Method | Path | Reason |\n|---|---|---|\n");
    for row in rows {
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            code(&row.method.to_uppercase()),
            code(&row.path),
            inline(&row.reason)
        ));
    }
    out
}

/// The page from the binding, its differences, and the generated specification pages it links.
pub fn render(binding: &str, tsv: &str, specification: &Files) -> Result<String> {
    let binding: Binding =
        serde_yaml_ng::from_str(binding).with_context(|| format!("parsing {BINDING}"))?;
    let differences = differences(tsv)?;
    let (upstream, projected): (Vec<_>, Vec<_>) = differences.into_iter().partition(|d| d.upstream);
    let mut equal = String::from("| Method | Path | Operation |\n|---|---|---|\n");
    let mut equal_count = 0;
    for route in &binding.routes {
        let (kind, name) = match (&route.command, &route.view) {
            (Some(command), None) => ("command", command),
            (None, Some(view)) => ("view", view),
            _ => bail!(
                "{BINDING}: the route {} {} names neither one command nor one view",
                route.method,
                route.path
            ),
        };
        if projected
            .iter()
            .any(|d| d.method == route.method && d.path == route.path)
        {
            continue;
        }
        equal_count += 1;
        equal.push_str(&format!(
            "| {} | {} | {kind} {} |\n",
            code(&route.method.to_uppercase()),
            code(&route.path),
            operation_link(name, specification)?
        ));
    }
    let total = equal_count + upstream.len();
    let mut page = front_matter(&Meta {
        title: "HTTP+JSON routes",
        label: "HTTP+JSON routes",
        description: "Which of A2A's HTTP+JSON routes the specification serves at their path, which it cannot, and why.",
        position: Some(2),
    });
    page.push_str(&format!(
        "{binding} binds operations of the specification to A2A's HTTP+JSON paths (`ess-http/1`), and \
         `ess generate --kind openapi --http` projects it to {projected}. `task check` holds that \
         document's (method, path) pairs to {upstream_doc}, the paths A2A's `a2a.proto` declares, and \
         fails on a difference {unmapped} does not name, or on a named one that no longer differs.\n\n\
         {equal_count} of A2A's {total} routes are served at their path. The other {missing} are \
         listed below with the reason ESS cannot serve them there, and so are the {extra} projected \
         routes that are not A2A's.\n\n\
         ## Served at A2A's path\n\n{equal}\n\
         ## A2A routes not served at their path\n\n{missing_table}\n\
         ## Projected routes that are not A2A's\n\n{extra_table}",
        binding = source_link(BINDING),
        projected = source_link(PROJECTED),
        upstream_doc = source_link(UPSTREAM),
        unmapped = source_link(UNMAPPED),
        missing = upstream.len(),
        extra = projected.len(),
        missing_table = table(&upstream),
        extra_table = table(&projected),
    ));
    Ok(page)
}

/// The page, keyed by its path relative to the repository root.
pub fn page(root: &Path, specification: &Files) -> Result<Files> {
    let read =
        |path: &str| fs::read_to_string(root.join(path)).with_context(|| format!("reading {path}"));
    let page = render(&read(BINDING)?, &read(UNMAPPED)?, specification)?;
    Ok(Files::from([(PAGE.to_owned(), page.into_bytes())]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BINDING_SAMPLE: &str = "type: ess-http/1\nroutes:\n  - command: a2a.tasks.SendMessage\n    method: post\n    path: /message:send\n  - view: a2a.tasks.Extra\n    method: get\n    path: /views/extra\n";
    const TSV: &str = "upstream\tpost\t/tasks/{id}:cancel\tUNMAPPED (ESS gap): ESS-HTTP-007 {id}:verb is refused\nprojected\tget\t/views/extra\tNot an A2A operation: Optional<String> | moved\n";

    fn specification() -> Files {
        Files::from([(
            "a2a-tasks.md".to_owned(),
            b"x\n### `SendMessage`\n\ny\n### `Extra`\n".to_vec(),
        )])
    }

    #[test]
    fn equal_routes_link_their_operation_and_differences_carry_their_reason() {
        let page = render(BINDING_SAMPLE, TSV, &specification()).unwrap();
        assert!(page.starts_with("---\n# generated by a2a-docs, do not edit\n"));
        assert!(page.contains("custom_edit_url: null\n"));
        assert!(page.contains("1 of A2A's 2 routes are served at their path. The other 1"));
        assert!(page.contains(
            "| `POST` | `/message:send` | command [`a2a.tasks.SendMessage`](./specification/a2a-tasks.md#sendmessage) |"
        ));
        assert!(!page.contains("| `GET` | `/views/extra` | view"));
        assert!(page.contains("| `POST` | `/tasks/{id}:cancel` | UNMAPPED (ESS gap): ESS-HTTP-007 {id}:verb is refused |"));
        assert!(page.contains("Not an A2A operation: Optional\\<String\\> \\| moved |"));
    }

    #[test]
    fn a_bad_row_or_an_unknown_operation_is_refused() {
        assert!(render(BINDING_SAMPLE, "sideways\tget\t/x\twhy\n", &specification()).is_err());
        assert!(render(BINDING_SAMPLE, "upstream\tget\t/x\n", &specification()).is_err());
        let unknown = BINDING_SAMPLE.replace("SendMessage", "SendLetter");
        let error = render(&unknown, TSV, &specification())
            .unwrap_err()
            .to_string();
        assert!(error.contains("a2a.tasks.SendLetter"), "{error}");
    }
}
