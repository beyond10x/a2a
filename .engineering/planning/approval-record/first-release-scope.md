---
format: aep.planning-md/3
id: approval-record:first-release-scope
kind: approval-record
status: draft
title: 0.1.0 scope and waves 1-3 approved by the orchestrator
relations:
- decides: story:first-release
- decides: story:resolve-unmapped
- decides: story:jsonrpc-binding
- decides: story:streaming-binding
- decides: story:http-binding
- decides: story:docs-site
revision: 2
---
## Decision

b10x-chef-orchestrator, 2026-10-22, relaying Timo ("adopt this A2A thing"): "carry your a2a plan in
.engineering/planning to a first release (0.1.0) through the repo's own process: spec, projections
(proto copy, OpenAPI, AsyncAPI, schema) as release artifacts, generated Rust crate, task check green.
No registry publishing (Timo's). Move to ess 0.68.0 when I send the tag. The Shared source gates
failure waits on Timo's secret refresh; do not work around it."

Follow-up, same day: "Plan approved as sent: hardening, 0.1.0, then waves 1-3, each closing with a
minor release (all under the grant; record as approval-record). If a binding needs the ESS
proto/JSON-RPC/gRPC features (ess stories 7-8), keep that part UNMAPPED and go on; do not block a
wave on ess."

Timo, in this session: "Yeah, and then you can cut the first new version, I think."

## Effect

0.1.0 ships the specification, the projections and the synthesized crate. Server, client, test kit
and documentation site follow in waves 1-3, each closing with a minor release.

Wave 2, b10x-chef-orchestrator, 2026-10-22, after verifying 0.2.0: "Wave 2 (http-binding, docs-site)
approved under the plan of 07:3x."
