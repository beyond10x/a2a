//! The specification reference: the pages of `ess generate --kind docs` for the system in `ess/`,
//! rewritten as site pages in one directory.

use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail, ensure};

use crate::{
    Files,
    page::{Meta, front_matter},
};

/// Where the specification pages land, relative to the repository root. `a2a-docs` owns it whole.
pub const DIR: &str = "website/docs/reference/specification";
/// The ESS system, relative to the repository root.
const SYSTEM: &str = "ess";
/// The pages `ess generate --kind docs` writes beside the index, in sidebar order after the domains.
const SYSTEM_PAGES: [&str; 3] = ["interactions.md", "crossings.md", "topology.md"];
/// The category file of [`DIR`]: its sidebar label and its place under Reference.
const CATEGORY: &str = "{\n  \"label\": \"Specification\",\n  \"position\": 4\n}\n";

/// Strip the leading HTML comment `ess generate` writes, returning the rest.
fn strip_ess_banner(page: &str) -> Result<&str> {
    let rest = page
        .strip_prefix("<!--")
        .context("ess page does not open with its provenance comment")?;
    let end = rest
        .find("-->")
        .context("ess provenance comment is not closed")?;
    Ok(rest[end + 3..].trim_start_matches('\n'))
}

/// The level-one heading and the body after it. The site prints the front matter title as the
/// page heading, and a second `<h1>` would take the anchor of a same-named entry, as `# Tasks`
/// would from `` ### `Tasks` ``.
fn split_title(body: &str) -> Result<(&str, &str)> {
    let (first, rest) = body.split_once('\n').unwrap_or((body, ""));
    let title = first
        .strip_prefix("# ")
        .context("ess page does not open with a level-one heading")?
        .trim();
    Ok((title, rest.trim_start_matches('\n')))
}

