---
format: aep.planning-md/3
id: review-result:adversary-docs-site-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:docs-site
relations:
- reviews: story:docs-site
revision: 1
---
unit: story:docs-site (adversary)
verdict: red (strongest: NEEDS-CHANGE)
cases: executed 35→38, red 1
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 directory, ~/.cache/a2a-gen/wave2/adv-docs-site/ (scratch, logs, copies, build-mut cargo dir)
needs-coordinator: no

Covers worktree `~/.local/state/worktree/trees/b10x/a2a/a2a-w2-docs-site`, detached at wave/2 91efc36, with the unit's uncommitted changes.

## 1. Diff

`git --no-pager diff --stat` (tracked): AGENTS.md +16 -2, CHANGELOG.md +11, Cargo.lock +217, README.md +10 -1, Taskfile.yml +18. All of that is the implementor's. The tracked diff cannot show my one file, because it sits in the untracked `crates/a2a-docs/`:

| Path | Kind |
|---|---|
| `crates/a2a-docs/tests/adv_docs_site.rs` (new, 3 cases) | test file |

No implementation file changed. Every plant and mutation was made in scratch copies.

## 2. Cases added (`crates/a2a-docs/tests/adv_docs_site.rs`)

| Case | Asserts | Now |
|---|---|---|
| `control_the_committed_site_passes_the_check` | `generate --check` passes on an unchanged scratch copy of the inputs | green |
| `a_page_in_reference_that_a2a_docs_does_not_generate_fails_the_check` | a page in `website/docs/reference/` that a2a-docs does not generate fails the check | **red** |
| `each_download_is_the_repository_file_its_name_says` | the 5 copied downloads equal literal source paths | green; red on mutant M1 |

The red case was run on its own before the suite (`cargo test -p a2a-docs --test adv_docs_site a_page_in_reference…`), exit 101:

```
generate --check passed with website/docs/reference/routes.md, which it does not generate; stdout: website: 17 generated files current
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out
```

Mutant M1 was built in a scratch copy, not in the tree. It exchanges the sources of `a2a-http.openapi.yaml` and `a2a.openapi.yaml` in `COPIES` (`downloads.rs:46,51`):

| Step | Result |
|---|---|
| crate unit tests | 17 passed, exit 0 |
| `generate` | wrote the files |
| `generate --check` | "17 generated files current", exit 0 |
| adversary literal case | "website/static/downloads/a2a-http.openapi.yaml is not generated/openapi-http/openapi/a2a-agent.yaml", exit 101 |

Log: `mutant-copies.log`.

## 3. Suite, run after the cases existed

| Run | Result |
|---|---|
| `b10x-build-slot task check` | exit 201 (`gate.log`). spec, projections-drift, drift, **http-paths**, **docs-check** ("17 generated files current"), fmt and clippy are green; `cargo test` is red on the case above |
| `cargo test --workspace --locked --no-fail-fast` | exit 101 (`suite.log`) |

`suite.log` by test binary:

| Binary | Passed | Failed |
|---|---|---|
| a2a-docs | 17 | 0 |
| adv_docs_site | 2 | 1 |
| a2a-xtask | 13 | 0 |
| adv_http_binding | 3 | 0 |
| adv_http_binding_pass2 | 2 | 0 |

Total 38. The before count of 35 is from the implementor's report. `ess/` is unchanged, so the conform counts were not rerun.

## 4. Findings

| # | file:line | Verdict | Origin | Severity | Finding | What reaches it |
|---|---|---|---|---|---|---|
| 1 | crates/a2a-docs/src/main.rs:146 | NEEDS-CHANGE | introduced | warning | The check owns only `reference/specification/` and `downloads/` as whole directories. Other files in `reference/` are compared by name only, so an extra or leftover page there passes, unlike the claim at AGENTS.md:23,32 | `write()` (main.rs:165) never removes a page the generator stopped writing, and `sidebars.ts:22` autogenerates Reference from the directory, so the leftover page is published. Fix: own `website/docs/reference/` whole, or report every file in it that is not an output |
| 2 | crates/a2a-docs/src/downloads.rs:355 | CONFIRMED | introduced | note | Self-referential test: it compares each download with the source the `COPIES` table names, so exchanging two sources (M1) passes the unit tests and `generate --check` | Only a code edit reaches it. The literal-pair case added in part 2 catches it |
| 3 | website/docs/concepts/coverage.md:21 | CONFIRMED | introduced | note | "each error's summary carries its code" is false for `a2a.tasks.TaskStateConflict` (ess/domains/tasks.yaml:493), whose summary has no code | Readers of the Coverage page. Fix: "each A2A error's summary" |

