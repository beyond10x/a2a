---
format: aep.planning-md/3
id: story:http-binding
kind: story
status: draft
title: Bind commands and views to A2A's HTTP+JSON paths
summary: An ess-http/1 document maps every command and view to A2A's HTTP+JSON method and path (POST /message:send, GET /tasks/{id}, ...); ess generate --kind openapi --http emits paths equal to upstream/openapi/a2a.openapi.yaml, checked by task check.
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:resolve-unmapped
revision: 1
---
## Outcome

Bind commands and views to A2A's HTTP+JSON paths.

## Acceptance

An ess-http/1 document maps every command and view to A2A's HTTP+JSON method and path (POST /message:send, GET /tasks/{id}, ...); ess generate --kind openapi --http emits paths equal to upstream/openapi/a2a.openapi.yaml, checked by task check.
