---
format: aep.planning-md/3
id: story:server
kind: story
status: draft
title: 'a2a-server: Rust handler over the generated contracts'
summary: crates/a2a-server implements the generated Behavior traits and serves A2A's HTTP+JSON binding; it takes every model type from generated/rust/a2a and defines none.
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:http-binding
- depends_on: story:streaming-binding
- depends_on: story:resolve-unmapped
revision: 1
---
## Outcome

a2a-server: Rust handler over the generated contracts.

## Acceptance

crates/a2a-server implements the generated Behavior traits and serves A2A's HTTP+JSON binding; it takes every model type from generated/rust/a2a and defines none.
