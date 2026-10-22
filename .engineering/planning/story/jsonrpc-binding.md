---
format: aep.planning-md/3
id: story:jsonrpc-binding
kind: story
status: active
title: Specify the JSON-RPC 2.0 binding
summary: The A2A JSON-RPC method names (SendMessage, GetTask, ...) and error codes (-32001 ... -32009) are specified and held by a conformance scenario per method.
relations:
- decomposes: epic:a2a-delivery
scope:
- confidence: inferred
  path: ess/domains/tasks.yaml
- confidence: inferred
  path: generated/rust/a2a
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T05:01:35Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-22T05:01:35Z", actor: "human:timo", revision: 5}
---
## Outcome

Specify the JSON-RPC 2.0 binding.

## Acceptance

The A2A JSON-RPC method names (SendMessage, GetTask, ...) and error codes (-32001 ... -32009) are specified and held by a conformance scenario per method.
