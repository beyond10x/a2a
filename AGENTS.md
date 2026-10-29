# AGENTS.md

The A2A (Agent2Agent) protocol v1.0.1, specified in ESS first. Everything else (projections, Rust
model, server, client, test kit, documentation) is derived from `ess/` or held to it.

Running code is Rust with clap derive. The documentation site is the only Node.

## Serves

- **O2**, decisions as data, with evidence: A2A is an ESS specification first, and every artifact
  is generated from it or checked against it.
- **O1**, governed reach: agent-to-agent calls go through a contract whose refusals are declared.
- **O6**, self-improvement: ESS gaps found here go back to ESS.

## Layout

| Path | What it is | Who writes it |
|---|---|---|
| `ess/` | ESS source (`ess/29`), listed in `ess/ess-inputs.yaml`, pinned to `ess 0.75.0` | authored |
| `ess/http.yaml`, `ess/http-unmapped.tsv` | `ess-http/1` binding of A2A's HTTP+JSON paths, pinned to the specification's digest; the (method, path) differences from `upstream/openapi/` it cannot close, each with its reason | authored |
| `upstream/a2a.proto` | Normative A2A source, verbatim from tag v1.0.1 | `task upstream` |
| `upstream/openapi/a2a.openapi.yaml` | OpenAPI 3.0 of A2A's HTTP+JSON binding, derived from the proto. Upstream publishes none | `task upstream` |
| `upstream/buf.gen.openapiv2.yaml`, `upstream/LICENSE` | Derivation template; upstream Apache-2.0 licence | authored / `task upstream` |
| `generated/{openapi,asyncapi,schema}/` | ESS projections of `ess/` | `task projections` |
| `generated/openapi-http/` | ESS projection of `ess/` through `ess/http.yaml` (`--http`) | `task projections` |
| `generated/rust/a2a/` | Rust crate synthesized from `ess/` (model, contracts, `server` feature) | `task generate` |
| `crates/a2a-xtask/` | Repository checks (generated-crate drift, HTTP path parity) | authored |
| `crates/a2a-docs/` | Generator of the site's reference pages and downloads, and the build's provenance | authored |
| `crates/a2a-server/` | A2A's HTTP+JSON binding over `generated/rust/a2a` (`server` feature): the 12 obligations in memory, every upstream route, SSE streams, and the `a2a-server` echo agent; `InProcess` runs the same obligations through the generated `System` without HTTP. Declares no type under a name the generated crate exports, at any visibility or qualifier; `tests/defines_none.rs` fails if it does | authored |
| `crates/a2a-client/` | Typed client of A2A's HTTP+JSON binding over `generated/rust/a2a` (`server` feature codecs): one method per operation, `tenant` selecting the `/{tenant}/` route, `google.rpc.Status` errors as `Error::Status` with a `Reason`, streams as `Events` iterators. HTTP/1.1 over `std::net::TcpStream`, `http://` only. Declares no type under a name the generated crate exports; `tests/defines_none.rs` fails if it does, and `tests/routes.rs` if the (method, path) pairs it sends differ from `upstream/openapi/` | authored |
| `crates/a2a-conformance/` | Interpreter of an `ess-conformance/52` suite against a `Target` (writes `ess-conformance-results/1`), the in-process target over `a2a-server`'s own obligations, and the gate `task conform` holds the report to | authored |
| `crates/a2a-conformance/unsupported.tsv` | The scenarios allowed to come out `unsupported`, one per line: scenario id, a tab, the exact reason the run gives. The gate fails on one missing, one extra or a changed reason | authored |
| `website/` | Docusaurus site on `@beyond10x/docs-system`; `docs/concepts/`, `docs/guides/`, `docs/index.md` | authored |
| `website/docs/reference/`, `website/static/downloads/` | Specification pages, routes, marker index, downloads with `SHA256SUMS` | `task docs-generate` |
| `.engineering/` | AEP planning store; the plan lives here, not in chat or plan files | `aep` |

## Commands

