---
format: aep.planning-md/3
id: story:docs-site
kind: story
status: active
title: Public documentation site with artifact downloads
summary: website/ is Docusaurus on @beyond10x/docs-system; generated reference pages are drift-checked by task check; the published site serves a2a.proto, OpenAPI, AsyncAPI and JSON Schema downloads.
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:http-binding
scope:
- confidence: inferred
  path: .github/workflows
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: crates
- confidence: inferred
  path: website
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T08:19:27Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-22T08:19:27Z", actor: "human:timo", revision: 4}
---
## Outcome

Public documentation site with artifact downloads.

## Acceptance

website/ is Docusaurus on @beyond10x/docs-system; generated reference pages are drift-checked by task check; the published site serves a2a.proto, OpenAPI, AsyncAPI and JSON Schema downloads.
