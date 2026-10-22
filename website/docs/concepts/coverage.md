---
title: What the specification covers
sidebar_label: Coverage
description: What of A2A v1.0.1 the ESS specification holds, which document holds which part, and what is not covered yet.
---

## Sources

The specification in `ess/` is written from two sources only: `upstream/a2a.proto`, verbatim from
A2A's v1.0.1 tag, and the A2A v1.0.1 specification text. Each domain names the sections and proto
messages it follows, and the comments in `ess/` quote the text a rule comes from.

## What it holds

| Part of A2A | Where it is |
|---|---|
| Tasks, their messages and artifacts, and push-notification configs (sections 3 and 4.1-4.3) | the [Tasks](../reference/specification/a2a-tasks.md) domain, `a2a.tasks` |
| The Agent Card and the extended Agent Card (section 4.4) | the [Discovery](../reference/specification/a2a-discovery.md) domain, `a2a.discovery` |
| The server agent, reached over JSON-RPC, gRPC or HTTP+JSON (section 5) | the component `a2a-agent`, on the [specification overview](../reference/specification/index.md) |
| JSON-RPC method names (section 9) | each method is the local name of the command it invokes; ListTasks is the view `a2a.tasks.Tasks` |
| A2A error codes (section 5.4) | each A2A error's summary carries its code |
| Which actor may read which view (section 13.1) | the read grants of the views |

Conformance scenarios in `ess/scenarios/` state the behaviour as acts and the outcomes expected of
them.

## The documents generated from it

| Document | What it describes |
|---|---|
| `generated/openapi/` | ESS's own HTTP surface: one route per command, not A2A's paths |
| `generated/openapi-http/` | The specification bound to A2A's HTTP+JSON paths through `ess/http.yaml`, where ESS can bind them |
| `generated/asyncapi/` | The events the agent publishes, including the two response streams |
| `generated/schema/` | JSON Schema 2020-12 files for the types, commands, responses, events, entities and errors |
| `generated/rust/a2a/` | A Rust crate synthesized from the specification |

`upstream/openapi/a2a.openapi.yaml` is not generated from the specification. It is derived from
`a2a.proto`, because A2A publishes no OpenAPI document, and it is the reference the HTTP+JSON
binding is compared with on [HTTP+JSON routes](../reference/http-routes.md).

All of these except the Rust crate are on [Downloads](../reference/downloads.md).

## What it does not hold yet

- Each rule of A2A that ESS cannot express yet is marked `UNMAPPED` beside the place it belongs.
  [UNMAPPED markers and ESS gaps](./gaps.md) explains the markers, and the
  [marker index](../reference/unmapped.md) lists them. Among them are the JSON-RPC binding itself,
  push delivery to a webhook and the Server-Sent Events stream transport.
- The repository has no server, client or test kit yet. Its hand-written Rust today is the
  repository's checks and this site's generator.
