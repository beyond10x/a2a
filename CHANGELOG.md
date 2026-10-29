# Changelog

All notable changes to this repository are recorded here, in the format of
[Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). Releases are bare-version tags such as
`0.1.0`; nothing is on a registry. Each GitHub Release carries the upstream `a2a.proto`, the OpenAPI,
AsyncAPI and JSON Schema documents, the specification bundle and a `SHA256SUMS` file. Changes collect
under `[Unreleased]` until the next release.

## [Unreleased]

## [0.5.0] - 2026-10-29

### Added

- `crates/a2a-client`: a typed client of A2A's HTTP+JSON binding over the generated crate. One
  method per operation (the public card, `SendMessage`, `SendStreamingMessage`, `GetTask`,
  `ListTasks`, `CancelTask`, `SubscribeToTask`, the four push notification config operations and
  `GetExtendedAgentCard`); a request's `tenant` selects the `/{tenant}/` route. Error answers are
  `Error::Status` with the HTTP status, the `google.rpc.Status` code, status and message, and the
  `ErrorInfo` reason as a `Reason`; an answer that is not the binding's is `Error::Protocol`.
  Streams are iterators of typed stream responses. HTTP/1.1 over `std::net::TcpStream` to
  `http://` URLs; `https://` is refused with `Error::UnsupportedScheme`. It declares no model
  type of the generated crate; `ListTasksRequest` and `ListTasksResponse` are its own, since the
  generated crate has none.
- `crates/a2a-conformance`: an interpreter of `ess-conformance/52` suites against a `Target` trait
  (execute a command as an actor, query a view, force an external outcome, read the events
  published since the last step; `TargetError::Unsupported` makes a scenario `unsupported`), which
  writes `ess-conformance-results/1`. Its in-process target runs every scenario on a fresh
  `a2a-server` through the generated `System`. `task conform`, now in `task check`, synthesizes the
  suite, runs it, reports with `ess verify conform report` and fails unless 0 failed, 0 error and
  every `unsupported` scenario is listed with its reason in `crates/a2a-conformance/unsupported.tsv`.
  On ESS 0.75.0: 169 of 182 pass, 13 unsupported (10 on push configs across tenants, whose tenant
  guard `ess/domains/tasks.yaml` leaves `UNMAPPED`; 3 authored claims whose `status` literal leaves
  out the `timestamp` the server stamps). An unsupported scenario's reason joins every distinct
  reason its checks gave. A configuration that only arranges what a later step reads, on a task of
  another tenant and naming no tenant, is created under the task's tenant, so the step the
  scenario is about runs. The extended card the target serves differs from the public card and is
  read with a bearer token, as an authenticated client sends it. A scenario's `arrange` setup of an
  `a2a.tasks.Task` (`establish_entity`) is stored in the server as given; another target answers
  it `unsupported`.
- Authored scenarios under `ess/scenarios/`: the `TaskPushNotificationConfigs` view of a task
  under its own tenant answers its url and token; `Tasks` under another tenant, or with no tenant,
  leaves out a tenant's task; `Tasks` with a `status` leaves out a task in another state; `GetTask`
  with `history_length: 1` answers the last message only; `ListTaskPushNotificationConfigs` with
  `page_size: 1` answers the first configuration only.
- `a2a-server` exports `InProcess`, `Obligations` and `ServerPorts`: the server's obligations run
  without HTTP, settings and limits reconfigurable between calls. `InProcess` is a trusted harness
  entry: it runs a command for whatever caller it is handed, and reads the
  `TaskPushNotificationConfigs` view, which has no tenant scope and which the binary serves on no
  route, for any task. `with_access` gives it the extended card's bearer token, which
  `call_presenting` checks as the binary does; the agent's steps run through the code the server
  loop runs them with.
