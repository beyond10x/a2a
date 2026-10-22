---
format: aep.planning-md/3
id: review-result:adversary-resolve-unmapped-pass-2
kind: review-result
status: active
title: Adversary pass 2 on story:resolve-unmapped
relations:
- reviews: story:resolve-unmapped
revision: 1
---
unit: story:resolve-unmapped (adversary re-pass)
verdict: red
cases: executed 16, red 4
origin: n/a
wrote-outside-worktree: ~/.cache/a2a-gen/wave1/adv2-resolve-unmapped/
needs-coordinator: yes

The rework fixed what the brief asked about. It is red because of one older problem: two views of push configurations that any caller, signed in or not, can read. I changed nothing in the worktree and kept no new scenario in `ess/`, because both new ones pass against the reworked spec.

Baseline: validate passes; synthesis gives 143 scenarios with 0 refusals; the run gives 127 passed, 0 failed and 16 errors, all in the known `/moved` and `grant/denied` classes. To plant a defect in the implementation side, I ran the unplanted suite against a changed copy of the spec, with only the suite's digest swapped (`crossrun.sh`).

| # | Check | Planted defect | Planted result | Real spec |
|---|---|---|---|---|
| 1 | Delete for an unknown task answers TaskNotFound | `task-not-found` answers success | caught by a generated scenario | passes |
| 1 | Delete never removes another task's config | drop the `task_id` condition | caught by `delete-config-of-other-task` and a generated scenario | passes |
| 1 | Delete removes only the named config | drop the `config_id` condition | caught by a generated scenario | passes |
| 1 | Deleting a config of a canceled task succeeds | — | scratch scenario `adv2-delete-config-of-completed-task` | passes |
| 2 | Create under another task does not overwrite | spec as it was at my first pass | `adv-config-id-of-other-task` errors (ESS-CF-TARGET) | passes |
| 2 | Get checks `config_id` | drop that condition | caught | passes |
| 2 | Get checks `task_id` | drop that condition | **not caught by any of the 143**; caught by scratch scenario `adv2-get-config-through-other-task` | passes |
| 2 | Per-task config view filters by task | filter lets every row through | caught | passes |
| 3 | 7 quotes match proto and spec word for word | one word changed in each | 6 of 7 not found (the 7th had no word to swap) | all 7 found |
| 4 | Client is refused AllTasks | grant AllTasks to `a2a.tasks.Client` | **not caught**: no scenario reads it as that actor | passes |
| 5 | Schema required lists | make the fields required again | `["configs"]` and `[…,"append"]` appear | `null` and `[task_id, context_id, artifact]` |

The rework's claim that removing AllTasks makes synthesis refuse 35 outcomes is correct: I removed it and got 35 refusals.

Findings that need your call:
- **Open views (blocker):** `PushNotificationConfigs` holds every task's url, token and authentication. It and `TaskPushNotificationConfigs` are served with no access check at all. Section 13.1 says results MUST be scoped to the caller. A copy that grants `Tasks` and `TaskPushNotificationConfigs` to Client and `PushNotificationConfigs` to Agent gives 149 scenarios, 0 refusals, 0 failed.
- **AllTasks scoping doesn't hold:** the `Tasks` view is also open. Called with no filters it returns the same rows AllTasks withholds; section 13.1 line 3081 covers exactly that case. The same grant fixes it.
- **Delete event on a no-op:** `deleted` emits PushNotificationConfigDeleted on every delete, including a second delete and an id that never existed. At HEAD a second delete emitted nothing. A separate no-op branch is refused by ESS (4 SYNTH-001 refusals), so it needs an `# UNMAPPED:` marker.
- **Missing scenario:** `adv2-get-config-through-other-task` is the only thing that catches a Get that ignores the task. I suggest the unit adopts it.

```findings
[
  {
    "file": "ess/domains/tasks.yaml",
    "line": 1374,
    "category": "spec-gap",
    "severity": "blocker",
    "verdict": "CONFIRMED",
    "origin": "pre-existing",
    "message": "PushNotificationConfigs (every task's url, token, authentication) and TaskPushNotificationConfigs are open views, served to any caller including anonymous (generated/openapi a2a-agent.yaml:1398,1449, no x-ess-may-read, no 403); section 13.1 requires scoping. Grant fix verified at 149 scenarios, 0 refusals, 0 failed."
  },
  {
    "file": "ess/domains/tasks.yaml",
    "line": 1279,
    "category": "spec-gap",
    "severity": "warning",
    "verdict": "CONFIRMED",
    "origin": "pre-existing",
    "message": "Tasks is open and, with no parameters, serves every task row that AllTasks's Agent-only grant withholds, so the 13.1 scoping claimed at 1319-1321 does not hold. Section 13.1 line 3081 covers the case with no filter. Grant Tasks to Client; the per-row gap remains."
  },
  {
    "file": "ess/domains/tasks.yaml",
    "line": 1324,
    "category": "ess-limit",
    "severity": "warning",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "AllTasks/grant/read/denied reads as a2a.discovery.Client and anonymous, never as a2a.tasks.Client; a target admitting a2a.tasks.Client passes all 143 (cross-run, 0 failed). Report to ESS: the denial cell picks the lowest-named actor across domains."
  },
  {
    "file": "ess/domains/tasks.yaml",
    "line": 946,
    "category": "ess-limit",
    "severity": "warning",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "deleted emits PushNotificationConfigDeleted on every delete under an existing task, including a second delete and an unknown id; HEAD's already-gone emitted none. A no-op branch is refused (ESS-SYNTH-001, 4 refusals), so add an UNMAPPED (ESS gap) marker; the comment at 936-941 omits the event."
  },
  {
    "file": "ess/domains/tasks.yaml",
    "line": 865,
    "category": "missing-scenario",
    "severity": "warning",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "Dropping task_id from Get's config-not-found guard survives the full suite; scratch scenario adv2-get-config-through-other-task kills it and passes on the real spec (not added to ess/)."
  },
  {
    "file": "ess/domains/tasks.yaml",
    "line": 1240,
    "category": "spec-gap",
    "severity": "note",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "TaskArtifactUpdated now mixes wire names; lastChunk is camelCase while task_id and context_id stay snake_case, where ProtoJSON gives taskId and contextId."
  },
  {
    "file": "ess/domains/tasks.yaml",
    "line": 868,
    "category": "ess-limit",
    "severity": "note",
    "verdict": "CONFIRMED",
    "origin": "pre-existing",
    "message": "Get found's response body is not bound to the matched row and carries no UNMAPPED marker, unlike List at 906."
  },
  {
    "file": "ess/domains/tasks.yaml",
    "line": 552,
    "category": "spec-gap",
    "severity": "note",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "The 7.6.1 quote drops its condition \"If the credential is received out-of-band\"."
  },
  {
    "file": "ess/domains/tasks.yaml",
    "line": 317,
    "category": "spec-gap",
    "severity": "note",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "The comment \"it never leaves the specification\" is false; row is served at /tasks/views/allPushNotificationConfigs and in the published PushNotificationConfigCreated. Accepting row is unaffected."
  }
]
```

Scripts and run outputs are in ~/.cache/a2a-gen/wave1/adv2-resolve-unmapped/; the two new scenarios are in its `scen/` folder and the grant fix copy is in `fix-view-grants/`.
