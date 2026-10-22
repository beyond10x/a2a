---
format: aep.planning-md/3
id: review-result:adversary-jsonrpc-binding-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:jsonrpc-binding
relations:
- reviews: story:jsonrpc-binding
revision: 1
---
unit: story:jsonrpc-binding (adversary). Worktree a2a-w1-jsonrpc-binding: the unit's uncommitted changes on wave/1 d99101d
verdict: red
cases: executed 164→167, red 3
origin: introduced 4 / pre-existing 1 / undecided 0 (all 3 red cases belong to the 1 pre-existing finding)
wrote-outside-worktree: 1 scratch root, `~/.cache/a2a-gen/wave1/adv-jsonrpc-binding/`
needs-coordinator: F1 needs a call on which answer wins. My cases assume the request-alone refusal beats the task lookup, because that is the unit's own rule on Create and Get.

**1. Diff stat**
- `git --no-pager diff --stat -- ess` shows 4 files, 178+/30-. components.yaml, discovery.yaml and tasks.yaml are the unit's changes; I did not touch them.
- My test files:
  - new: `ess/scenarios/adv-push-capability-before-task-lookup.yaml` (32 lines), `adv-version-before-task-lookup.yaml` (59), `adv-streaming-capability-before-task-lookup.yaml` (20)
  - `ess/ess-inputs.yaml`: +3 lines after line 32, one per adv case. The unit's 14 lines are unchanged.
- No non-test path was changed by me.

**2. Cases added.** Each was run alone (`ess verify conform author` with only that case listed). All exit 1 on the current tree, and all exit 1 with the same observations on base d99101d.

| case | act (task `no-such-task`) | expected | observed |
|---|---|---|---|
| adv-push-capability-… | List, Delete | push-not-supported / PushNotificationNotSupported | task-not-found / TaskNotFound |
| adv-version-… | GetTask, CancelTask, SubscribeToTask | version-not-supported / VersionNotSupported | not-found / TaskNotFound |
| adv-version-… | List, Delete | version-not-supported / VersionNotSupported | task-not-found / TaskNotFound |
| adv-streaming-capability-… | SubscribeToTask | streaming-not-supported / UnsupportedOperation | not-found / TaskNotFound |

Verbatim extract (the ESS-CF-OUTCOME and ESS-CF-ERROR checks fail on every act):
```
input: a2a.tasks.ListTaskPushNotificationConfigs(task_id = "no-such-task")
expected:
- error = a2a.tasks.PushNotificationNotSupported
observed:
- error = a2a.tasks.TaskNotFound
```

**3. Suite run** (the Taskfile `conform-interpreted` commands, on a scratch copy)
- with my cases: `167 scenarios: 148 passed, 3 failed, 16 error, 0 unsupported`, exit 1. The 3 failed are the adv cases.
- unit alone: `164 scenarios: 148 passed, 0 failed, 16 error`, exit 3. This matches the implementor's count.
- base d99101d: `153 scenarios: 137 passed, 0 failed, 16 error`. The 16 errors are the same set as after the unit (diff empty).
- All 13 earlier authored scenarios and all 11 jsonrpc scenarios pass. No regression.

**4. Findings**
- **F1** `ess/domains/tasks.yaml:501`, `:512`, `:521`. CONFIRMED, pre-existing, warning.
  - What happens: the group refusals (-32009, -32008, -32003, -32004) answer after the task lookup on GetTask, Cancel, Subscribe, List and Delete. Create and Get answer them before it (`before_row: true`, rule at `:929`).
  - Result: with the same agent card, an unknown task gets -32003 from Create but -32001 from List.
  - Upstream does not say which comes first when both conditions hold. Sections 3.3.4 and 3.6.2 say MUST return without exception.
  - Fix: declare those refusals per command with `before_row: true`, as Create and Get do (a group cannot carry it). Or record it as an ESS gap.
