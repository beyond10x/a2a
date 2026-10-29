---
format: aep.planning-md/3
id: story:testkit-wire-events
kind: story
status: active
title: a2a-testkit observes task events over the wire
summary: The wire target derives task event claims from the answers and streams that carry them, so more than 7 of 177 scenarios run against an endpoint.
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:testkit
- depends_on: story:view-tenant-scope
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T14:49:09Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-29T14:49:09Z", actor: "human:timo", revision: 6}
---
## Outcome

`a2a-testkit run` observes more of the suite over the wire. Measured on `wave/5-int` after
story:view-tenant-scope (`a2a-testkit run` against `a2a-server --listen 127.0.0.1:0`, suite from
`ess verify conform synthesize` with ESS 0.75.0): 183 scenarios, 7 passed, 0 failed, 0 error, 176
`unsupported`; 118 of them stop at "a2a.tasks.SendMessage took `task-created`, which publishes
events the A2A HTTP+JSON binding does not carry", and every synthesized scenario then reads the
`AllTasks` and `Tasks` views, which the target answers `unsupported`.

The wire target derives the task events a scenario expects from the answers and streams that
carry them, and reads the two task views for the tasks the open scenario created, so the
scenarios whose setup sends a message and whose check is a client operation on that task, or a
stream of it, run and pass.

## Acceptance

1. `TaskCreated`: after a `SendMessage` or `SendStreamingMessage` whose answer holds a task the
   scenario had not seen, `published()` reports `a2a.tasks.TaskCreated` with the task's id,
   context id and tenant as the answer and the request carry them. An answer that holds no task
   (a direct message) derives none. Held by a stub-endpoint test in `crates/a2a-testkit/tests/`.
2. `TaskStatusUpdated` and `TaskArtifactUpdated`: the status and artifact frames of a
   `SendStreamingMessage` or `SubscribeToTask` stream are reported as those events, each field
   read as the generated contract names it. Held by a stub-endpoint test.
3. Every event or field the wire does not carry, and every outcome whose events the binding has
   no correspondence for, stays `TargetError::Unsupported` with its reason; no event is reported
   that the answer does not carry. Held by a stub-endpoint test that answers a task-free success
   and gets no `passed`.
4. Views: `a2a.tasks.AllTasks` is answered with the tasks the open scenario created, each read
   back with GetTask under the tenant it was created under; `a2a.tasks.Tasks` with ListTasks under
   the parameter's tenant, keeping only the tasks the open scenario created. Every other view, and
   a read that cannot be so scoped, stays `unsupported` with its reason.
5. Gap A: a push notification configuration command on a task the scenario created under another
   tenant is `unsupported` with the reason `a2a-conformance`'s in-process target gives, never
   `failed` against a2a-server's correct `TaskNotFound`.
6. `crates/a2a-testkit/tests/endpoint.rs`, against a2a-server on a loopback port: 0 `failed`,
   0 `error`, and `passed` for `a2a.tasks.SendMessage/outcome/task-created`,
   `a2a.tasks.SendStreamingMessage/outcome/task-created`, `a2a.tasks.GetTask/outcome/found`,
   `a2a.tasks.CancelTask/outcome/canceled` and `a2a.tasks.SubscribeToTask/outcome/subscribed`,
   besides the 7 that pass today. A scenario leaves this list only by a new revision of this
   story that names the reason the run gave.
7. The planted-defect tests (`tests/planted_defect.rs`, `tests/adv_mutants.rs`) still go red, and
   an endpoint that answers a wrong task id, state or context in the answer that carries the event
   fails the scenario that reads it (new mutant cases).
8. `AGENTS.md`'s `crates/a2a-testkit` row and `README.md` give the counts the endpoint run printed.

## Context

This amends story:testkit acceptance 1, under which a step's published events were always
`unsupported`. adversary-testkit-pass-1 F1 and F2 (a stub endpoint must not earn a2a-server's
verdict) still hold: an event is derived only from what the endpoint itself answered, never from
the suite's expectation, which items 3 and 7 test.

## Scope

`crates/a2a-testkit/src/{target.rs,contract.rs,expected.rs}`, `crates/a2a-testkit/tests/`,
possibly `crates/a2a-conformance/src/server.rs` (making the gap A reason shared),
`AGENTS.md`, `README.md`, `CHANGELOG.md`. It runs the suite story:view-tenant-scope changes (183
scenarios), which is its only dependency on it; it does not touch `ess/`, `generated/` or
`crates/a2a-server`.
