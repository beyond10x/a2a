---
format: aep.planning-md/3
id: decision-blocker:blocking-send-message
kind: decision-blocker
status: cleared
title: Blocking SendMessage follows a2a.proto
relations:
- blocks: story:resolve-unmapped
revision: 2
transitions:
- {from: "open", to: "cleared", at: "2026-10-22T08:04:51Z", actor: "human:timo", revision: 2}
---
## Question

Does SendMessage block by default (specification.md 3.1.1) or follow `SendMessageConfiguration.blocking`
as `upstream/a2a.proto` declares it?

Raised by review-result:adversary-resolve-unmapped-pass-1 (finding at ess/domains/tasks.yaml:433).

## Decision

Follow the proto. Specification.md 1.4 makes `a2a.proto` the normative source where the two differ.
The misattributed proto quote was corrected.

Taken by the wave coordinator under approval-record:first-release-scope; landed in unit 59a1671,
merge 7c6b8d3.
