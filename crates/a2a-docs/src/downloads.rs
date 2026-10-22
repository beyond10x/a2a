//! The downloads the site serves: copies of the protocol definition and of the documents generated
//! from the specification under `website/static/downloads/`, a JSON Schema bundle, a `SHA256SUMS`
//! file, and the page that lists them with the assets of the latest GitHub Release.

use std::{fs, path::Path};

use anyhow::{Context, Result, bail, ensure};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::{
    Files,
    page::{BASE_URL, Meta, REPOSITORY, code, front_matter, source_link},
};

/// Where the downloads land, relative to the repository root. `a2a-docs` owns it whole.
pub const DIR: &str = "website/static/downloads";
/// The page that lists them, relative to the repository root.
pub const PAGE: &str = "website/docs/reference/downloads.md";
/// The JSON Schema files of `ess generate --kind schema`, relative to the repository root.
const SCHEMA: &str = "generated/schema/schema";
/// The bundle's file name.
const BUNDLE: &str = "a2a.schema.json";
const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

/// A download copied byte for byte from a repository file.
struct Copy {
    name: &'static str,
    source: &'static str,
    contents: &'static str,
}

const COPIES: [Copy; 5] = [
    Copy {
        name: "a2a.proto",
        source: "upstream/a2a.proto",
        contents: "A2A v1.0.1's normative protocol definition, verbatim from the upstream tag. © The A2A Project Authors, Apache-2.0.",
    },
    Copy {
        name: "a2a-upstream.openapi.yaml",
        source: "upstream/openapi/a2a.openapi.yaml",
        contents: "OpenAPI 3.0 of A2A's HTTP+JSON binding, derived from `a2a.proto` (upstream publishes none).",
    },
    Copy {
        name: "a2a-http.openapi.yaml",
        source: "generated/openapi-http/openapi/a2a-agent.yaml",
        contents: "OpenAPI of the specification through `ess/http.yaml`: A2A's paths where ESS can bind them.",
    },
    Copy {
        name: "a2a.openapi.yaml",
        source: "generated/openapi/openapi/a2a-agent.yaml",
        contents: "OpenAPI of the specification on ESS's command-shaped routes, not A2A's paths.",
    },
    Copy {
        name: "a2a.asyncapi.yaml",
        source: "generated/asyncapi/asyncapi/a2a-agent.yaml",
        contents: "AsyncAPI of the events the agent publishes.",
    },
];

/// A GitHub Release asset of `task release-artifacts`, by name with `{TAG}` for the version.
struct Asset {
    name: &'static str,
    contents: &'static str,
}

const ASSETS: [Asset; 7] = [
    Asset {
        name: "a2a-{TAG}-specification.tar.gz",
        contents: "`ess/`, `upstream/`, the OpenAPI, AsyncAPI and JSON Schema projections, `LICENSE` and `CHANGELOG.md`",
    },
    Asset {
        name: "a2a-{TAG}-json-schema.tar.gz",
        contents: "the JSON Schema files of `generated/schema/schema/`",
    },
    Asset {
        name: "a2a.proto",
        contents: "`upstream/a2a.proto`",
    },
    Asset {
        name: "a2a-{TAG}-upstream.openapi.yaml",
        contents: "`upstream/openapi/a2a.openapi.yaml`",
    },
    Asset {
        name: "a2a-{TAG}.openapi.yaml",
        contents: "`generated/openapi/`'s document",
    },
    Asset {
        name: "a2a-{TAG}.asyncapi.yaml",
        contents: "`generated/asyncapi/`'s document",
    },
    Asset {
        name: "SHA256SUMS",
        contents: "the SHA-256 of every other asset",
    },
];

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn size(bytes: usize) -> String {
    format!("{:.1} KiB", bytes as f64 / 1024.0)
}