## 5. Attacked, could not break (each check was planted red first)

| Check | Result |
|---|---|
| Drift, 30 plants in scratch copies | All red with the correct message: edits to every spec page, `_category_.json`, http-routes, unmapped, the downloads page and each of the 7 downloads; a deleted download; stray files in `downloads/` and `specification/`; stale `ess/http.yaml`, `http-unmapped.tsv`, `a2a.proto`, upstream OpenAPI, each `generated/` projection, a schema file, an ess marker comment and a new CHANGELOG release. The only exception is the stray page in `reference/` (finding 1). `plants.log` |
| Downloads byte equality | `a2a.proto` = the upstream tarball's `specification/a2a.proto`. The 4 OpenAPI/AsyncAPI copies equal their sources (`cmp`). Bundle: 89/89 resources equal their files apart from the added `$id`; 401/401 `$ref` resolve inside their own resource. `sha256sum --check --strict` OK |
| Built site | `build/downloads` equals `static/downloads`. 35 distinct `/a2a/` links, 0 broken. All 7 release asset links exist on GitHub release 0.2.0 (`gh release view`, read-only) |
| CHANGELOG parse | Unreleased and link references are skipped. Pre-releases cannot occur, because `release-check` refuses tags that are not bare versions. The AGENTS.md release step is true: plant "stale CHANGELOG release" makes the check red |
| Workflows | Same as loom apart from the ess install, which a2a-docs tests do not need. `project-site.yml@fb4024e…` pinned. Publishing requires the bot as actor and triggering actor, the `beyond10x/a2a` repository, the main branch, a push event and a successful build. The manifest has the fields that project-site.yml:172-178 checks. Nothing publishes from a PR or a non-bot push. As in loom, `pages.yml` does not run `docs-check` before uploading |
| Committed content | 0 home-directory or absolute paths, 0 secret patterns. The only outside-company names are in upstream verbatim text (`google.api` imports and an example). All 9 generated pages carry both headers and `custom_edit_url: null` |
| Hand-written claims | 37 markers (36 ESS gap, 1 open question) and routes 9/22, 13 and 12 match my independent count. Sections 3, 4.1-4.4, 5, 5.4, 9 and 13.1 exist upstream. The three gaps named in coverage.md are markers at components.yaml:10, 59 and 74 |

## 6. Paths written outside the worktree

All under `~/.cache/a2a-gen/wave2/adv-docs-site/`:

| Kind | Paths |
|---|---|
| Scripts | `mkcopy.sh`, `chk.sh`, `plants.sh`, `mutant-copies.sh`, `bundle-eq.sh`, `bundle-eq-plant.sh`, `refs.sh`, `links.sh` |
| Logs | `plants.log`, `mutant-copies.log`, `red-1-stray-reference.log`, `gate.log`, `suite.log` |
| Copies | `base/`, `plant/`, `mut/`, each 4.8M |
| Site and bundle | `build-plant/` (8.2M), `bundle-plant.json`, `bundle-plant2.json`, `upstream.routes`, `openapi-http.routes`, `u.n`, `p.n` |
| Cargo dirs | `build-mut/` (163M, cargo target of the mutant) and `build/` (ess scratch). Not deleted by hand |

Outside that directory:
- `~/.cache/b10x-target/a2a/tmp/adv-docs-site/` (empty).
- Cargo artifacts in `~/.cache/b10x-target/a2a`.
- This report.

## 7. Findings block

```findings
- {file: "crates/a2a-docs/src/main.rs", line: 146, category: "acceptance", severity: "warning", verdict: "NEEDS-CHANGE", origin: "introduced", message: "generate --check owns only reference/specification/ and downloads/ whole, so a page in website/docs/reference/ that a2a-docs does not generate (a renamed or dropped page that write() leaves behind, or a hand-added one) passes and is published by the autogenerated Reference sidebar, contrary to AGENTS.md:23,32"}
- {file: "crates/a2a-docs/src/downloads.rs", line: 355, category: "mutant", severity: "note", verdict: "CONFIRMED", origin: "introduced", message: "the downloads test compares each download with the source the COPIES table names, so exchanging the sources of a2a-http.openapi.yaml and a2a.openapi.yaml passes all 17 unit tests and generate --check after regeneration"}
- {file: "website/docs/concepts/coverage.md", line: 21, category: "judgement", severity: "note", verdict: "CONFIRMED", origin: "introduced", message: "the claim that each error's summary carries its code is false for a2a.tasks.TaskStateConflict (ess/domains/tasks.yaml:493), which has no code"}
```
