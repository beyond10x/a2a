# Changelog

All notable changes to this repository are recorded here, in the format of
[Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). Releases are bare-version tags such as
`0.1.0`; nothing is on a registry. Each GitHub Release carries the upstream `a2a.proto`, the OpenAPI,
AsyncAPI and JSON Schema documents, the specification bundle and a `SHA256SUMS` file. Changes collect
under `[Unreleased]` until the next release.

## [Unreleased]

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

[Unreleased]: https://github.com/beyond10x/a2a/compare/0.2.0...HEAD
[0.2.0]: https://github.com/beyond10x/a2a/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/beyond10x/a2a/releases/tag/0.1.0
