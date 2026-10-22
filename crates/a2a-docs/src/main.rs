#![forbid(unsafe_code)]
//! Generates the derivable part of the documentation site, checks it for drift, and binds a built
//! site to its source commit. The site build itself is Docusaurus, under `website/`.
//!
//! It owns `website/docs/reference/` and `website/static/downloads/` whole, and generates:
//!
//! - the specification reference (`website/docs/reference/specification/`), from
//!   `ess generate --kind docs` ([`spec`]);
//! - the HTTP+JSON route table (`reference/http-routes.md`), from `ess/http.yaml` and
//!   `ess/http-unmapped.tsv` ([`routes`]);
//! - the UNMAPPED marker index (`reference/unmapped.md`), from the comments of `ess/`
//!   ([`markers`]);
//! - the downloads (`website/static/downloads/`) and the page that lists them
//!   (`reference/downloads.md`) ([`downloads`]).
mod downloads;
mod markers;
mod page;
mod routes;
mod spec;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};

/// Generated files, keyed by name or by path.
pub type Files = BTreeMap<String, Vec<u8>>;

#[derive(Parser)]
#[command(
    about = "Generate and check the derived pages and downloads of the a2a documentation site"
)]
struct Cli {
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Regenerate the reference pages and downloads of website/ from the repository, or check them
    /// for drift.
    Generate {
        /// Write nothing; fail when a generated file is missing, stale or edited by hand, or an owned
        /// directory holds a file it does not generate.
        #[arg(long)]
        check: bool,
        /// Repository root.
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// The `ess` executable.
        #[arg(long, default_value = "ess")]
        ess: PathBuf,
    },
    /// Bind a built site to its source commit: write `.well-known/b10x-site.json` into it.
    Provenance {
        /// The built site, `website/build`.
        #[arg(long)]
        site: PathBuf,
        /// The full Git revision the site was built from.
        #[arg(long)]
        commit: String,
    },
}

/// The project-site publication record for one build of the `/a2a/` site.
fn site_manifest(commit: &str) -> Result<String> {
    ensure!(
        commit.len() == 40
            && commit.bytes().all(|b| b.is_ascii_hexdigit())
            && commit != "0".repeat(40),
        "commit must be a nonzero full Git revision"
    );
    Ok(format!(
        "{{\n  \"baseUrl\": \"{}\",\n  \"commit\": \"{commit}\",\n  \"repository\": \"a2a\",\n  \"schema\": \"b10x-project-site/v1\"\n}}",
        page::BASE_URL
    ))
}

fn write_provenance(site: &Path, commit: &str) -> Result<()> {
    let manifest = site_manifest(commit)?;
    ensure!(
        site.join("index.html").is_file(),
        "{} holds no built site",
        site.display()
    );
    fs::create_dir_all(site.join(".well-known"))?;
    fs::write(site.join(".well-known/b10x-site.json"), manifest)
        .context("writing site provenance")?;
    fs::write(site.join(".nojekyll"), "")?;
    Ok(())
}

/// The reference section of the site, relative to the repository root. `a2a-docs` owns it whole:
/// every file in it is generated, and anything else in it fails the check.
const REFERENCE: &str = "website/docs/reference";

/// Everything `a2a-docs` writes: the directories it owns whole, each with its files keyed by their
/// `/`-separated path inside it. Directory paths are relative to the repository root.
struct Outputs {
    dirs: BTreeMap<&'static str, Files>,
}

/// `path`, relative to the repository root, as a path inside `dir`.
fn inside(dir: &str, path: &str) -> Result<String> {
    path.strip_prefix(dir)
        .and_then(|rest| rest.strip_prefix('/'))
        .filter(|rest| !rest.is_empty())
        .map(str::to_owned)
        .with_context(|| format!("{path} is not inside {dir}"))
}

