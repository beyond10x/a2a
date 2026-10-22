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

Record each change under `## Unreleased` in `CHANGELOG.md`. A release moves those entries under its
version heading and tags the same version.
