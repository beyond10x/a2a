//! The UNMAPPED marker index: every `UNMAPPED` marker in the YAML comments of `ess/`, with its
//! line on GitHub, whether it names an ESS gap, and its first sentence.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{
    Files,
    page::{Meta, SOURCE, code, front_matter, inline},
};

/// The page, relative to the repository root.
pub const PAGE: &str = "website/docs/reference/unmapped.md";
/// The specification, relative to the repository root.
const SPEC: &str = "ess";
/// The longest summary a row shows before it is cut at a word.
const SUMMARY: usize = 300;
/// The shortest summary that ends at the first sentence.
const BRIEF: usize = 60;

/// One marker: where it is, whether it names an ESS gap, and its first sentence.
#[derive(Debug, PartialEq)]
pub struct Marker {
    pub line: usize,
    pub ess_gap: bool,
    pub summary: String,
}

/// The comment text of `line`, when the line is a YAML comment.
fn comment(line: &str) -> Option<&str> {
    line.trim_start().strip_prefix('#').map(str::trim)
}

/// The text after an `UNMAPPED` marker in `text`: `UNMAPPED` followed by `:` or ` (`. A mention
/// such as "UNMAPPED there" or "(UNMAPPED, see …)" is not a marker.
fn after_marker(text: &str) -> Option<&str> {
    text.match_indices("UNMAPPED").find_map(|(at, word)| {
        let rest = &text[at + word.len()..];
        (rest.starts_with(':') || rest.starts_with(" (")).then_some(rest)
    })
}

/// `text` up to the end of its first sentence, with the sentences after it while the summary is
/// shorter than [`BRIEF`] characters (a marker often opens with its topic alone, "push
/// delivery."), and at most [`SUMMARY`] characters.
fn first_sentence(text: &str) -> String {
    let end = text
        .match_indices(". ")
        .map(|(at, _)| at + 1)
        .find(|end| *end >= BRIEF)
        .unwrap_or(text.len());
    let sentence = &text[..end];
    if sentence.chars().count() <= SUMMARY {
        return sentence.to_owned();
    }
    let cut: String = sentence.chars().take(SUMMARY).collect();
    let cut = cut.rsplit_once(' ').map_or(cut.as_str(), |(head, _)| head);
    format!("{cut} …")
}

/// Every marker in one YAML file's comments. A marker's text runs on over the comment lines that
/// follow it at the same indentation, up to the next marker or the end of the comment.
pub fn markers(source: &str) -> Vec<Marker> {
    let lines: Vec<&str> = source.lines().collect();
    let mut found = Vec::new();
    for (n, line) in lines.iter().enumerate() {
        let Some(rest) = comment(line).and_then(after_marker) else {
            continue;
        };
        let indent = line.len() - line.trim_start().len();
        let mut text = rest.trim_start_matches(':').trim().to_owned();
        for next in &lines[n + 1..] {
            let Some(more) = comment(next) else { break };
            if next.len() - next.trim_start().len() != indent || after_marker(more).is_some() {
                break;
            }
            text.push(' ');
            text.push_str(more);
        }
        let ess_gap = text.starts_with("(ESS gap)");
        let text = text
            .strip_prefix("(ESS gap)")
            .map_or(text.as_str(), |rest| rest.trim_start_matches(':').trim());
        found.push(Marker {
            line: n + 1,
            ess_gap,
            summary: first_sentence(text),
        });
    }
    found
}

/// Every YAML file under `dir`, sorted.
fn yaml_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.is_dir() {
            found.extend(yaml_files(&path)?);
        } else if path.extension().is_some_and(|ext| ext == "yaml") {
            found.push(path);
        }
    }
    found.sort();
    Ok(found)
}