fn outputs(root: &Path, ess: &Path) -> Result<Outputs> {
    let specification = spec::pages(root, ess)?;
    let (downloads, downloads_page) = downloads::outputs(root)?;
    let mut pages = routes::page(root, &specification)?;
    pages.extend(markers::page(root)?);
    pages.extend(downloads_page);
    let section = inside(REFERENCE, spec::DIR)?;
    let mut reference = Files::new();
    for (name, content) in specification {
        reference.insert(format!("{section}/{name}"), content);
    }
    for (path, content) in pages {
        reference.insert(inside(REFERENCE, &path)?, content);
    }
    Ok(Outputs {
        dirs: BTreeMap::from([(REFERENCE, reference), (downloads::DIR, downloads)]),
    })
}

/// The files and subdirectories under `dir`, by `/`-separated path inside it: a file with its
/// content, a directory as `name/`. A directory none of `expected` lies in is listed, not entered.
fn tree(
    dir: &Path,
    prefix: &str,
    expected: &Files,
    found: &mut BTreeMap<String, Option<Vec<u8>>>,
) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
        if entry.file_type()?.is_dir() {
            let nested = format!("{name}/");
            if expected.keys().any(|path| path.starts_with(&nested)) {
                tree(&entry.path(), &nested, expected, found)?;
            } else {
                found.insert(nested, None);
            }
        } else {
            found.insert(name, Some(fs::read(entry.path())?));
        }
    }
    Ok(())
}

/// Every difference between the expected files and the files in `dir`, which they own.
fn drift(expected: &Files, dir: &Path) -> Result<Vec<String>> {
    let mut actual = BTreeMap::new();
    if dir.exists() {
        tree(dir, "", expected, &mut actual)?;
    }
    let mut problems = Vec::new();
    for (name, content) in expected {
        match actual.get(name) {
            None => problems.push(format!("missing {name}")),
            Some(found) if found.as_ref() != Some(content) => {
                problems.push(format!("stale or edited {name}"));
            }
            Some(_) => {}
        }
    }
    for name in actual.keys().filter(|name| !expected.contains_key(*name)) {
        problems.push(format!("not generated {name}"));
    }
    Ok(problems)
}

/// Every difference between the outputs and the repository, by directory.
fn check(root: &Path, outputs: &Outputs) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    for (relative, files) in &outputs.dirs {
        problems.extend(
            drift(files, &root.join(relative))?
                .into_iter()
                .map(|problem| format!("{relative}: {problem}")),
        );
    }
    Ok(problems)
}

