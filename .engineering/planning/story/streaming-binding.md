---
format: aep.planning-md/3
id: story:streaming-binding
kind: story
status: active
title: Specify SSE streaming for SendStreamingMessage and SubscribeToTask
summary: StreamResponse events (task, message, statusUpdate, artifactUpdate) are declared and projected to AsyncAPI; generated/asyncapi carries both streams.
relations:
- decomposes: epic:a2a-delivery
scope:
- confidence: inferred
  path: ess/domains/tasks.yaml
- confidence: inferred
  path: generated/asyncapi
- confidence: inferred
  path: generated/rust/a2a
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T05:01:35Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-22T05:01:35Z", actor: "human:timo", revision: 6}
---
## Outcome

Specify SSE streaming for SendStreamingMessage and SubscribeToTask.

## Acceptance

StreamResponse events (task, message, statusUpdate, artifactUpdate) are declared and projected to AsyncAPI; generated/asyncapi carries both streams.