/// Every schema of `ess generate --kind schema` as one JSON Schema 2020-12 compound document. Each
/// file is an embedded resource under `$defs`, keyed `<directory>.<name>`, with the `$id`
/// `schema/<directory>/<file>`; its `#/$defs/…` references resolve inside it, as in the file.
pub fn schema_bundle(files: &[(String, String, Vec<u8>)]) -> Result<Vec<u8>> {
    let mut defs = Map::new();
    for (directory, file, bytes) in files {
        let at = format!("{SCHEMA}/{directory}/{file}");
        let stem = file
            .strip_suffix(".schema.json")
            .with_context(|| format!("{at} is not a .schema.json file"))?;
        let Value::Object(schema) =
            serde_json::from_slice(bytes).with_context(|| format!("parsing {at}"))?
        else {
            bail!("{at} is not a JSON object");
        };
        ensure!(
            schema.get("$schema") == Some(&json!(DIALECT)),
            "{at} is not a JSON Schema 2020-12 document"
        );
        ensure!(!schema.contains_key("$id"), "{at} already has an $id");
        let mut embedded = Map::new();
        embedded.insert("$schema".to_owned(), json!(DIALECT));
        embedded.insert(
            "$id".to_owned(),
            json!(format!("schema/{directory}/{file}")),
        );
        embedded.extend(schema.into_iter().filter(|(key, _)| key != "$schema"));
        let key = format!("{directory}.{stem}");
        ensure!(
            defs.insert(key.clone(), Value::Object(embedded)).is_none(),
            "two schema files are bundled as {key}"
        );
    }
    ensure!(!defs.is_empty(), "{SCHEMA} holds no schema files");
    let bundle = json!({
        "$schema": DIALECT,
        "$id": format!("https://beyond10x.github.io{BASE_URL}downloads/{BUNDLE}"),
        "title": "A2A v1.0.1 JSON Schema bundle",
        "description": format!("Every JSON Schema of {SCHEMA}/ in the a2a repository, generated from its ESS specification. Each file is an embedded resource under $defs, keyed <directory>.<name>, with the $id schema/<directory>/<file>."),
        "$defs": defs,
    });
    Ok((serde_json::to_string_pretty(&bundle)? + "\n").into_bytes())
}

/// Every `*.schema.json` under [`SCHEMA`], as (directory, file, bytes), sorted.
fn schema_files(root: &Path) -> Result<Vec<(String, String, Vec<u8>)>> {
    let base = root.join(SCHEMA);
    let mut found = Vec::new();
    for directory in fs::read_dir(&base).with_context(|| format!("reading {SCHEMA}"))? {
        let directory = directory?;
        ensure!(
            directory.path().is_dir(),
            "{SCHEMA} holds a file outside a directory: {}",
            directory.file_name().to_string_lossy()
        );
        let name = directory.file_name().to_string_lossy().into_owned();
        for file in fs::read_dir(directory.path())? {
            let file = file?;
            let file_name = file.file_name().to_string_lossy().into_owned();
            found.push((name.clone(), file_name, fs::read(file.path())?));
        }
    }
    found.sort();
    Ok(found)
}

/// The version of the latest release: the first `## [x.y.z] - <date>` heading of the changelog.
pub fn latest_release(changelog: &str) -> Result<String> {
    changelog
        .lines()
        .find_map(|line| {
            let rest = line.strip_prefix("## [")?;
            let (version, after) = rest.split_once(']')?;
            let numeric = version.split('.').count() == 3
                && version
                    .split('.')
                    .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()));
            (numeric && after.starts_with(" - ")).then(|| version.to_owned())
        })
        .context("CHANGELOG.md has no `## [x.y.z] - <date>` release heading")
}