/// The page from the markers of each file, keyed by the file's path relative to the repository.
pub fn render(files: &[(String, Vec<Marker>)]) -> String {
    let all: Vec<&Marker> = files.iter().flat_map(|(_, found)| found).collect();
    let gaps = all.iter().filter(|marker| marker.ess_gap).count();
    let mut page = front_matter(&Meta {
        title: "UNMAPPED markers",
        label: "UNMAPPED markers",
        description: "Every UNMAPPED marker in the specification: what ESS cannot express yet, and what A2A leaves open.",
        position: Some(3),
    });
    page.push_str(&format!(
        "The specification marks with `UNMAPPED` each rule of A2A it does not hold, in a YAML \
         comment beside the place the rule belongs. `ess/` has {total} markers: {gaps} name a \
         construct ESS lacks (an ESS gap), {open} an open question. Each row links the marker's \
         line and shows its first sentence; the comment there carries the source quote and, for \
         an ESS gap, the refusal ESS gave.\n\n\
         The HTTP+JSON routes ESS cannot serve at A2A's path are on [HTTP+JSON \
         routes](./http-routes.md).\n",
        total = all.len(),
        open = all.len() - gaps,
    ));
    for (path, found) in files {
        if found.is_empty() {
            continue;
        }
        page.push_str(&format!(
            "\n## {}\n\n| Line | Kind | Marker |\n|---|---|---|\n",
            code(path)
        ));
        for marker in found {
            page.push_str(&format!(
                "| [{line}]({SOURCE}/{path}#L{line}) | {kind} | {summary} |\n",
                line = marker.line,
                kind = if marker.ess_gap {
                    "ESS gap"
                } else {
                    "open question"
                },
                summary = inline(&marker.summary),
            ));
        }
    }
    page
}

/// The page, keyed by its path relative to the repository root.
pub fn page(root: &Path) -> Result<Files> {
    let mut files = Vec::new();
    for path in yaml_files(&root.join(SPEC))? {
        let relative = path
            .strip_prefix(root)?
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        let source = fs::read_to_string(&path).with_context(|| format!("reading {relative}"))?;
        files.push((relative, markers(&source)));
    }
    Ok(Files::from([(
        PAGE.to_owned(),
        render(&files).into_bytes(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
types:
  # UNMAPPED (ESS gap): the exactly-one rule. ESS 0.67.0 validates it as a type invariant
  # and nothing more. A third sentence the summary leaves out.
  Part: {}
  # A comment that mentions (UNMAPPED, see components.yaml) and UNMAPPED there.
  # is read. UNMAPPED (ESS gap): the per-row boundary within the Agent's reads (which tasks one
  # agent may see); ESS grants a whole view.
    # UNMAPPED: A2A v1.0.1 is silent on a second create.
  # UNMAPPED: (ESS gap) a no-op delete emits `Deleted` on every run, whether a row existed or not. More.
id: 1 # UNMAPPED: not a comment line
";

    #[test]
    fn markers_are_comment_lines_with_a_marker_and_run_on_at_their_indentation() {
        assert_eq!(
            markers(SAMPLE),
            [
                Marker {
                    line: 2,
                    ess_gap: true,
                    summary: "the exactly-one rule. ESS 0.67.0 validates it as a type invariant and nothing more.".into()
                },
                Marker {
                    line: 6,
                    ess_gap: true,
                    summary: "the per-row boundary within the Agent's reads (which tasks one agent may see); ESS grants a whole view.".into()
                },
                Marker {
                    line: 8,
                    ess_gap: false,
                    summary: "A2A v1.0.1 is silent on a second create.".into()
                },
                Marker {
                    line: 9,
                    ess_gap: true,
                    summary: "a no-op delete emits `Deleted` on every run, whether a row existed or not.".into()
                },
            ]
        );
    }

    #[test]
    fn long_sentences_are_cut_at_a_word() {
        let long = format!("{} end.", "word ".repeat(100));
        let cut = first_sentence(&long);
        assert!(cut.ends_with(" …"));
        assert!(cut.chars().count() <= SUMMARY + 2);
    }

    #[test]
    fn the_page_counts_and_links_every_marker() {
        let page = render(&[
            ("ess/domains/tasks.yaml".into(), markers(SAMPLE)),
            ("ess/system.yaml".into(), Vec::new()),
        ]);
        assert!(page.contains("`ess/` has 4 markers: 3 name a construct ESS lacks"));
        assert!(page.contains("1 an open question"));
        assert!(page.contains("\n## `ess/domains/tasks.yaml`\n"));
        assert!(!page.contains("ess/system.yaml"));
        assert!(page.contains(
            "| [2](https://github.com/beyond10x/a2a/blob/main/ess/domains/tasks.yaml#L2) | ESS gap | the exactly-one rule. ESS 0.67.0 validates it as a type invariant and nothing more. |"
        ));
        assert!(page.contains("| open question | A2A v1.0.1 is silent on a second create. |"));
    }
}
