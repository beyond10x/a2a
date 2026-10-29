---
format: aep.planning-md/3
id: approval-record:wave-4-scope
kind: approval-record
status: draft
title: Wave 4 (client, conformance-gate, testkit) approved by the orchestrator (D201)
relations:
- decides: story:client
- decides: story:conformance-gate
- decides: story:testkit
revision: 1
---
## Decision

b10x-chef-orchestrator, 2026-10-29 (D201), under the grant in the workspace instructions:
"a2a wave 4 (client, conformance-gate, testkit) and its release. Approved under the grant" ...
"I decide these three for this wave (D201). Move them draft -> proposed -> accepted with the
store's own verbs, recording an approval-record that quotes D201 as the orchestrator's decision
under the grant." Same message (D198, 2026-10-29): "do NOT move the ESS pin (stays 0.75.0)."

## Effect

story:client, story:conformance-gate and story:testkit move draft -> proposed -> active after
one plan-critic round (review-result:plan-critics-wave-4), whose findings were fixed in the
story text. Wave 4 runs client and conformance-gate in parallel, then testkit, on one integration
branch, and closes with release 0.5.0 on ESS 0.75.0.
