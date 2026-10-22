---
format: aep.planning-md/3
id: story:conformance-gate
kind: story
status: draft
title: Run the synthesized conformance suite against a2a-server in task check
summary: task check runs the suite synthesized from ess/ against a2a-server with 0 failed and 0 errored, and a planted defect turns it red.
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:server
revision: 1
---
## Outcome

Run the synthesized conformance suite against a2a-server in task check.

## Acceptance

task check runs the suite synthesized from ess/ against a2a-server with 0 failed and 0 errored, and a planted defect turns it red.
