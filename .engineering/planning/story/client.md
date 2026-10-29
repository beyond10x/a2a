---
format: aep.planning-md/3
id: story:client
kind: story
status: active
title: 'a2a-client: typed Rust client'
summary: 'crates/a2a-client: one typed method per A2A HTTP+JSON operation over the generated types, every google.rpc.Status error decoded to a typed reason; tested against a2a-server.'
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:server
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: README.md
- confidence: cited
  path: crates/a2a-client
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T11:21:59Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-29T11:21:59Z", actor: "human:timo", revision: 7}
---
## Outcome

A Rust program talks to an A2A v1.0.1 agent over the HTTP+JSON binding (section 11) through
`crates/a2a-client`, with the request and answer types of `generated/rust/a2a` and every
`google.rpc.Status` error decoded into a typed value. `a2a-testkit` (story:testkit) drives
endpoints through it.

## Acceptance

1. `crates/a2a-client` has one public method per A2A operation of the HTTP+JSON binding: the
   public card (`GET /.well-known/agent-card.json`), `SendMessage`, `SendStreamingMessage`,
   `GetTask`, `ListTasks`, `CancelTask`, `SubscribeToTask`, `Create/Get/List/Delete
   TaskPushNotificationConfig` and `GetExtendedAgentCard`, each with an optional tenant that
   selects the `/{tenant}/` route. A test records, with a listening socket, the (method, path)
   each client method sends with and without a tenant, and fails on any pair of
   `upstream/openapi/a2a.openapi.yaml` no method sends, and on any sent pair that is neither in
   that file nor the one named exception `GET /.well-known/agent-card.json`.
2. Against `a2a-server` with the echo agent on port 0, one test per method asserts a value:
   the card's `name` is the built-in card's; `SendMessage` returns a task whose id is new;
   `GetTask` returns that task (same id and state); `ListTasks` lists it; `CancelTask` on a task
   in `TASK_STATE_INPUT_REQUIRED` (the echo agent's answer to a message with no content) returns it in `TASK_STATE_CANCELED`; the push config methods
   return the config created, list it, get it by id and delete it so a following get is answered 404 with
   status code `NOT_FOUND`; `GetExtendedAgentCard` with the configured bearer token
   returns the extended card's `name`. Each runs once with and once without a tenant.
   `SendStreamingMessage` and `SubscribeToTask` yield the server-sent events in order as typed
   `StreamResponse`s: the first is the task, the last carries a terminal or interrupted state,
   and the iterator ends when the server closes the stream after a complete event.
3. Every error answer is decoded into `a2a_client::Error::Status`: the HTTP status, the
   `google.rpc.Status` code and message, and the `ErrorInfo` reason as an enum with one variant
   per reason the A2A v1.0.1 specification's error table names (each variant's doc cites it) and
   per reason `a2a-server` answers (`crates/a2a-server/src/router.rs`), plus `Other(String)`. A
   test provokes each reason `a2a-server` can be made to answer and asserts its variant; a
   reason it cannot is decoded from an answer body written out in the test.
4. An answer that is not the binding's is `Error::Protocol`, never a panic: a non-JSON body, a
   2xx body that does not decode as the operation's type, and a stream that ends inside an event
   (no blank-line terminator). A test feeds each from a socket that answers with those bytes.
5. `crates/a2a-client` defines no model type: the test `defines_none` (as in `a2a-server`)
   fails on a `struct` or `enum` that repeats a type `generated/rust/a2a` has.
6. Transport is HTTP/1.1 over `std::net::TcpStream` to `http://` base URLs; an `https://` URL is
   refused with `Error::UnsupportedScheme`. TLS and the JSON-RPC binding are a later milestone,
   not this story.

## Scope

- `crates/a2a-client/` (new crate: `src/`, `tests/`)
- `Cargo.lock`
- `CHANGELOG.md` (`## [Unreleased]`), `README.md`, `AGENTS.md` (Layout row)

Shares `Cargo.lock`, `CHANGELOG.md`, `README.md` and `AGENTS.md` with story:conformance-gate,
which runs at the same time; the wave coordinator resolves those merges, no ordering edge.