- **F2** jsonrpc scenarios. Every error a scenario claims is pinned: 25 of 34 planted swaps went red. The 9 that survived are branches no scenario claims. NEEDS-CHANGE, introduced, warning, because the acceptance criterion ("held by a conformance scenario per method") is ambiguous on this.
  - SendMessage context-mismatch `:597` and context-not-accepted `:600`
  - SendStreamingMessage `:686`, `:689`, `:692`, `:700` (its -32001, -32004 terminal, -32005 and -32602 are held by nothing)
  - CancelTask not-cancelable-now `:816`
  - GetTaskPushNotificationConfig `:1011`, `:1015`, `:1021`
- **F3** `tasks.yaml:931-942`. NEEDS-CHANGE, introduced, note.
  - Planted defect: I removed Create's three `before_row` lines. All 24 authored scenarios still passed against the planted copy, so the unit's precedence change is not pinned.
  - A probe does catch it. It passes on the current tree and goes red on the plant (observed task-not-found / TaskNotFound): `~/.cache/a2a-gen/wave1/adv-jsonrpc-binding/probes/probe-create-get-before-task-lookup.yaml`.
- **F4** `tasks.yaml:461`. NEEDS-CHANGE, introduced, note.
  - The name and both quotes are correct: specification.md:560 and :1187 are the only places upstream mentions it, and the proto has none.
  - The comment says "it is what a client reports about an agent's answer". No upstream text says this, and the INTERNAL/500 mapping suggests a server may answer it.
- **F5** `ess/scenarios/jsonrpc-send-message.yaml:53`. NEEDS-CHANGE, introduced, note.
  - The task-terminal act sends `task-done`, which no step created. It passes only because the outcome is external. A truthful target would answer -32001.
  - Fix: create a task, cancel it, then send the message to it.

**5. Attacked and not broken**
- Attack 1: all 11 proto RPCs have a command or view (ListTasks is a view) and a scenario. All codes in the summaries match the 5.4 table row by row. Every error listed in 3.1 is answered by its command.
- ESS-AUTHOR-041 fix: Create and Get answer the three refusals ahead of the task lookup and ahead of `created` (probe passes). Plant shown in F3.
- Error swaps: 25 of 34 plants went red, each on the scenario that claims that error, including group swaps through send-message, send-streaming, List and Delete.

**6. Paths written outside the worktree** (all under `~/.cache/a2a-gen/wave1/adv-jsonrpc-binding/`, mode 700): `scripts/{alone,probe,plant,swaps,suite}.sh`, `probes/`, `plants/` (35 sed files plus lists), `plant/` (35 dirs), `alone/` (7 dirs), `suite/{after,unit,base}`, `ref/`, `base/ess`, `ess-inputs.unit.yaml`, `alone-*.txt`. My worktree lease is released.

```findings
- {file: ess/domains/tasks.yaml, line: 501, category: correctness, severity: warning, verdict: CONFIRMED, origin: pre-existing, message: "Group refusals -32009/-32008/-32003/-32004 answer after the task lookup on GetTask, CancelTask, SubscribeToTask, List and Delete, while Create and Get answer them before_row; red: adv-push-capability-before-task-lookup, adv-version-before-task-lookup, adv-streaming-capability-before-task-lookup (also red on d99101d)"}
- {file: ess/domains/tasks.yaml, line: 686, category: coverage, severity: warning, verdict: NEEDS-CHANGE, origin: introduced, message: "9 error branches no scenario claims; swapping each error left the suite green: SendMessage :597 :600, SendStreamingMessage :686 :689 :692 :700, CancelTask :816, GetTaskPushNotificationConfig :1011 :1015 :1021"}
- {file: ess/domains/tasks.yaml, line: 931, category: coverage, severity: note, verdict: NEEDS-CHANGE, origin: introduced, message: "Removing Create's three before_row lines leaves all 24 authored scenarios green; probe-create-get-before-task-lookup (scratch) pins it"}
- {file: ess/domains/tasks.yaml, line: 461, category: sourcing, severity: note, verdict: NEEDS-CHANGE, origin: introduced, message: "InvalidAgentResponse rationale 'what a client reports' has no upstream source; 5.4 maps it to INTERNAL/500"}
- {file: ess/scenarios/jsonrpc-send-message.yaml, line: 53, category: test-validity, severity: note, verdict: NEEDS-CHANGE, origin: introduced, message: "task-terminal act targets task-done, which no step created; a truthful target answers TaskNotFound"}
```
