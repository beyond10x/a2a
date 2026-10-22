---
format: aep.planning-md/3
id: story:resolve-unmapped
kind: story
status: active
title: Resolve the UNMAPPED markers in ess/ from upstream sources
summary: Each of the 12 UNMAPPED markers in ess/domains is resolved from a2a.proto or the A2A v1.0.1 specification text, or reported to ESS as a missing construct; ess specify validate passes and synthesis has 0 refusals.
relations:
- decomposes: epic:a2a-delivery
scope:
- confidence: inferred
  path: ess/domains/discovery.yaml
- confidence: inferred
  path: ess/domains/tasks.yaml
- confidence: inferred
  path: generated/rust/a2a
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T05:01:34Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-22T05:01:34Z", actor: "human:timo", revision: 6}
---
## Outcome

Resolve the UNMAPPED markers in ess/ from upstream sources.

## Acceptance

Each of the 12 UNMAPPED markers in ess/domains is resolved from a2a.proto or the A2A v1.0.1 specification text, or reported to ESS as a missing construct; ess specify validate passes and synthesis has 0 refusals.