/// The downloads page.
fn render(files: &Files, release: &str) -> String {
    let mut page = front_matter(&Meta {
        title: "Downloads",
        label: "Downloads",
        description: "a2a.proto and the OpenAPI, AsyncAPI and JSON Schema documents generated from the specification, as on main and as released.",
        position: Some(1),
    });
    page.push_str(
        "The site serves these files as they are on `main`. Each is the repository file named \
         beside it, copied by `task docs-generate`; `task check` fails when a copy differs from its \
         source.\n\n| File | Size | Contents | Copied from |\n|---|---|---|---|\n",
    );
    let link = |name: &str| format!("[{}](pathname://{BASE_URL}downloads/{name})", code(name));
    for copy in &COPIES {
        page.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            link(copy.name),
            size(files[copy.name].len()),
            copy.contents,
            source_link(copy.source)
        ));
    }
    page.push_str(&format!(
        "| {} | {} | Every JSON Schema of `{SCHEMA}/` as one JSON Schema 2020-12 document: each file is an embedded resource under `$defs`, keyed `<directory>.<name>`. | [`{SCHEMA}/`]({REPOSITORY}/tree/main/{SCHEMA}) |\n",
        link(BUNDLE),
        size(files[BUNDLE].len()),
    ));
    page.push_str(&format!(
        "| {} | {} | The SHA-256 of every file above, for `sha256sum --check`. | |\n",
        link("SHA256SUMS"),
        size(files["SHA256SUMS"].len())
    ));
    page.push_str(&format!(
        "\n```text title=\"SHA256SUMS\"\n{}```\n",
        String::from_utf8_lossy(&files["SHA256SUMS"])
    ));
    page.push_str(&format!(
        "\n## Release {release}\n\nThe GitHub Release [{release}]({REPOSITORY}/releases/tag/{release}) \
         carries the specification as released, with its own `SHA256SUMS`. Earlier releases are \
         on [the releases page]({REPOSITORY}/releases).\n\n| Asset | Contents |\n|---|---|\n"
    ));
    for asset in &ASSETS {
        let name = asset.name.replace("{TAG}", release);
        page.push_str(&format!(
            "| [{}]({REPOSITORY}/releases/download/{release}/{name}) | {} |\n",
            code(&name),
            asset.contents
        ));
    }
    page
}

