---
format: aep.planning-md/3
id: story:http-binding
kind: story
status: active
title: Bind commands and views to A2A's HTTP+JSON paths
summary: An ess-http/1 document maps every command and view to A2A's HTTP+JSON method and path (POST /message:send, GET /tasks/{id}, ...); ess generate --kind openapi --http emits paths equal to upstream/openapi/a2a.openapi.yaml, checked by task check.
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:resolve-unmapped
scope:
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: ess/ess-inputs.yaml
- confidence: inferred
  path: ess/http.yaml
- confidence: inferred
  path: generated/openapi
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T08:19:26Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-22T08:19:27Z", actor: "human:timo", revision: 4}
---
## Outcome

Bind commands and views to A2A's HTTP+JSON paths.

## Acceptance

An ess-http/1 document maps every command and view to A2A's HTTP+JSON method and path (POST /message:send, GET /tasks/{id}, ...); ess generate --kind openapi --http emits paths equal to upstream/openapi/a2a.openapi.yaml, checked by task check.
