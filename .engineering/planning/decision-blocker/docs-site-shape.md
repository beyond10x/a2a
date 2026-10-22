---
format: aep.planning-md/3
id: decision-blocker:docs-site-shape
kind: decision-blocker
status: cleared
title: docs-site build, pin and downloads
relations:
- blocks: story:docs-site
revision: 2
transitions:
- {from: "open", to: "cleared", at: "2026-10-22T08:54:41Z", actor: "human:timo", revision: 2}
---
## Question

How does story:docs-site build, pin and serve its downloads, and how far does the unit publish?

## Decision

- Stage A of the docs skill only inside the unit; the bot push, Atlas roster, live checks and homepage come after the wave merges.
- `@beyond10x/docs-system` pinned by git sha to its main at de73baa (0.11.0); loom's pin is the fallback.
- A Rust crate `crates/a2a-docs` (clap) generates the reference pages, the HTTP+JSON route table and the downloads; `task check` runs `a2a-docs generate --check`.
- "Serves downloads": a2a.proto, both OpenAPI projections, the AsyncAPI and the JSON Schema are copied into `website/static/downloads/`, committed and drift-checked; the site also links the latest GitHub Release assets.

Taken by the wave coordinator under approval-record:first-release-scope.
