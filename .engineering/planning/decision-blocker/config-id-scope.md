---
format: aep.planning-md/3
id: decision-blocker:config-id-scope
kind: decision-blocker
status: cleared
title: Push config id is scoped to its parent task
relations:
- blocks: story:resolve-unmapped
revision: 2
transitions:
- {from: "open", to: "cleared", at: "2026-10-22T08:04:51Z", actor: "human:timo", revision: 2}
---
## Question

Is a push-notification config id unique per agent or per parent task?

Raised by review-result:adversary-resolve-unmapped-pass-1 (finding at ess/domains/tasks.yaml:749).

## Decision

Per parent task. `upstream/a2a.proto` nests the config under its task
(`tasks/{task_id}/pushNotificationConfigs/{config_id}`), and specification.md 1.4 makes the proto
normative. A create of the same id under another task is a separate config.

Taken by the wave coordinator under approval-record:first-release-scope; landed in unit 59a1671,
merge 7c6b8d3.

## Left open

A second create of the same config id under the same task: A2A v1.0.1 says nothing. It stays
UNMAPPED in ess/domains/tasks.yaml and goes to the orchestrator as an upstream question.
