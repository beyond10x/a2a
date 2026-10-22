---
format: aep.planning-md/3
id: review-result:adversary-streaming-binding-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:streaming-binding
relations:
- reviews: story:streaming-binding
revision: 1
---
```
unit: story:streaming-binding (adversary) — worktree a2a-w1-streaming-binding, uncommitted on 7c6b8d3
verdict: NEEDS-CHANGE
cases: executed 153→153, red 0
origin: introduced 3 / pre-existing 1 / undecided 0
wrote-outside-worktree: 1 directory tree, ~/.cache/a2a-gen/wave1/adv-streaming-binding/ (52M)
needs-coordinator: how to fix F1 (copy the answer, or record an ESS gap); where to route F3 (pre-existing)
```

**1. Diff stat.** This is the unit's own diff. I wrote nothing in the worktree.
```
 118 files changed, 1218 insertions(+), 244 deletions(-)
 ess/components.yaml    |  11 ++++
 ess/domains/tasks.yaml | 164 ++++++++++++++++++++++++++++++++++++++++++++++++-
 ess/ess-inputs.yaml    |   2 +
```
There are also two untracked files, `ess/scenarios/stream-{message-only,starts-with-task}.yaml`, both the implementor's. The worktree lease is released (`session-end` exit 0).

**2. Cases.** I added none. The one property worth a case is "the Task the stream begins with is the task that was created", and ESS 0.67.0 refuses every way of writing it, so it is F1 below. The case is kept in scratch only:
```
refusal[ESS-AUTHOR-013]: `a2a.tasks/authored/adv-stream-task-is-created-task` in .../cases/adv-stream-task-is-created-task.yaml
  the payload of `a2a.tasks.MessageStreamResponse` declares no `task.id`
```
A nested capture gives ESS-AUTHOR-013/-019, and `$observed` inside the payload gives ESS-AUTHOR-042/-015.

**3. Suite run.** Worktree spec, outputs in scratch: `ess verify conform synthesize --path ess --scenarios ess` then `run --target interpreted --report-format 2`, exit 3.
```
153 scenario(s) (13 authored), 0 refusal(s)
153 scenarios: 137 passed, 0 failed, 16 error, 0 unsupported
```
- All 11 resolve-unmapped authored scenarios pass.
- The 16 errors are the 12 agent moves (`StartWorking/outcome/moved` and the like) plus 4 `grant/denied`.

**Planted defects, run before any check is called green.** "Reverse" means the unchanged suite run against the planted spec's interpreter. To do that I rebound `provenance.spec_digest` in a scratch copy of the suite.

| Plant (tasks.yaml) | Reverse run | Caught by |
|---|---|---|
| :705 SUBMITTED→WORKING | 1 failed | `task-created`: `observed: task.status.state = "TASK_STATE_WORKING"` |
| :703 context_id→generated | 1 failed | `task-created`: `observed: task.context_id = "generated-3"` |
| :701-710 `task: {generated: true}` | 1 failed | `task.context_id was not carried; task.status.state was not carried` |
| :696 drop the stream from task-created | refused at validate | ESS-AUTHOR-037 in stream-starts-with-task |
| :680 direct-message `message` generated | 1 failed | `event field message differs from actual response field reply` |
| :676-682 drop the direct-message stream | 2 failed | direct-message and stream-message-only |
| :668 continued `task` generated | 1 failed | `event field task differs from actual response field task` |
| :669 continued `message` generated | 1 failed | continued |
| :665-671 drop the continued stream | 1 failed | continued |
| direct-message / continued emit TaskSubscriptionResponse instead | 2 / 1 failed | direct-message (+authored) / continued |
| spec :1413 statusUpdate→status_update | `ess generate --check` exit 1 | projections-drift |

When the suite is instead resynthesized from the planted spec, every payload plant passes 137/0/16. Only the reverse direction catches them.

**4. Findings** (they cover the worktree on 7c6b8d3).

