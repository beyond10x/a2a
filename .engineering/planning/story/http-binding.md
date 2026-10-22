---
format: aep.planning-md/3
id: story:http-binding
kind: story
status: implemented
title: Bind commands and views to A2A's HTTP+JSON paths
summary: An ess-http/1 document maps every command and view to A2A's HTTP+JSON method and path (POST /message:send, GET /tasks/{id}, ...); ess generate --kind openapi --http emits paths equal to upstream/openapi/a2a.openapi.yaml, checked by task check.
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:resolve-unmapped
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/a2a-xtask
- confidence: cited
  path: ess/http-unmapped.tsv
- confidence: cited
  path: ess/http.yaml
- confidence: cited
  path: generated/openapi-http
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T08:19:26Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-22T08:19:27Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-22T10:16:19Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
## Outcome

Bind commands and views to A2A's HTTP+JSON paths.

## Acceptance

An ess-http/1 document maps every command and view to A2A's HTTP+JSON method and path (POST /message:send, GET /tasks/{id}, ...); ess generate --kind openapi --http emits paths equal to upstream/openapi/a2a.openapi.yaml, checked by task check.

## Scope

Read at wave close from `git diff --stat 91efc36^1 91efc36` (merge 91efc36 into wave/2). The inferred lines `ess/ess-inputs.yaml` and `generated/openapi` did not land: the binding is a document beside the specification and projects into the new `generated/openapi-http`.

- `AGENTS.md`
- `CHANGELOG.md`
- `Taskfile.yml`
- `crates/a2a-xtask/src/main.rs`
- `crates/a2a-xtask/tests/adv_http_binding.rs`
- `crates/a2a-xtask/tests/adv_http_binding_pass2.rs`
- `ess/http-unmapped.tsv`
- `ess/http.yaml`
- `generated/openapi-http/.ess-output/state.json`
- `generated/openapi-http/openapi/a2a-agent.yaml`
- `generated/rust/a2a/.ess-output/state.json`
