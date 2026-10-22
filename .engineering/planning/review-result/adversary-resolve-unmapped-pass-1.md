---
format: aep.planning-md/3
id: review-result:adversary-resolve-unmapped-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:resolve-unmapped
relations:
- reviews: story:resolve-unmapped
revision: 1
---
unit: story:resolve-unmapped (adversary)
verdict: red
cases: executed 140, red 17
origin: n/a
wrote-outside-worktree: ~/.cache/a2a-gen/wave1/adv-resolve-unmapped/
needs-coordinator: yes

Suite: 140 scenarios: 123 passed, 1 failed, 16 error (ess verify conform run, interpreted).
Mutation (copy without status_message): 87 mutants, 56 killed, 0 survived, 1 inconclusive, 17 stillborn, 13 unwitnessed.
Planted P1-P3 killed; P5 (StartWorking publishes COMPLETED) survived the unit suite, killed by adv-agent-moves-carry-status; P4c untestable (system-decided input).
Process deviation (adversary's own): full suite ran before the first case was written.

Added: adv-delete-config-of-unknown-task (red, ESS-CF-OUTCOME), adv-config-id-of-other-task (red, ESS-CF-TARGET), adv-agent-moves-carry-status (green; kills P5-P9).

```findings
- file: ess/domains/tasks.yaml
  line: 866
  category: spec-gap
  severity: blocker
  verdict: CONFIRMED
  origin: pre-existing
  message: Delete with an unknown task_id answers already-gone (success) where 3.1.10 requires TaskNotFoundError; remodel probes are refused with ESS-SYNTH-001.
- file: ess/domains/tasks.yaml
  line: 749
  category: spec-gap
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Config id is scoped per agent although the proto nests it under its parent task; a create of the same id under another task has no defined answer.
- file: ess/domains/tasks.yaml
  line: 433
  category: spec-gap
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The blocking SendMessage choice follows 3.1.1 against the proto, which 1.4 makes normative, and misattributes the proto quote.
- file: ess/domains/tasks.yaml
  line: 831
  category: spec-gap
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: ListTaskPushNotificationConfigs response requires configs and nextPageToken; proto marks neither REQUIRED, so a valid {} fails the schema.
- file: ess/domains/tasks.yaml
  line: 1169
  category: spec-gap
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: TaskArtifactUpdated append/last_chunk are required snake_case Booleans; proto bool is omitted when false and written lastChunk.
- file: ess/domains/tasks.yaml
  line: 914
  category: missing-scenario
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: Agent-move payload status.state was unchecked (P5 survived); adv-agent-moves-carry-status now pins it.
- file: ess/domains/tasks.yaml
  line: 1135
  category: ess-limit
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: UpdateArtifact append/last_chunk payload mapping cannot be checked; the interpreted target refuses to compare system-decided input.
- file: ess/domains/tasks.yaml
  line: 525
  category: spec-gap
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Stream-end choice does not cite 7.6.1, which supports it.
- file: ess/domains/tasks.yaml
  line: 368
  category: spec-gap
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: VersionNotSupported and ExtensionSupportRequired summaries omit gRPC FAILED_PRECONDITION from the 5.4 table.
- file: ess/domains/tasks.yaml
  line: 1211
  category: spec-gap
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Tasks view status parameter uses lifecycle names, not the TASK_STATE_* wire enum.
- file: ess/domains/tasks.yaml
  line: 1240
  category: spec-gap
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: AllTasks view has no A2A source and is unscoped; 13.1 requires results scoped to the caller.
```
