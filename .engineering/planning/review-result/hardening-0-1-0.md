---
format: aep.planning-md/3
id: review-result:hardening-0-1-0
kind: review-result
status: active
title: ess:hardening of ess/ before 0.1.0
relations:
- reviews: epic:a2a-delivery
revision: 1
---
## Scope

`ess:hardening` of `ess/` at 917e417 (ESS 0.67.0) before the first release. No implementation exists
yet, so techniques 2 (random sequences), 3 (caller replay), 4 (determinism) and 5 (metamorphic
relations) were not run. Technique 7 (spec diff) was not run: there is no earlier release to diff
against.

## Technique 8: design review

- Planted defect: GetTask's `not-found` outcome deleted in a copy of `ess/`; the copy still validated.
- Result on the planted copy: 22 findings against A2A v1.0.1 `docs/specification.md` and `a2a.proto`;
  finding 1 is the planted defect ("GetTask has no unknown_instance → TaskNotFound outcome").
- The real spec has the outcome, so finding 1 is dropped. Findings 21 and 22 are modelling devices
  the spec already declares (agent-side moves, the config listing view) and are dropped.

## Technique 1: mutation audit

- `ess verify conform mutate --path ess --target interpreted`: exit 3, ESS-MUTATE-001, 12 baseline
  scenarios ended `error` (the six agent moves). `--known-failing` does not admit `error`.
- Re-run on a copy with `status_message: input.status_message` removed from the six agent moves:
  79 mutants, 50 killed, 0 survived, 1 inconclusive, 16 stillborn, 12 unwitnessed, 9 unavailable.
- The 50 killed mutants are the planted defects of this technique.

## Technique 6: guard analysis

Four guards (`when_subject` on SubscribeToTask and GetTaskPushNotificationConfig, `when_related` on
Create and Delete of a push config) read by hand from the specification. The terminal-state and
other-task guards partition their inputs. Delete has no other-task guard (finding on line 720). No
tool ran, and no planted defect was made for this technique, so its clean result on the other three
guards is unverified.

```findings
[
{"file":"ess/domains/tasks.yaml","line":580,"category":"spec-gap","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"a second CancelTask on a Canceled task answers TaskNotCancelable while A2A section 3.3 calls cancel idempotent; the design contradicts itself, so the answer is an open question"},
{"file":"ess/domains/tasks.yaml","line":301,"category":"spec-gap","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"a push-notification config must persist until task completion or explicit deletion, but nothing ends a config when its task becomes terminal"},
{"file":"ess/domains/tasks.yaml","line":970,"category":"spec-gap","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"listing push-notification configs of an unknown task returns an empty list where A2A requires TaskNotFoundError"},
{"file":"ess/domains/tasks.yaml","line":455,"category":"spec-gap","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"SendMessage with return_immediately unset must block until a terminal or interrupted state, but the spec answers with a Submitted task"},
{"file":"ess/domains/tasks.yaml","line":463,"category":"spec-gap","severity":"note","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"the stream end condition says terminal or interrupted while the cited section 3.1.2 says terminal only; upstream disagrees with itself at section 9"},
{"file":"ess/domains/tasks.yaml","line":449,"category":"spec-gap","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"no outcome rejects a client-supplied contextId the agent cannot accept, which A2A section 3.4 requires"},
{"file":"ess/domains/tasks.yaml","line":441,"category":"spec-gap","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"a message with only taskId must take the task's contextId; the continued outcome does not declare it"},
{"file":"ess/domains/tasks.yaml","line":913,"category":"spec-gap","severity":"blocker","verdict":"CONFIRMED","origin":"pre-existing","message":"TaskStatusUpdated carries task_id and context_id but no status, which a2a.proto marks REQUIRED on TaskStatusUpdateEvent"},
{"file":"ess/domains/tasks.yaml","line":233,"category":"spec-gap","severity":"blocker","verdict":"CONFIRMED","origin":"pre-existing","message":"no command sets a task's artifacts or history and no artifact-update event exists, so results cannot be returned as A2A requires"},
{"file":"ess/domains/tasks.yaml","line":114,"category":"spec-gap","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"artifact_id uniqueness within a task is not declared"},
{"file":"ess/domains/tasks.yaml","line":122,"category":"spec-gap","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"Artifact and Message parts must hold at least one Part; the lists admit empty"},
{"file":"ess/domains/tasks.yaml","line":535,"category":"spec-gap","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"historyLength is accepted but nothing bounds the returned history by it"},
{"file":"ess/domains/tasks.yaml","line":445,"category":"spec-gap","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"a push-notification config in SendMessage configuration creates no PushNotificationConfig"},
{"file":"ess/domains/tasks.yaml","line":799,"category":"spec-gap","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"RequireAuth admits an absent status message where A2A requires one unless negotiated out of band"},
{"file":"ess/domains/tasks.yaml","line":403,"category":"spec-gap","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"no outcome checks the tenant field against the agent interface's tenant"},
{"file":"ess/domains/discovery.yaml","line":347,"category":"spec-gap","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"GetExtendedAgentCard has neither VersionNotSupported nor ExtensionSupportRequired, because the negotiation group binds only to tasks.Client"},
{"file":"ess/components.yaml","line":32,"category":"spec-gap","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"push delivery to configured webhooks on a status change is stated in a summary only"},
{"file":"ess/domains/tasks.yaml","line":720,"category":"spec-gap","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"DeleteTaskPushNotificationConfig deletes a config owned by another task where Get refuses it with TaskNotFound; found by the design review and by guard analysis"},
{"file":"ess/domains/tasks.yaml","line":571,"category":"spec-gap","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"CancelTask and task-created leave status_timestamp unset while every agent move sets it"},
{"file":"ess/domains/tasks.yaml","line":748,"category":"ess-limit","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"the interpreted target ends 12 agent moved scenarios in error (awaited answer carried no status_message), so the mutation baseline is refused with ESS-MUTATE-001 and the audit ran on a copy without those writes"},
{"file":"ess/domains/tasks.yaml","line":758,"category":"ess-limit","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"12 error-swap mutants on the six agent moves' not-found and wrong-state refusals are unwitnessed: the synthesized suite does not change when the error name changes"}
]
```