| Command | Does |
|---|---|
| `task check` | The gate: validate `ess/`, projection and synthesis drift, `task conform`, fmt, clippy, test, build the generated crate |
| `task generate`, `task projections` | Rewrite `generated/` after a change to `ess/` |
| `task docs-generate` | Rewrite `website/docs/reference/` and `website/static/downloads/`; `task docs-check` (in `task check`) fails on a missing, stale or hand-edited file there, and on any file there it does not generate |
| `task website` | `npm ci` and `npm run build` in `website/`, as `.github/workflows/pages.yml` does. Not in `task check` |
| `cargo run -p a2a-server -- --listen 127.0.0.1:8080 [--card <json>] [--extended-card <json> --extended-card-token-file <path>] [--max-connections 256] [--request-timeout 10] [--max-subscribers-per-task 16] [--max-subscribers <half of --max-connections>] [--max-tasks 10000] [--max-history 1000] [--max-push-configs 16]` | Serve the echo agent; the card at `/.well-known/agent-card.json`. Over the connection cap a connection is closed; a request not in within the timeout (seconds) is answered 408. A stream, task, history message or push config over its bound is answered 429 and nothing is kept. `GET /extendedAgentCard` answers 401 without `Authorization: Bearer <token>` (the token file's content), and `--extended-card` requires the token file. The token is never logged |
| `task conform` | Synthesize the suite into `~/.cache/a2a-gen/conform/` (removed first), run it against `a2a-server`'s obligations in process (`cargo run -p a2a-conformance -- run`), write `report.json` with `ess verify conform report`, then `a2a-conformance gate`: fails unless 0 failed, 0 error and the unsupported scenarios are exactly those `crates/a2a-conformance/unsupported.tsv` names, with their reasons. In `task check` |
| `task conform-interpreted` | Synthesized suite on the ESS interpreted target. Not a gate; on ESS 0.75.0, 177 of 177 scenarios pass |
| `task upstream` | Fetch the pinned tarball, verify sha256, refresh `upstream/`. Needs `buf`, `protoc-gen-openapiv2`, `npx` |
| `task plan` | `aep plan artifact validate` |

## Rules

- Change `ess/` first, then regenerate. Never edit `generated/` by hand; `task check` fails on drift.
- After `task projections` or a change to `ess/`, `upstream/` or a released version, run
  `task docs-generate`. Never edit `website/docs/reference/` or `website/static/downloads/` by
  hand. Hand-written site pages are concepts and guides only, and their commands are ones that were
  run.
- The site is <https://beyond10x.github.io/a2a/>. `pages.yml` builds it and uploads the build on a
  push to `main`; `b10x-docs-site.yml` publishes that exact build through
  `beyond10x/website`'s `project-site.yml`.
- `upstream/` changes only through `task upstream`. Moving the pin: edit `A2A_VERSION` and
  `A2A_SHA256` in `Taskfile.yml`, run `task upstream`, update `ess/` against the proto diff.
- The OpenAPI derivation uses grpc-gateway `protoc-gen-openapiv2` and `swagger2openapi`. gnostic
  `protoc-gen-openapi` rewrites `{id=*}` path templates to `/*` and is not used.
- `ess/` is authored from `upstream/a2a.proto` and the A2A specification text. An `UNMAPPED:` marker
  is an open question; resolve it from an upstream source, never by guessing.
- `generated/openapi/` is ESS's command-shaped HTTP surface, not A2A's wire paths.
  `upstream/openapi/` is A2A's; `generated/openapi-http/` is ESS's projection of it, and
  `task http-paths` (in `task check`) fails on any (method, path) difference
  `ess/http-unmapped.tsv` does not name, and on a named one that no longer differs.
- `ess/http.yaml` pins the specification's `source_digest`. After any change to `ess/`'s
  specification files, `ess specify http validate` refuses it with `ESS-HTTP-002`, naming the new
  digest: copy that digest into `ess/http.yaml`.
- A task belongs to the tenant of the route that created it; a plain route creates and sees only
  tenant-less tasks. Every task read, list, cancel, subscribe and push-config operation answers
  another tenant's task exactly as an unknown id (`TaskNotFound`).
- Every commit and push is `b10x-bot[bot]` through `b10x-gates bot`. The repository is enrolled in
  the common Gates policy (`.github/workflows/shared-gates.yml`).
- Nothing from a customer or employer outside beyond10x enters this repository.

## Releases

Releases are bare-version tags (`0.1.0`, no `v`); every crate inherits the workspace version and
nothing is on a registry.

- `CHANGELOG.md` follows Keep a Changelog 1.1.0. Changes collect under `## [Unreleased]`.
- A release commit on `main` sets `[workspace.package] version` in `Cargo.toml` and `Cargo.lock`,
  turns `## [Unreleased]` into `## [<version>] - <date>` under a new empty `## [Unreleased]`, and
  moves the link references at the bottom. It also carries `task docs-generate`'s output: the
  Downloads page names the newest released version from `CHANGELOG.md`, so without it
  `task check` fails. `task release-check TAG=<version>` passes on it.
- The tag is the bot's, on that commit:
  `b10x-gates bot --repository beyond10x/a2a -- tag -a <version> -m "a2a <version>" <commit>`, then
  `b10x-gates bot --repository beyond10x/a2a -- push origin <version>`.
- The GitHub Release is the bot's, created right after the tag push:
  `b10x-gates api --method POST --path /repos/beyond10x/a2a/releases --input <request.json> …` with
  `tag_name` and `name` set to the version and a body that passes `b10x-gates scan-text`: 1–2 lines,
  then `### Breaking` and `### Changed` (at most 8 items each), then a link to the CHANGELOG entry.
- `.github/workflows/release.yml` runs on the tag: `release-check`, then `check.yml`, then
  `task release-artifacts`, and uploads the assets to the bot's Release once it exists (it waits up
  to 30 minutes). It never creates the Release.
- A release is finished when the tag, the green `release` run and the Release with every asset of
  `task release-artifacts` are verified. Never move or delete a pushed tag; fix forward.
