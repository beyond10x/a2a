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
| `ess/` | ESS source (`ess/29`), listed in `ess/ess-inputs.yaml`, pinned to `ess 0.67.0` | authored |
| `ess/http.yaml`, `ess/http-unmapped.tsv` | `ess-http/1` binding of A2A's HTTP+JSON paths, pinned to the specification's digest; the (method, path) differences from `upstream/openapi/` it cannot close, each with its reason | authored |
| `upstream/a2a.proto` | Normative A2A source, verbatim from tag v1.0.1 | `task upstream` |
| `upstream/openapi/a2a.openapi.yaml` | OpenAPI 3.0 of A2A's HTTP+JSON binding, derived from the proto. Upstream publishes none | `task upstream` |
| `upstream/buf.gen.openapiv2.yaml`, `upstream/LICENSE` | Derivation template; upstream Apache-2.0 licence | authored / `task upstream` |
| `generated/{openapi,asyncapi,schema}/` | ESS projections of `ess/` | `task projections` |
| `generated/openapi-http/` | ESS projection of `ess/` through `ess/http.yaml` (`--http`) | `task projections` |
| `generated/rust/a2a/` | Rust crate synthesized from `ess/` (model, contracts, `server` feature) | `task generate` |
| `crates/a2a-xtask/` | Repository checks (generated-crate drift, HTTP path parity) | authored |
| `crates/a2a-docs/` | Generator of the site's reference pages and downloads, and the build's provenance | authored |
| `website/` | Docusaurus site on `@beyond10x/docs-system`; `docs/concepts/`, `docs/guides/`, `docs/index.md` | authored |
| `website/docs/reference/`, `website/static/downloads/` | Specification pages, routes, marker index, downloads with `SHA256SUMS` | `task docs-generate` |
| `.engineering/` | AEP planning store; the plan lives here, not in chat or plan files | `aep` |

## Commands

| Command | Does |
|---|---|
| `task check` | The gate: validate `ess/`, projection and synthesis drift, fmt, clippy, test, build the generated crate |
| `task generate`, `task projections` | Rewrite `generated/` after a change to `ess/` |
| `task docs-generate` | Rewrite `website/docs/reference/` and `website/static/downloads/`; `task docs-check` (in `task check`) fails on a missing, stale or hand-edited file there, and on any file there it does not generate |
| `task website` | `npm ci` and `npm run build` in `website/`, as `.github/workflows/pages.yml` does. Not in `task check` |
| `task conform-interpreted` | Synthesized suite on the ESS interpreted target. Not a gate: 14 of 120 scenarios error on ESS defects |
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