/// Replace each owned directory with its outputs, so a file the generator no longer writes is gone.
fn write(root: &Path, outputs: &Outputs) -> Result<()> {
    for (relative, files) in &outputs.dirs {
        let dir = root.join(relative);
        if dir.exists() {
            fs::remove_dir_all(&dir).with_context(|| format!("clearing {relative}"))?;
        }
        for (name, content) in files {
            let path = dir.join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).with_context(|| format!("creating {relative}"))?;
            }
            fs::write(&path, content).with_context(|| format!("writing {relative}/{name}"))?;
        }
        println!("{relative}: wrote {} files", files.len());
    }
    Ok(())
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Action::Generate {
            check: checking,
            root,
            ess,
        } => {
            let outputs = outputs(&root, &ess)?;
            if checking {
                let problems = check(&root, &outputs)?;
                if !problems.is_empty() {
                    bail!(
                        "the generated documentation has drifted; run `task docs-generate`:\n  {}",
                        problems.join("\n  ")
                    );
                }
                let files: usize = outputs.dirs.values().map(Files::len).sum();
                println!("website: {files} generated files current");
            } else {
                write(&root, &outputs)?;
            }
        }
        Action::Provenance { site, commit } => {
            write_provenance(&site, &commit)?;
            println!("{}: bound to {commit}", site.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh directory under the workspace's build directory, not the system temporary one.
    fn scratch(name: &str) -> PathBuf {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let dir = spec::build_dir(&root, std::env::var_os("CARGO_TARGET_DIR"))
            .join("a2a-docs")
            .join(format!("test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample() -> Outputs {
        Outputs {
            dirs: BTreeMap::from([
                (
                    "website/static/downloads",
                    Files::from([
                        ("a.proto".to_owned(), b"A\n".to_vec()),
                        ("b.yaml".to_owned(), b"B\n".to_vec()),
                    ]),
                ),
                (
                    REFERENCE,
                    Files::from([
                        ("page.md".to_owned(), b"P\n".to_vec()),
                        ("section/inner.md".to_owned(), b"I\n".to_vec()),
                    ]),
                ),
            ]),
        }
    }

    #[test]
    fn pages_are_placed_inside_the_reference_directory_only() {
        assert_eq!(
            inside(REFERENCE, "website/docs/reference/unmapped.md").unwrap(),
            "unmapped.md"
        );
        assert_eq!(inside(REFERENCE, spec::DIR).unwrap(), "specification");
        assert!(inside(REFERENCE, "website/docs/referenced.md").is_err());
        assert!(inside(REFERENCE, "website/docs/index.md").is_err());
        assert!(inside(REFERENCE, REFERENCE).is_err());
    }

    #[test]
    fn provenance_names_the_commit_and_refuses_a_bad_one() {
        let commit = "e86f43d90b637fc2c1790f77d59efff2eb852386";
        let manifest = site_manifest(commit).unwrap();
        assert!(manifest.contains(&format!("\"commit\": \"{commit}\"")));
        assert!(manifest.contains("\"schema\": \"b10x-project-site/v1\""));
        assert!(manifest.contains("\"baseUrl\": \"/a2a/\""));
        assert!(manifest.contains("\"repository\": \"a2a\""));
        assert!(site_manifest("e86f43d").is_err());
        assert!(site_manifest(&"0".repeat(40)).is_err());
        assert!(site_manifest(&format!("{}\"", &commit[..39])).is_err());

        let site = scratch("provenance");
        assert!(write_provenance(&site, commit).is_err());
        fs::write(site.join("index.html"), "").unwrap();
        write_provenance(&site, commit).unwrap();
        let written = fs::read_to_string(site.join(".well-known/b10x-site.json")).unwrap();
        assert_eq!(written, manifest);
        assert!(site.join(".nojekyll").is_file());
        fs::remove_dir_all(site).unwrap();
    }

    #[test]
    fn a_missing_site_fails_the_check_naming_every_file() {
        let root = scratch("missing");
        assert_eq!(
            check(&root, &sample()).unwrap(),
            [
                "website/docs/reference: missing page.md",
                "website/docs/reference: missing section/inner.md",
                "website/static/downloads: missing a.proto",
                "website/static/downloads: missing b.yaml",
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn edited_downloads_and_pages_and_stray_files_fail_the_check() {
        let root = scratch("edited");
        let outputs = sample();
        write(&root, &outputs).unwrap();
        assert!(check(&root, &outputs).unwrap().is_empty());

        let downloads = root.join("website/static/downloads");
        fs::write(downloads.join("b.yaml"), "B, edited by hand\n").unwrap();
        fs::write(downloads.join("c.json"), "{}\n").unwrap();
        fs::create_dir(downloads.join("nested")).unwrap();
        let reference = root.join(REFERENCE);
        fs::write(reference.join("page.md"), "P!\n").unwrap();
        fs::write(reference.join("old.md"), "a page no longer generated\n").unwrap();
        fs::write(reference.join("section/stray.md"), "S\n").unwrap();
        fs::create_dir_all(reference.join("hand/written")).unwrap();
        fs::write(reference.join("hand/written/page.md"), "H\n").unwrap();
        assert_eq!(
            check(&root, &outputs).unwrap(),
            [
                "website/docs/reference: stale or edited page.md",
                "website/docs/reference: not generated hand/",
                "website/docs/reference: not generated old.md",
                "website/docs/reference: not generated section/stray.md",
                "website/static/downloads: stale or edited b.yaml",
                "website/static/downloads: not generated c.json",
                "website/static/downloads: not generated nested/",
            ]
        );

        write(&root, &outputs).unwrap();
        assert!(check(&root, &outputs).unwrap().is_empty());
        assert!(!reference.join("old.md").exists());
        assert!(!reference.join("hand").exists());
        assert!(!reference.join("section/stray.md").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