- `crates/a2a-testkit`: checks any A2A endpoint from outside. `WireTarget` is a `Target` of
  `a2a-conformance` over `a2a-client`, each command sent as the operation `OPERATIONS` binds it to
  (the routes of `ess/http.yaml` plus the two `ess/http-unmapped.tsv` names), the tenant taken from
  the scenario's input, and each answer read as the outcome the generated contract declares for
  it. An error counts only at the HTTP status and `google.rpc.Status` status A2A maps its reason
  to, and the external outcomes the scenario rules out (`VERSION_NOT_SUPPORTED` to the client's own
  version, a refusal of a capability the card declares, a task that cannot be canceled when there
  is no task) are set aside first. An outcome other than the one the scenario expects is `failed`.
  A published event, a forced or taken external outcome, an actor other than the client, a view
  with no A2A operation and a read of `a2a.tasks.Tasks`, whose state a live endpoint cannot reset,
  are `unsupported` with that reason. `a2a-testkit run --url <base> --suite <suite.json>
  [--results <out>] [--bearer-file <path>]` prints one line per scenario and the counts, then
  writes `ess-conformance-results/1` with `--results`, and exits 0 only when at least one scenario
  passed and none is failed or error. `--bearer-file` sends the token the file holds and never
  prints it; without it, an extended card served only to authenticated clients is `unsupported`.
  Against `a2a-server` 7 of 182 pass and 175 are unsupported. The helpers `expect_card`,
  `expect_task_state`, `expect_error_reason`, `expect_stream_states` and
  `expect_push_config_roundtrip` answer a `Mismatch` naming the expected and the observed value
  instead of panicking. They check the task's id, the mapped status of an error, that a stream
  starts with its task and carries updates of that task only, and a push config's authentication.

### Changed

- `Taskfile.yml` no longer points every checkout at one shared `CARGO_TARGET_DIR`; each checkout
  builds into its own `target/`.

## [0.4.0] - 2026-10-29

### Added

- `crates/a2a-server` serves A2A's HTTP+JSON binding over the generated crate's `server` feature
  and defines no model type of its own. It implements the 12 obligations of
  `generated/rust/a2a/PLAN.md` over stores of its own, answers all 22 upstream (method, path)
  pairs (9 through the generated layer, 13 by hand) plus `/.well-known/agent-card.json`, and
  streams over server-sent events. Errors are `google.rpc.Status`. The `a2a-server` binary takes
  `--listen`, `--card`, `--extended-card`, `--extended-card-token-file`, `--max-connections`,
  `--request-timeout`, `--max-subscribers-per-task`, `--max-subscribers`, `--max-tasks`,
  `--max-history` and `--max-push-configs`. The agent's steps run as commands of the system's
  `agent` actor, and streams carry the events the system publishes. Status timestamps carry
  milliseconds, one instant per command.
- Connections: each connection has its own reading and writing thread, so a client that sends or
  reads slowly holds back no other. At most `--max-connections` (default 256, fewer where the file
  descriptor limit would run out first) are held; over the cap the connection waiting longest for
  a request head, before or after its answer, is closed, or the new one when every connection is
  being answered. A request not in within `--request-timeout` seconds (default 10) of its
  connection opening is answered 408 `DEADLINE_EXCEEDED`; bytes arriving do not extend it, and a
  connection that sent nothing is closed without an answer. Once its answer is written, a
  connection is closed when as long again has passed. At most 4 MiB waits for one connection's
  writer; a stream whose client falls further behind is dropped and closed. A stream whose client
  hung up or half-closed is dropped within 250 ms of the end of its input; bytes it sends before
  that are ignored.
- Bounds: at most `--max-subscribers-per-task` streams (default 16) follow one task and
  `--max-subscribers` (default half of `--max-connections`) in all; at most `--max-tasks` tasks
  (default 10000) per tenant, requests without a tenant counting as one tenant; at most
  `--max-history` messages (default 1000) in one task's history and `--max-push-configs` push
  notification configs (default 16) per task. A request over a bound is answered 429
  `RESOURCE_EXHAUSTED` and keeps nothing; replacing a push notification config of the same id is
  not refused. A status message an agent's step adds to a history already at its bound is not
  kept. ListTasks orders and pages by status timestamp and id and reads only its page's tasks, and
  GetTask copies only the `historyLength` messages it answers.
