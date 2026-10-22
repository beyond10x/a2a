---
format: aep.planning-md/3
id: story:client
kind: story
status: draft
title: 'a2a-client: typed Rust client'
summary: crates/a2a-client sends every A2A command over HTTP+JSON using generated types and decodes every A2A error; tested against a2a-server.
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:server
revision: 1
---
## Outcome

a2a-client: typed Rust client.

## Acceptance

crates/a2a-client sends every A2A command over HTTP+JSON using generated types and decodes every A2A error; tested against a2a-server.