/// The first paragraph of `body`, when it is prose.
fn first_paragraph(body: &str) -> Option<String> {
    let paragraph = body
        .lines()
        .skip_while(|line| line.trim().is_empty())
        .take_while(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let prose = !paragraph.is_empty() && !paragraph.starts_with(['#', '`', '|', '[', '-', '<']);
    prose.then_some(paragraph)
}

/// `body` with every relative link target rewritten from where `ess generate` put the page to
/// [`DIR`], where every page sits beside the index: `../x.md` from a domain page and
/// `domains/x.md` from the index become `./x.md`, a bare `x.md` becomes `./x.md`. Anchors and
/// absolute URLs stay.
fn rewrite_links(body: &str, in_domains: bool) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(at) = rest.find("](") {
        out.push_str(&rest[..at + 2]);
        rest = &rest[at + 2..];
        let end = rest.find(')').unwrap_or(rest.len());
        let target = &rest[..end];
        let up = if in_domains { "../" } else { "domains/" };
        if target.starts_with('#')
            || target.contains("://")
            || target.starts_with("./")
            || target.starts_with('/')
        {
            out.push_str(target);
        } else {
            out.push_str("./");
            out.push_str(target.strip_prefix(up).unwrap_or(target));
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// Every Markdown link target in `body`, in order.
fn link_targets(body: &str) -> Vec<&str> {
    body.match_indices("](")
        .filter_map(|(at, _)| {
            let rest = &body[at + 2..];
            rest.find(')').map(|end| &rest[..end])
        })
        .collect()
}

/// One page as the site serves it: under [`DIR`] as `name`, from the ess page `source`.
struct Placed<'a> {
    name: String,
    source: &'a str,
    in_domains: bool,
    position: Option<u32>,
}

/// The pages of one `ess generate --kind docs` run, keyed by their path under its `docs/`, as the
/// files of [`DIR`]. Domain pages move up beside the index; links between the pages are rewritten
/// to match, and a link to anything else is refused.
pub fn site_pages(raw: &BTreeMap<String, String>) -> Result<Files> {
    let mut placed = Vec::new();
    ensure!(
        raw.contains_key("index.md"),
        "ess generated no index.md for the specification"
    );
    placed.push(Placed {
        name: "index.md".to_owned(),
        source: "index.md",
        in_domains: false,
        position: None,
    });
    let mut position = 0;
    for name in raw.keys() {
        if let Some(domain) = name.strip_prefix("domains/") {
            ensure!(
                domain.ends_with(".md") && !domain.contains('/'),
                "ess generated an unexpected domain file {name}"
            );
            position += 1;
            placed.push(Placed {
                name: domain.to_owned(),
                source: name,
                in_domains: true,
                position: Some(position),
            });
        }
    }
    ensure!(position > 0, "ess generated no domain pages");
    for name in SYSTEM_PAGES {
        ensure!(raw.contains_key(name), "ess generated no {name}");
        position += 1;
        placed.push(Placed {
            name: name.to_owned(),
            source: name,
            in_domains: false,
            position: Some(position),
        });
    }
    let known: BTreeSet<&str> = placed.iter().map(|page| page.source).collect();
    if let Some(stray) = raw.keys().find(|name| !known.contains(name.as_str())) {
        bail!(
            "ess generated {stray}, which a2a-docs does not place; add it to the specification pages"
        );
    }
    let names: BTreeSet<&str> = placed.iter().map(|page| page.name.as_str()).collect();

    let mut files = Files::new();
    files.insert("_category_.json".to_owned(), CATEGORY.as_bytes().to_vec());
    for page in &placed {
        let body = strip_ess_banner(&raw[page.source])
            .with_context(|| format!("ess page {}", page.source))?;
        let (heading, body) =
            split_title(body).with_context(|| format!("ess page {}", page.source))?;
        let body =
            rewrite_links(body, page.in_domains).replace("`ess generate`", "`task docs-generate`");
        for target in link_targets(&body) {
            if target.starts_with('#') || target.starts_with("https://") {
                continue;
            }
            let file = target
                .strip_prefix("./")
                .map(|file| file.split('#').next().unwrap_or(file));
            ensure!(
                file.is_some_and(|file| names.contains(file)),
                "ess page {} links to {target}, outside the specification pages",
                page.source
            );
        }
        let (title, label, description) = if page.name == "index.md" {
            (
                format!("{heading} specification"),
                "Overview".to_owned(),
                format!(
                    "The {heading} ESS specification: its domains, components and system graph, generated from ess/."
                ),
            )
        } else if page.in_domains {
            (
                format!("{heading} domain"),
                heading.to_owned(),
                first_paragraph(&body)
                    .with_context(|| format!("ess page {} has no summary", page.source))?,
            )
        } else {
            (
                heading.to_owned(),
                heading.to_owned(),
                first_paragraph(&body)
                    .with_context(|| format!("ess page {} has no summary", page.source))?,
            )
        };
        let content = front_matter(&Meta {
            title: &title,
            label: &label,
            description: &description,
            position: page.position,
        }) + &body;
        files.insert(page.name.clone(), content.into_bytes());
    }
    Ok(files)
}

/// The build directory `a2a-docs` writes its scratch under: `$CARGO_TARGET_DIR` when set, else
/// `<root>/target`.
pub fn build_dir(root: &Path, cargo_target_dir: Option<OsString>) -> PathBuf {
    cargo_target_dir.map_or_else(|| root.join("target"), PathBuf::from)
}

/// Tells apart the output directories one process asks for.
static OUTPUTS: AtomicU64 = AtomicU64::new(0);

/// The `--out` of `ess generate --kind docs`: `<build>/a2a-docs/<run>/docs`. ESS locks the parent
/// of its output exclusively and every ancestor shared, so the parent is a directory of this run's
/// own, never one another process may hold, such as the system temporary directory.
pub fn ess_output_dir(build: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    let n = OUTPUTS.fetch_add(1, Ordering::Relaxed);
    build
        .join("a2a-docs")
        .join(format!("{}-{nanos}-{n}", std::process::id()))
        .join("docs")
}

/// Every Markdown file under `dir`, keyed by its path relative to `base` with `/` separators.
fn read_tree(base: &Path, dir: &Path, into: &mut BTreeMap<String, String>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            read_tree(base, &path, into)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let relative = path
                .strip_prefix(base)?
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            into.insert(relative, fs::read_to_string(&path)?);
        }
    }
    Ok(())
}