- `GET /extendedAgentCard` (and its tenant twin) answers 401 `UNAUTHENTICATED` with
  `WWW-Authenticate: Bearer` unless the request carries `Authorization: Bearer <token>`, the
  token read from `--extended-card-token-file` and compared in constant time. `--extended-card`
  without the token file is refused at startup, and the token is never logged.
- A request head over its limit is answered 431 and a chunked body 411, as `google.rpc.Status`
  `INVALID_ARGUMENT`, no longer 500. The generated layer's answers are read back with a nesting
  limit of 128, so a message within the request limit of 64 leaves its task readable.
- Tenants: a task belongs to the tenant of the route that created it, and a plain route creates
  and sees only tenant-less tasks. ListTasks lists only the request's tenant's tasks. GetTask,
  CancelTask, SubscribeToTask, SendMessage with a `taskId` and the four push-config operations
  answer another tenant's task exactly as an unknown id.
- Page tokens are cursors bound to their operation: a ListTasks token is refused by
  ListTaskPushNotificationConfigs and the reverse, and a token this server did not issue
  (including `0a` and `+0`) is answered 400 `INVALID_ARGUMENT`.
- `ess/domains/tasks.yaml`: a task keeps the `tenant` it was created under, and the `Tasks` view
  names its fields and parameters as the wire does (`contextId`, `statusTimestampAfter`) and takes
  a `tenant` parameter. GetTask, CancelTask, SubscribeToTask and
  ListTaskPushNotificationConfigs answer `TaskNotFound` for a task of another tenant, with an
  `other-tenant` scenario each.
  `ess/http.yaml` ignores unknown query fields on its four query routes; the specification says
  implementations "SHOULD ignore unrecognized fields".
  `ess/domains/tasks.yaml` declares the error `ResourceExhausted` and the outcomes `task-limit` and
  `history-limit` of SendMessage and SendStreamingMessage and `config-limit` of
  CreateTaskPushNotificationConfig.

### Changed

- The documentation site uses `@beyond10x/docs-system` 0.12.0 with `product: 'a2a'`: the A2A
  mark and signature hue, a generated favicon, and `.nojekyll` in the build.
- ESS pinned to 0.69.2 (`ess/ess-inputs.yaml`, the `check` and `pages` workflows), and
  `generated/rust/a2a` resynthesized: its public types are unchanged; `json` gains `parse_written`
  and `admitted`, the `server` feature reads its own answers back at any depth instead of panicking,
  and the crate's dependency requirements are caret ranges instead of exact versions.
- ESS pinned to 0.75.0, and `generated/rust/a2a` resynthesized: its public types are unchanged; the
  `server` feature gains `dispatch_with_events`, `handle_with_events` and per-event wire decoders.
- `task check` runs `cargo fmt --check` instead of `cargo fmt --all --check`. `--all` also reaches
  path dependencies, which now include the synthesized `generated/rust/a2a`, and that crate is not
  rustfmt-clean. The workspace crates are checked as before.

## [0.3.0] - 2026-10-22

### Added

- `ess/http.yaml`, an `ess-http/1` binding of A2A's HTTP+JSON paths, projected to
  `generated/openapi-http/`. `task check` holds its (method, path) pairs to
  `upstream/openapi/a2a.openapi.yaml`: 9 of 22 are equal, and every difference (the `/{tenant}/`
  variants, `:cancel` and `:subscribe`, the agent-side operations) is named with its reason in
  `ess/http-unmapped.tsv`.
- A documentation site in `website/`, Docusaurus on `@beyond10x/docs-system` 0.11.0, to be served
  at <https://beyond10x.github.io/a2a/>. `.github/workflows/pages.yml` builds it and
  `b10x-docs-site.yml` publishes the build of each `main` commit.
