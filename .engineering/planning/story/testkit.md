---
format: aep.planning-md/3
id: story:testkit
kind: story
status: draft
title: 'a2a-testkit: assertions harness for any A2A endpoint'
summary: crates/a2a-testkit ships a CLI and library that run the conformance scenarios and assertion helpers against an A2A endpoint by URL; green on a2a-server, red on a planted defect.
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:conformance-gate
revision: 1
---
## Outcome

a2a-testkit: assertions harness for any A2A endpoint.

## Acceptance

crates/a2a-testkit ships a CLI and library that run the conformance scenarios and assertion helpers against an A2A endpoint by URL; green on a2a-server, red on a planted defect.