| file:line | verdict | origin | measured | what reaches it |
|---|---|---|---|---|
| ess/domains/tasks.yaml:702 | NEEDS-CHANGE | introduced | In task-created, the stream's `task.id: {generated: true}` is tied neither to the created row nor to the answer. A probe on the reference interpreter observed `TaskCreated.task_id = "generated-1"` but stream `task.id = "generated-2"`. AsyncAPI :92 also lacks the "carries the new instance's identity" clause that :138 has. No UNMAPPED line records this. | Every streamed message that creates a task (3.1.2: "the stream MUST begin with the Task object"). Two fixes: `task: {response: task}` validates with 0 refusals, as `continued` does, but drops the SUBMITTED and context pins. Or record an ESS gap: `{subject:}` gives ESS-COMMAND-004, `{payload:}` gives ESS-COMMAND-002, and nested `{response: task.id}` is refused. |
| ess/domains/tasks.yaml:705 | CONFIRMED | introduced | Pinning the first element to TASK_STATE_SUBMITTED has no upstream source. 3.1.2 states no state, and the 6.2 example (specification.md:1376) streams `TASK_STATE_WORKING` first. | A server that behaves like upstream's own example fails `SendStreamingMessage/outcome/task-created` (the planted WORKING run above). |
| ess/domains/tasks.yaml:167 | CONFIRMED | pre-existing | TaskSnapshot `context_id` is required, but a2a.proto:173 has `string context_id = 2;` with no REQUIRED. Generated AsyncAPI `type.a2a.tasks.TaskSnapshot` has `required: [id, contextId, status]`. The base OpenAPI already has the same list. | Both stream channels now reject a Task element that has no contextId. |
| generated/asyncapi/asyncapi/a2a-agent.yaml:92 | INFEASIBLE | introduced | The `send.a2a.tasks.MessageStreamResponse` description names only `task-created`. `continued` (tasks.yaml:665) and `direct-message` (:676) also emit it. | Readers of the projection. The text comes from the ESS generator, which lists only outcomes that create, change or move a row, so this unit cannot fix it. |

**5. Attacked and could not break.**
- Names and wire names of StreamResponse (task/message/statusUpdate/artifactUpdate), TaskStatusUpdateEvent and TaskArtifactUpdateEvent match the proto, including the required lists. My jq shape check found these mismatches on a planted copy and none on the unit's file.
- Both channels exist (`message:stream`, `tasks:subscribe`), with `additionalProperties: false` and no row or instance property. The only extra keys are the `x-ess-name/kind/field` annotations that every schema in the projection carries.
- The answer-to-event copies in `continued` and `direct-message` are correct (`reply` goes out on the `message` wire field).
- The quotes in the comments from 3.1.2, 3.1.6 and 11.7 match upstream.

**6. Paths outside the worktree.** All under `~/.cache/a2a-gen/wave1/adv-streaming-binding/`:
- Copies of the spec: `cur/`, `base/`, `probe1/`.
- My refused case and its output: `cases/`, `caseA-suite.json`.
- The task-id probe: `probe-ids/`, `probe-ids.jq`.
- The AsyncAPI shape check: `asyncapi-plant/`, `asyncapi-shape.jq`.
- One directory per plant: `plant-p1-ctx-generated`, `plant-p1-state-working`, `plant-p1-task-absent`, `plant-p1-tc-no-stream`, `plant-p2-dm-message-generated`, `plant-p2-dm-no-stream`, `plant-p3-cont-message-generated`, `plant-p3-cont-no-stream`, `plant-p3-cont-task-generated`, `plant-p4-cont-swap-channel`, `plant-p4-dm-swap-channel`, `plant-probe-tc-response-task`, `plant-drift`.
- Scripts: `plant-full.sh`, `plant-old-suite.sh`, `cross-run.sh`, `reverse-run.sh`.
- The suite run: `final-suite.json`, `final-synth.txt`, `final-run.txt`, `final-report.json`.

```findings
[
  {
    "file": "ess/domains/tasks.yaml",
    "line": 702,
    "category": "property",
    "severity": "warning",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "the Task a task-created stream begins with gets an id independent of the created task (interpreter observed generated-1 vs generated-2), with no ESS-gap record; `task: {response: task}` validates as an alternative"
  },
  {
    "file": "ess/domains/tasks.yaml",
    "line": 705,
    "category": "judgement",
    "severity": "note",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "pinning the first stream element to TASK_STATE_SUBMITTED has no upstream source and fails a server that streams WORKING first as specification.md 6.2 does"
  },
  {
    "file": "ess/domains/tasks.yaml",
    "line": 167,
    "category": "contract-drift",
    "severity": "note",
    "verdict": "CONFIRMED",
    "origin": "pre-existing",
    "message": "TaskSnapshot requires contextId while a2a.proto Task.context_id is not REQUIRED, and both stream channels now carry that schema"
  },
  {
    "file": "generated/asyncapi/asyncapi/a2a-agent.yaml",
    "line": 92,
    "category": "contract-drift",
    "severity": "note",
    "verdict": "INFEASIBLE",
    "origin": "introduced",
    "message": "the MessageStreamResponse send operation names only task-created as emitter, omitting continued and direct-message; the ESS generator writes this text"
  }
]
```
