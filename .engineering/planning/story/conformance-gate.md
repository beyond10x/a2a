---
format: aep.planning-md/3
id: story:conformance-gate
kind: story
status: active
title: Run the synthesized conformance suite against a2a-server in task check
summary: crates/a2a-conformance interprets the ESS-synthesized suite against a2a-server's own obligation implementation; task check fails on any failed, errored or unlisted unsupported scenario, and a planted defect turns it red.
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:server
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
  path: Taskfile.yml
- confidence: cited
  path: crates/a2a-conformance
- confidence: cited
  path: crates/a2a-server/src/lib.rs
- confidence: cited
  path: crates/a2a-server/src/ports.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T11:21:59Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-29T11:21:59Z", actor: "human:timo", revision: 7}
---
## Outcome

`task check` holds `a2a-server`'s implementation to the conformance suite ESS synthesizes from
`ess/`, not only the ESS interpreted reference target, and a defect in the server's obligations
turns the gate red.

## Acceptance

1. `crates/a2a-conformance` is a library with a `Target` trait (execute a command as an actor,
   query a view, force an external outcome, read the events published since the last step), each
   method returning `Result<_, TargetError>` where `TargetError::Unsupported(reason)` makes the
   scenario `unsupported` with that reason. Its interpreter runs every step kind of an
   `ess-conformance/52` suite against a `Target` and writes an `ess-conformance-results/1`
   document with one status per scenario. A step kind the interpreter does not know is `error`
   for that scenario, never a skip.
2. Its target for `a2a-server` runs each scenario on a fresh instance of the server's own
   obligation implementation (the ports `a2a-server`'s binary serves, through the generated
   `System`), with events read from `System::published`. It does not use the generated memory
   ports or the ESS interpreted target. `a2a-server` exposes what the target needs without a new
   model type.
3. `task conform` synthesizes the suite with the pinned `ess` (`ess verify conform synthesize
   --path ess --scenarios ess`) into `$HOME/.cache/a2a-gen/conform/suite.json`, after removing
   that directory so every file in it is fresh; runs the interpreter against the `a2a-server`
   target; and turns the results into `report.json` there with `ess verify conform report`. It
   fails unless the report has 0 failed and 0 error, and every `unsupported` scenario is named,
   with its reason, in `crates/a2a-conformance/unsupported.tsv`; it also fails on a name there
   that is not `unsupported` in the run. `task check` runs `task conform`.
4. A test target `planted_defect` runs the suite (synthesized by the test with the `ess` on
   `PATH` into a temporary directory) against the `a2a-server` target with one obligation broken:
   CancelTask answers `canceled` for a task already in a terminal state. It asserts that
   `a2a.tasks.Task/state/{Completed,Failed,Canceled,Rejected}/refuses/a2a.tasks.CancelTask`
   come back `failed` by name, and that `a2a.tasks.CancelTask/outcome/canceled` still passes.
5. The interpreter's step handling is tested on its own with a stub `Target`: for each step
   kind, one scenario that passes and one that fails, and one scenario whose target answers
   `Unsupported` and comes back `unsupported` with the reason.

## Scope

- `crates/a2a-conformance/` (new crate: `src/`, `tests/`, `unsupported.tsv`)
- `crates/a2a-server/src/lib.rs`, `crates/a2a-server/src/ports.rs` (a public entry for the target)
- `Taskfile.yml` (`conform`, `check`)
- `Cargo.lock`
- `CHANGELOG.md`, `README.md`, `AGENTS.md` (Layout and Commands rows)

Shares `Cargo.lock`, `CHANGELOG.md`, `README.md` and `AGENTS.md` with story:client, which runs
at the same time; the wave coordinator resolves those merges, no ordering edge.
