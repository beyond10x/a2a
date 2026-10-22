# AGENTS.md

The A2A (Agent2Agent) protocol v1.0.1, specified in ESS first. Everything else (projections, Rust
model, server, client, test kit, documentation) is derived from `ess/` or held to it.

Running code is Rust with clap derive. The documentation site is the only Node.

## Layout

| Path | What it is | Who writes it |
|---|---|---|
| `ess/` | ESS source (`ess/29`), listed in `ess/ess-inputs.yaml`, pinned to `ess 0.67.0` | authored |
| `upstream/a2a.proto` | Normative A2A source, verbatim from tag v1.0.1 | `task upstream` |
| `upstream/openapi/a2a.openapi.yaml` | OpenAPI 3.0 of A2A's HTTP+JSON binding, derived from the proto. Upstream publishes none | `task upstream` |
| `upstream/buf.gen.openapiv2.yaml`, `upstream/LICENSE` | Derivation template; upstream Apache-2.0 licence | authored / `task upstream` |
| `generated/{openapi,asyncapi,schema}/` | ESS projections of `ess/` | `task projections` |
| `generated/rust/a2a/` | Rust crate synthesized from `ess/` (model, contracts, `server` feature) | `task generate` |
| `crates/a2a-xtask/` | Repository checks (generated-crate drift) | authored |
| `.engineering/` | AEP planning store; the plan lives here, not in chat or plan files | `aep` |

## Commands

| Command | Does |
|---|---|
| `task check` | The gate: validate `ess/`, projection and synthesis drift, fmt, clippy, test, build the generated crate |
| `task generate`, `task projections` | Rewrite `generated/` after a change to `ess/` |
| `task conform-interpreted` | Synthesized suite on the ESS interpreted target. Not a gate: 14 of 120 scenarios error on ESS defects |
| `task upstream` | Fetch the pinned tarball, verify sha256, refresh `upstream/`. Needs `buf`, `protoc-gen-openapiv2`, `npx` |
| `task plan` | `aep plan artifact validate` |

## Rules

- Change `ess/` first, then regenerate. Never edit `generated/` by hand; `task check` fails on drift.
- `upstream/` changes only through `task upstream`. Moving the pin: edit `A2A_VERSION` and
  `A2A_SHA256` in `Taskfile.yml`, run `task upstream`, update `ess/` against the proto diff.
- The OpenAPI derivation uses grpc-gateway `protoc-gen-openapiv2` and `swagger2openapi`. gnostic
  `protoc-gen-openapi` rewrites `{id=*}` path templates to `/*` and is not used.
- `ess/` is authored from `upstream/a2a.proto` and the A2A specification text. An `UNMAPPED:` marker
  is an open question; resolve it from an upstream source, never by guessing.
- `generated/openapi/` is ESS's command-shaped HTTP surface, not A2A's wire paths.
  `upstream/openapi/` is A2A's.
- Every commit and push is `b10x-bot[bot]` through `b10x-gates bot`. The repository is enrolled in
  the common Gates policy (`.github/workflows/shared-gates.yml`).
- Nothing from a customer or employer outside beyond10x enters this repository.

## Releases

Releases are bare-version tags (`0.1.0`, no `v`); every crate inherits the workspace version and
nothing is on a registry.

- `CHANGELOG.md` follows Keep a Changelog 1.1.0. Changes collect under `## [Unreleased]`.
- A release commit on `main` sets `[workspace.package] version` in `Cargo.toml` and `Cargo.lock`,
  turns `## [Unreleased]` into `## [<version>] - <date>` under a new empty `## [Unreleased]`, and
  moves the link references at the bottom. `task release-check TAG=<version>` passes on it.
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
