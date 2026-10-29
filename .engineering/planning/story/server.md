---
format: aep.planning-md/3
id: story:server
kind: story
status: implemented
title: 'a2a-server: Rust handler over the generated contracts'
summary: crates/a2a-server implements the generated Behavior traits and serves A2A's HTTP+JSON binding; it takes every model type from generated/rust/a2a and defines none.
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:http-binding
- depends_on: story:streaming-binding
- depends_on: story:resolve-unmapped
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/a2a-server
- confidence: cited
  path: crates/a2a-xtask
- confidence: cited
  path: ess/domains/tasks.yaml
- confidence: cited
  path: ess/http.yaml
- confidence: cited
  path: generated/rust/a2a
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T10:28:06Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-10-22T10:28:06Z", actor: "human:timo", revision: 8}
- {from: "active", to: "implemented", at: "2026-10-29T11:06:31Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"test_result":1,"review_outcome":3,"verification":1}}}
---
## Outcome

a2a-server: Rust handler over the generated contracts.

## Acceptance

crates/a2a-server implements the generated Behavior traits and serves A2A's HTTP+JSON binding; it takes every model type from generated/rust/a2a and defines none.

## Scope

Read at wave close from `git diff --stat 5fed063 60bad69` (unit commit 60bad69, merged into
wave/3-int as 429b83d). The unit was written on 2026-10-22 on 6520aef (ESS 0.69.x) and ported onto
main 5fed063; `generated/`, the reference pages and the downloads were regenerated with the pinned
ESS 0.75.0. The inferred `Cargo.toml` line did not land: the workspace takes `crates/*`.

- `crates/a2a-server/` (new: `src/` 10 files, `tests/` 20 test targets and `tests/common/`)
- `ess/domains/tasks.yaml`, `ess/http.yaml`
- `crates/a2a-xtask/src/main.rs` (synthesis takes `--http ess/http.yaml`)
- `Taskfile.yml`, `Cargo.lock`, `AGENTS.md`, `README.md`, `CHANGELOG.md`
- `generated/rust/a2a/`, `generated/openapi/`, `generated/openapi-http/`, `generated/asyncapi/`,
  `generated/schema/`
- `website/docs/reference/`, `website/static/downloads/`

## Decisions

Coordinator decisions after review-result:security-server-pass-1 (2026-10-22):

1. The request-head deadline runs from connect or from the previous answer; arriving bytes do not reset it. At the cap, any connection waiting for a request head may be evicted.
2. Every request the intake admits stays answerable and readable: replies and stored reads never re-parse under a limit the intake did not enforce.
3. Followers are bounded: `--max-subscribers-per-task` (default 16) and `--max-subscribers` (default half of `--max-connections`). Over a bound the stream is refused with 429 RESOURCE_EXHAUSTED.
4. ListTasks orders by key (timestamp, id) and builds rows only for the returned page.
5. Storage caps: `--max-tasks` per tenant (10000), `--max-history` per task (1000), `--max-push-configs` per task (16). Over a cap: 429 RESOURCE_EXHAUSTED, nothing stored.
6. The generated 431 and 411 answers pass through unchanged.
7. The extended agent card needs a bearer token from `--extended-card-token-file`, compared in constant time; otherwise 401 UNAUTHENTICATED. `--extended-card` without a token file is refused at startup.