/// Run `ess generate --kind docs` over `<root>/ess` into a scratch directory of this run's own and
/// return [`site_pages`] of its output.
pub fn pages(root: &Path, ess: &Path) -> Result<Files> {
    let spec = root.join(SYSTEM);
    let scratch = ess_output_dir(&build_dir(root, std::env::var_os("CARGO_TARGET_DIR")));
    let owned = scratch
        .parent()
        .context("the ESS output directory has no parent")?;
    if let Some(dir) = owned.parent() {
        fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    fs::create_dir(owned).with_context(|| format!("creating {}", owned.display()))?;
    let result = (|| {
        let output = Command::new(ess)
            .args(["generate", "--kind", "docs", "--path"])
            .arg(&spec)
            .arg("--out")
            .arg(&scratch)
            .output()
            .with_context(|| format!("running {}", ess.display()))?;
        if !output.status.success() {
            bail!(
                "{} generate --kind docs --path {} failed: {}",
                ess.display(),
                spec.display(),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let docs = scratch.join("docs");
        let mut raw = BTreeMap::new();
        read_tree(&docs, &docs, &mut raw)?;
        site_pages(&raw)
    })();
    let _ = fs::remove_dir_all(owned);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const BANNER: &str =
        "<!--\ngenerated from a2a v1\ndo not edit: regenerate with `ess generate`\n-->\n\n";

    fn raw() -> BTreeMap<String, String> {
        let mut raw = BTreeMap::new();
        raw.insert(
            "index.md".to_owned(),
            format!("{BANNER}# a2a v1\n\n## Graph\n\n- **[Tasks](domains/a2a-tasks.md)**\n- [the topology](topology.md)\n\nRegenerate it with `ess generate`.\n"),
        );
        raw.insert(
            "domains/a2a-tasks.md".to_owned(),
            format!("{BANNER}# Tasks\n\nTasks an agent runs.\n\n[Back to the index](../index.md).\n\n### `Tasks`\n\nSee [Task](#task).\n"),
        );
        for (name, title) in [
            ("interactions.md", "Interactions"),
            ("crossings.md", "Type crossings"),
            ("topology.md", "Topology"),
        ] {
            raw.insert(
                name.to_owned(),
                format!(
                    "{BANNER}# {title}\n\nWhat {title} says.\n\n[Back to the index](index.md).\n"
                ),
            );
        }
        raw
    }

    fn text(files: &Files, name: &str) -> String {
        String::from_utf8(files[name].clone()).unwrap()
    }

    #[test]
    fn domain_pages_move_beside_the_index_with_their_links() {
        let files = site_pages(&raw()).unwrap();
        assert_eq!(
            files.keys().collect::<Vec<_>>(),
            [
                "_category_.json",
                "a2a-tasks.md",
                "crossings.md",
                "index.md",
                "interactions.md",
                "topology.md"
            ]
        );
        let index = text(&files, "index.md");
        assert!(index.starts_with(
            "---\n# generated by a2a-docs, do not edit\ntitle: \"a2a v1 specification\"\n"
        ));
        assert!(index.contains("custom_edit_url: null\n---\n\n<!-- generated by a2a-docs, do not edit -->\n\n## Graph\n"));
        assert!(index.contains("](./a2a-tasks.md)"));
        assert!(index.contains("](./topology.md)"));
        assert!(index.contains("`task docs-generate`"));
        assert!(!index.contains("generated from a2a v1"));
        assert!(!index.contains("# a2a v1"));

        let tasks = text(&files, "a2a-tasks.md");
        assert!(tasks.contains("title: \"Tasks domain\"\nsidebar_label: \"Tasks\"\nsidebar_position: 1\ndescription: \"Tasks an agent runs.\"\n"));
        assert!(tasks.contains("[Back to the index](./index.md)"));
        assert!(!tasks.contains("\n# Tasks\n"));
        assert!(tasks.contains("### `Tasks`"));
        assert!(text(&files, "topology.md").contains("sidebar_position: 4\n"));
        assert!(text(&files, "topology.md").contains("[Back to the index](./index.md)"));
    }

    #[test]
    fn links_out_of_the_pages_and_unplaced_pages_are_refused() {
        let mut leaking = raw();
        leaking.insert(
            "crossings.md".to_owned(),
            format!("{BANNER}# Type crossings\n\nSee [the source](../../ess/system.yaml).\n"),
        );
        let error = site_pages(&leaking).unwrap_err().to_string();
        assert!(error.contains("outside the specification pages"), "{error}");

        let mut stray = raw();
        stray.insert(
            "glossary.md".to_owned(),
            format!("{BANNER}# Glossary\n\nWords.\n"),
        );
        let error = site_pages(&stray).unwrap_err().to_string();
        assert!(error.contains("glossary.md"), "{error}");

        let mut bare = raw();
        bare.insert(
            "topology.md".to_owned(),
            "# Topology\n\nNo banner.\n".to_owned(),
        );
        assert!(site_pages(&bare).is_err());
    }

    /// ESS locks the parent of its `--out` exclusively, so a parent shared with other processes,
    /// such as the system temporary directory, fails whenever another `ess` run holds it.
    #[test]
    fn ess_output_parent_is_owned_by_the_run() {
        let root = Path::new("/workspace");
        let build = build_dir(root, None);
        let first = ess_output_dir(&build);
        let second = ess_output_dir(&build);
        for out in [&first, &second] {
            let parent = out.parent().unwrap();
            assert_ne!(parent, std::env::temp_dir());
            assert!(
                parent.starts_with(root.join("target/a2a-docs"))
                    && parent != root.join("target/a2a-docs")
            );
        }
        assert_ne!(first.parent(), second.parent());
        assert_eq!(
            build_dir(root, Some("/shared/target".into())),
            Path::new("/shared/target")
        );
    }
}
