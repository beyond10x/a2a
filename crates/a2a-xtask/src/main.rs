use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::{Parser, Subcommand};

const SPECIFICATION: &str = "ess";
const GENERATED: &str = "generated/rust/a2a";
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
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let root = repository_root();
    let result = match cli.command {
        Task::Generate => generate(&root),
        Task::Drift => drift(&root),
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
    Path::new(env!("CARGO_MANIFEST_DIR"))
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
