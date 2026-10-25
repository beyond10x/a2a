---
format: aep.planning-md/3
id: approval-record:ess-0-69-2-pin
kind: approval-record
status: draft
title: ESS pin 0.69.2 and report row 29 closed by ESS 0.69.0
summary: Pin moved to ess 0.69.2; the push-config create tenant guard synthesizes natively, so story:server drops its UNMAPPED marker on resume.
relations:
- decides: story:server
revision: 1
---
## Decision

b10x-chef-orchestrator, 2026-10-25, under its class-C grant (dependency upgrade including the ess
pin) and the workspace rule "ALWAYS upgrade to most recent ESS": "move a2a's ESS pin to the newest
release, 0.69.2 ... Then drop the ESS workaround the a2a store records for ESS report row 29 ...
remove the workaround with its test proving the native behaviour, and record the closure in the
store. Leave rows that wait for ess 0.70.0 alone."

## Effect

- The pin moves to ess 0.69.2: `ess/ess-inputs.yaml`, the `check` and `pages` workflows (asset
  checksum from the release's `SHA256SUMS`), and `generated/rust/a2a` resynthesized. Its public
  types are unchanged; `task check` is green.
- Row 29 (CreateTaskPushNotificationConfig: a `when_related:` guard comparing the parent task's
  stored `tenant` with the `Optional` input `tenant` was refused as ESS-SYNTH-003) is fixed in ESS
  0.69.0. Its CHANGELOG: "`ess verify synthesize` witnesses a `when_related:` refusal whose guard
  tests an `Optional` input's presence ... It refused it as ESS-SYNTH-003".
- Verified on 0.69.2 with the row-29 reproduction (the server unit's specification with the guard
  in place): no ESS-SYNTH-003, the scenario
  `a2a.tasks.CreateTaskPushNotificationConfig/outcome/other-tenant` is synthesized, and the
  interpreted run passes 171 of 171. DeleteTaskPushNotificationConfig `deleted` is synthesized too;
  GetTaskPushNotificationConfig `found` is still refused (ESS-SYNTH-001, row 30, not closed here).
- The workaround is not on `main`. It is the `UNMAPPED (ESS gap)` marker standing in for the
  `other-tenant` guard of CreateTaskPushNotificationConfig in the server unit's uncommitted
  specification, parked with wave 3. `main` carries neither the marker nor the guard, so nothing is
  removed here, and adding the guard on `main` would add an outcome to the generated crate's
  public types, which this item keeps unchanged.
- On resume, story:server replaces that marker with the guard
  (`when_related: {via: input.task_id, predicate: …}`, error `a2a.tasks.TaskNotFound`) and keeps
  the synthesized `other-tenant` scenario as the test of the native behaviour.
