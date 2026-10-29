---
format: aep.planning-md/3
id: story:testkit
kind: story
status: implemented
title: 'a2a-testkit: assertions harness for any A2A endpoint'
summary: 'crates/a2a-testkit: CLI and library that run the wire-observable conformance scenarios and assertion helpers against an A2A endpoint by URL through a2a-client; green on a2a-server, red on a planted defect.'
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:conformance-gate
- depends_on: story:client
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: README.md
- confidence: cited
  path: crates/a2a-testkit
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T11:22:00Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-10-29T11:22:00Z", actor: "human:timo", revision: 9}
- {from: "active", to: "implemented", at: "2026-10-29T14:04:08Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Outcome

Anyone who runs an A2A agent checks it from outside: `a2a-testkit run --url <base>` runs the
conformance scenarios that are observable over the HTTP+JSON binding against that endpoint, and
the library's assertion helpers let a Rust test state what an endpoint must answer. It reaches
`http://` endpoints that serve the HTTP+JSON binding; `https://` (TLS) and the JSON-RPC binding
are a later milestone, and an `https://` URL is refused as `a2a-client` refuses it.

## Acceptance

1. `crates/a2a-testkit` implements `a2a_conformance::Target` (story:conformance-gate) over
   `a2a-client` (story:client) for an endpoint given by base URL. Its mapping from an ESS
   command or view to a client call is derived from `ess/http.yaml`; the caller's tenant comes
   from the scenario's input. A step that cannot be observed on the wire (a published event, a
   forced external outcome, a command or read by an actor other than the client, a view A2A has
   no operation for) answers `TargetError::Unsupported` with that reason, so its scenario is
   `unsupported` and never passes. So is a scenario that needs state the wire cannot reset.
2. The CLI `a2a-testkit run --url <base> --suite <suite.json> [--results <out>]` (clap derive)
   prints one line per scenario and the counts, writes `ess-conformance-results/1` with
   `--results`, and exits 0 only when no scenario is `failed` or `error`.
3. The library exposes assertion helpers over a connected endpoint: `expect_card`,
   `expect_task_state`, `expect_error_reason`, `expect_stream_states` and
   `expect_push_config_roundtrip`. Each returns a `Mismatch` naming the expected and the observed
   value instead of panicking.
4. A test starts `a2a-server` with the echo agent on port 0, synthesizes the suite with the `ess`
   on `PATH` (`ess verify conform synthesize --path ess --scenarios ess`) into a temporary
   directory, runs the CLI against the server, and asserts exit 0 with
   `a2a.tasks.GetTask/outcome/not-found` and `a2a.tasks.CancelTask/outcome/not-found` `passed`.
   It asserts each helper returns no `Mismatch` against the same server.
5. A test starts a planted-defect endpoint (a2a-server behind a proxy in the test that rewrites
   the answer to `GetTask` of an unknown id into 200 with an empty task) and asserts the CLI
   exits non-zero with `a2a.tasks.GetTask/outcome/not-found` listed `failed`, and
   `expect_error_reason(TASK_NOT_FOUND)` returns a `Mismatch`.

## Scope

- `crates/a2a-testkit/` (new crate: `src/`, `src/main.rs`, `tests/`)
- `Cargo.lock`
- `CHANGELOG.md`, `README.md`, `AGENTS.md` (Layout row)

Reads `crates/a2a-conformance` (the `Target` trait and interpreter) and `crates/a2a-client`
without changing them; both are merged before this story starts.
