---
format: aep.planning-md/3
id: story:docs-site
kind: story
status: implemented
title: Public documentation site with artifact downloads
summary: website/ is Docusaurus on @beyond10x/docs-system; generated reference pages are drift-checked by task check; the published site serves a2a.proto, OpenAPI, AsyncAPI and JSON Schema downloads.
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:http-binding
scope:
- confidence: cited
  path: .github/workflows
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/a2a-docs
- confidence: cited
  path: website
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T08:19:27Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-22T08:19:27Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-22T10:16:19Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Outcome

Public documentation site with artifact downloads.

## Acceptance

website/ is Docusaurus on @beyond10x/docs-system; generated reference pages are drift-checked by task check; the published site serves a2a.proto, OpenAPI, AsyncAPI and JSON Schema downloads.

## Scope

Read at wave close from `git diff --stat 9e1a2e1^1 9e1a2e1` (merge 9e1a2e1 into wave/2), grouped by directory. `website/static/downloads/` and `website/docs/reference/` are written by `a2a-docs generate`.

- `AGENTS.md`
- `Cargo.lock`
- `CHANGELOG.md`
- `crates/a2a-docs/`
- `.github/workflows/b10x-docs-site.yml`
- `.github/workflows/pages.yml`
- `README.md`
- `Taskfile.yml`
- `website/docs/concepts/`
- `website/docs/guides/`
- `website/docs/index.md`
- `website/docs/reference/`
- `website/docusaurus.config.ts`
- `website/.gitignore`
- `website/.npmrc`
- `website/package.json`
- `website/package-lock.json`
- `website/product.json`
- `website/sidebars.ts`
- `website/static/downloads/`
- `website/static/.nojekyll`
- `website/tsconfig.json`