- `crates/a2a-docs` generates the site's reference pages: the specification (`ess generate --kind
  docs`), the HTTP+JSON routes served at A2A's path and the reasons for the others, and an index of
  every `UNMAPPED` marker. It owns `website/docs/reference/` whole: `task docs-check`, part of
  `task check`, fails on a missing, stale or hand-edited page there and on any page it does not
  generate, and `task docs-generate` removes a page it no longer writes.
- Site downloads, with `SHA256SUMS`: `a2a.proto`, the OpenAPI document derived from it, the
  specification's two OpenAPI documents (command routes, and A2A's paths through `ess/http.yaml`),
  its AsyncAPI document, and its JSON Schemas as one JSON Schema 2020-12 document. `task
  docs-check` holds each to the repository files it is made from.
- Release asset `a2a-<version>-http.openapi.yaml`, the OpenAPI document of `ess/http.yaml`; the
  specification bundle carries `generated/openapi-http/` too.

## [0.2.0] - 2026-10-22

The specification now covers A2A's JSON-RPC method names and error codes and both response streams,
and resolves 6 of the 12 UNMAPPED markers from `a2a.proto` and the A2A v1.0.1 specification.

### Changed

- Push notification configs are scoped to their parent task. Deleting a config of an unknown task
  answers TaskNotFound; a delete under an existing task is idempotent.
- Views are granted per section 13.1: `Tasks` and `TaskPushNotificationConfigs` to the Client,
  `PushNotificationConfigs` and `AllTasks` to the Agent.
- SendMessage blocking follows `SendMessageConfiguration.blocking` from `a2a.proto`; a second cancel
  answers TaskNotCancelable; streams end on a terminal state.
- Every TaskStatusUpdated payload carries the status. `TaskSnapshot.context_id` and the
  ListTaskPushNotificationConfigs response fields are optional, as `a2a.proto` declares them.
- Refusals decided by the request alone (version, extension, push and streaming capability) answer
  before the task lookup on every command that reads a task.

### Added

- The JSON-RPC method of each operation (its command name) and the JSON-RPC code of each A2A error
  (-32001 to -32009, with InvalidAgentResponse -32006), each beside its table row.
- The MessageStreamResponse and TaskSubscriptionResponse events with the four StreamResponse members,
  published on the `message:stream` and `tasks:subscribe` AsyncAPI channels.
- ListTaskPushNotificationConfigs, UpdateArtifact with TaskArtifactUpdated, and the
  ContextNotAccepted, VersionNotSupported and ExtensionSupportRequired errors.
- 27 new authored conformance scenario files in `ess/scenarios/` (28 in total).

### Fixed

- `cargo run -p a2a-xtask -- drift` reads the repository root at run time, so a binary built in
  another worktree no longer runs `ess` in a directory that is gone.
- The release workflow uploads its assets to the GitHub Release the bot creates, and no longer
  creates the Release itself.

## [0.1.0] - 2026-10-22

ESS specification of A2A v1.0.1 and the artifacts generated from it.

### Added

- ESS specification of A2A v1.0.1 in `ess/`: domains `a2a.tasks` and `a2a.discovery`, 15 commands,
  4 events, 3 views and one component `a2a-agent`.
- Upstream `a2a.proto` pinned at v1.0.1 and an OpenAPI 3.0 document derived from it.
- OpenAPI, AsyncAPI and JSON Schema projections and a synthesized Rust crate under `generated/`,
  held to `ess/` by `task check`.
- Release assets: `a2a.proto`, both OpenAPI documents, the AsyncAPI document, the JSON Schema
  archive and the specification bundle, with `SHA256SUMS`.

[Unreleased]: https://github.com/beyond10x/a2a/compare/0.5.0...HEAD
[0.5.0]: https://github.com/beyond10x/a2a/compare/0.4.0...0.5.0
[0.4.0]: https://github.com/beyond10x/a2a/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/beyond10x/a2a/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/beyond10x/a2a/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/beyond10x/a2a/releases/tag/0.1.0
