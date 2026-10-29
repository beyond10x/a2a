---
format: aep.planning-md/3
id: approval-record:wave-5-scope
kind: approval-record
status: draft
title: Wave 5 (view-tenant-scope, testkit-wire-events) approved by the orchestrator (D211)
relations:
- decides: story:view-tenant-scope
- decides: story:testkit-wire-events
revision: 1
---
## Decision

b10x-chef-orchestrator, 2026-10-29 (D211), under the grant in the workspace instructions: "a2a
wave 5 and its release. Approved under the grant" ... "I accept both stories (approval-record
citing D211; run the plan critics if the store requires; fix findings in the text).
view-tenant-scope first and with a security-reviewer pass" ... "Where ESS cannot express the
guard (gap A), enforce and test it in the Rust server and record the spec gap; do not wait on
ESS. Then testkit-wire-events. Wave cap 4 units." Same message (D198, 2026-10-29): "do NOT move
the ESS pin (0.75.0)."

## Effect

story:view-tenant-scope and story:testkit-wire-events move draft -> proposed -> active after one
plan-critic round (review-result:plan-critics-wave-5), whose findings were fixed in the story
text. Wave 5 runs view-tenant-scope, then testkit-wire-events, on one integration branch
(`wave/5-int`), and closes with one release on ESS 0.75.0.
