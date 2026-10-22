# Changelog

All notable changes to this repository are recorded here, in the format of
[Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/). Releases are bare-version tags such as
`0.1.0`; nothing is on a registry. Each GitHub Release carries the upstream `a2a.proto`, the OpenAPI,
AsyncAPI and JSON Schema documents, the specification bundle and a `SHA256SUMS` file. Changes collect
under `[Unreleased]` until the next release.

## [Unreleased]

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

[Unreleased]: https://github.com/beyond10x/a2a/compare/0.1.0...HEAD
[0.1.0]: https://github.com/beyond10x/a2a/releases/tag/0.1.0
