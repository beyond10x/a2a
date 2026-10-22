---
format: aep.planning-md/3
id: decision-blocker:conformance-in-check
kind: decision-blocker
status: cleared
title: task check runs conformance from wave 3
relations:
- blocks: story:conformance-gate
revision: 2
transitions:
- {from: "open", to: "cleared", at: "2026-10-22T08:04:51Z", actor: "human:timo", revision: 2}
---
## Question

Does `task check` run the conformance suite in wave 1?

Raised during the wave: every unit ran `task conform-interpreted` by hand, and `task check` does not
call it (Taskfile.yml).

## Decision

No. story:conformance-gate in wave 3 owns wiring the suite into `task check`, against a2a-server.
Until then, `task conform-interpreted` exits 201 on the 16 known errors (12 awaited answers with no
`status_message`, 4 grant/denied cells with no consistent view query); wave close read
168 scenarios, 152 passed, 0 failed, 16 error (~/.cache/a2a-gen/wave1/close-conform2.log).

Taken by the wave coordinator under approval-record:first-release-scope.
