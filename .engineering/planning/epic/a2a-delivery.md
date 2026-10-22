---
format: aep.planning-md/3
id: epic:a2a-delivery
kind: epic
status: draft
title: A2A v1.0.1 delivered from one ESS specification
summary: Spec, published artifacts, generated Rust model, server, client, conformance suite, test kit and documentation site.
revision: 1
---
## Outcome

A public, gated beyond10x repository that delivers A2A v1.0.1 from one ESS specification: the
upstream proto, OpenAPI, AsyncAPI and JSON Schema as published artifacts; a generated Rust data model;
a Rust server (handler) and client; a conformance suite; a test kit with assertions that drives any
A2A endpoint; and a public documentation site like ESS's.

## Acceptance

`task check` validates `ess/` with 0 refusals and 0 `UNMAPPED:` markers, holds every generated
artifact to it, and runs the synthesized conformance suite against `a2a-server` with 0 failed and 0
errored; `a2a-testkit` passes against `a2a-server` and fails against a planted defect; the site at
https://beyond10x.github.io/a2a/ serves the proto, OpenAPI, AsyncAPI and schema downloads.

## Rule

Model types come from `ess generate synthesize`, never hand transcription. Running code is Rust.