/// The downloads, keyed by file name under [`DIR`], and the page, keyed by its path relative to
/// the repository root.
pub fn outputs(root: &Path) -> Result<(Files, Files)> {
    let mut files = Files::new();
    for copy in &COPIES {
        let bytes =
            fs::read(root.join(copy.source)).with_context(|| format!("reading {}", copy.source))?;
        files.insert(copy.name.to_owned(), bytes);
    }
    files.insert(BUNDLE.to_owned(), schema_bundle(&schema_files(root)?)?);
    let sums: String = files
        .iter()
        .map(|(name, bytes)| format!("{}  {name}\n", hex_sha256(bytes)))
        .collect();
    files.insert("SHA256SUMS".to_owned(), sums.into_bytes());
    let changelog =
        fs::read_to_string(root.join("CHANGELOG.md")).context("reading CHANGELOG.md")?;
    let page = render(&files, &latest_release(&changelog)?);
    Ok((files, Files::from([(PAGE.to_owned(), page.into_bytes())])))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository() -> &'static Path {
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    #[test]
    fn the_bundle_embeds_each_schema_as_a_resource_with_its_own_id() {
        let schema = |title: &str| {
            format!("{{\"$schema\": \"{DIALECT}\", \"title\": \"{title}\", \"$ref\": \"#/$defs/x\", \"$defs\": {{\"x\": {{}}}}}}")
                .into_bytes()
        };
        let bundle = schema_bundle(&[
            (
                "commands".into(),
                "a2a.tasks.GetTask.schema.json".into(),
                schema("Get"),
            ),
            (
                "types".into(),
                "a2a.tasks.Part.schema.json".into(),
                schema("Part"),
            ),
        ])
        .unwrap();
        let bundle: Value = serde_json::from_slice(&bundle).unwrap();
        assert_eq!(bundle["$schema"], DIALECT);
        assert_eq!(
            bundle["$id"],
            "https://beyond10x.github.io/a2a/downloads/a2a.schema.json"
        );
        let get = &bundle["$defs"]["commands.a2a.tasks.GetTask"];
        assert_eq!(get["$id"], "schema/commands/a2a.tasks.GetTask.schema.json");
        assert_eq!(get["$ref"], "#/$defs/x");
        assert_eq!(
            get.as_object().unwrap().keys().take(3).collect::<Vec<_>>(),
            ["$schema", "$id", "title"]
        );
        assert_eq!(bundle["$defs"]["types.a2a.tasks.Part"]["title"], "Part");

        let draft7 = br#"{"$schema": "http://json-schema.org/draft-07/schema#"}"#.to_vec();
        assert!(schema_bundle(&[("types".into(), "a.schema.json".into(), draft7)]).is_err());
        assert!(schema_bundle(&[]).is_err());
    }

    #[test]
    fn the_latest_release_is_the_first_versioned_heading() {
        let changelog = "# Changelog\n\n## [Unreleased]\n\n## [0.2.0] - 2026-10-22\n\n## [0.1.0] - 2026-10-20\n";
        assert_eq!(latest_release(changelog).unwrap(), "0.2.0");
        assert!(latest_release("## [Unreleased]\n").is_err());
    }

    /// The page links release assets by name, so each name must be one `task release-artifacts`
    /// still writes: the text of the task that writes it.
    #[test]
    fn every_listed_release_asset_is_written_by_release_artifacts() {
        const WRITTEN_BY: [(&str, &str); 7] = [
            (
                "a2a-{TAG}-specification.tar.gz",
                "BUNDLE: a2a-{{.TAG}}-specification",
            ),
            (
                "a2a-{TAG}-json-schema.tar.gz",
                "a2a-{{.TAG}}-json-schema.tar.gz",
            ),
            ("a2a.proto", "{{.OUT}}/a2a.proto"),
            (
                "a2a-{TAG}-upstream.openapi.yaml",
                "a2a-{{.TAG}}-upstream.openapi.yaml",
            ),
            ("a2a-{TAG}.openapi.yaml", "a2a-{{.TAG}}.openapi.yaml"),
            ("a2a-{TAG}.asyncapi.yaml", "a2a-{{.TAG}}.asyncapi.yaml"),
            ("SHA256SUMS", "> SHA256SUMS"),
        ];
        let taskfile = fs::read_to_string(repository().join("Taskfile.yml")).unwrap();
        let (_, task) = taskfile.split_once("  release-artifacts:").unwrap();
        let task = task.split("\n  plan:").next().unwrap();
        for asset in &ASSETS {
            let (_, text) = WRITTEN_BY
                .iter()
                .find(|(name, _)| *name == asset.name)
                .unwrap_or_else(|| panic!("no task text names {}", asset.name));
            assert!(
                task.contains(text),
                "release-artifacts no longer writes {}",
                asset.name
            );
        }
    }

    #[test]
    fn the_downloads_are_the_repository_files_and_sums_cover_them() {
        let (files, page) = outputs(repository()).unwrap();
        let literal = [
            ("a2a.proto", "upstream/a2a.proto"),
            (
                "a2a-upstream.openapi.yaml",
                "upstream/openapi/a2a.openapi.yaml",
            ),
            (
                "a2a-http.openapi.yaml",
                "generated/openapi-http/openapi/a2a-agent.yaml",
            ),
            (
                "a2a.openapi.yaml",
                "generated/openapi/openapi/a2a-agent.yaml",
            ),
            (
                "a2a.asyncapi.yaml",
                "generated/asyncapi/asyncapi/a2a-agent.yaml",
            ),
        ];
        assert_eq!(literal.len(), COPIES.len());
        for (name, source) in literal {
            assert!(
                files[name] == fs::read(repository().join(source)).unwrap(),
                "the download {name} is not {source}"
            );
            let copy = COPIES.iter().find(|copy| copy.name == name).unwrap();
            assert_eq!(
                copy.source, source,
                "the page names another source for {name}"
            );
        }
        let sums = String::from_utf8(files["SHA256SUMS"].clone()).unwrap();
        assert_eq!(sums.lines().count(), files.len() - 1);
        for (name, bytes) in &files {
            if name != "SHA256SUMS" {
                assert!(sums.contains(&format!("{}  {name}\n", hex_sha256(bytes))));
            }
        }
        let page = String::from_utf8(page[PAGE].clone()).unwrap();
        assert!(page.contains("[`a2a.proto`](pathname:///a2a/downloads/a2a.proto)"));
        assert!(page.contains("/releases/download/"));
    }
}
