# a2a

The [A2A (Agent2Agent) protocol](https://github.com/a2aproject/A2A) v1.0.1 as an executable
specification in [ESS](https://github.com/beyond10x/ess), and the Rust that is generated from it.

| Path | Contents |
|---|---|
| `ess/` | The specification: task lifecycle, messages, push-notification configs, the extended Agent Card |
| `upstream/` | The pinned upstream `a2a.proto`, its licence, and an OpenAPI 3.0 document derived from it |
| `generated/` | OpenAPI, AsyncAPI and JSON Schema projections, and the synthesized Rust crate |
| `crates/` | Hand-written Rust: repository checks and the documentation generator today; server, client and test kit to follow |
| `website/` | The documentation site (Docusaurus) |

```console
task check
```

validates the specification and fails when anything generated from it has drifted. It also runs
`task conform`: the conformance suite ESS synthesizes from `ess/`, run against the server's own
implementation in process (`crates/a2a-conformance`), fails on any failed or error scenario and on
any `unsupported` one not listed with its reason in `crates/a2a-conformance/unsupported.tsv`.

`cargo run -p a2a-server -- --listen 127.0.0.1:8080` serves an echo agent over A2A's HTTP+JSON
binding. An extended Agent Card given with `--extended-card` is served only to requests carrying
`Authorization: Bearer <token>`, the token read from `--extended-card-token-file`, which
`--extended-card` requires.

## Documentation

The documentation site is <https://beyond10x.github.io/a2a/>. It has the specification's pages,
which of A2A's HTTP+JSON routes are served at their path, every `UNMAPPED` marker, and downloads of
`a2a.proto` and the OpenAPI, AsyncAPI and JSON Schema documents with their SHA-256 sums. Its
reference pages and downloads are generated from this repository and held to it by `task check`.

Agent instructions are in [AGENTS.md](AGENTS.md); changes are listed in [CHANGELOG.md](CHANGELOG.md).

Licensed under Apache-2.0. `upstream/` is © The A2A Project Authors under its own Apache-2.0
licence (`upstream/LICENSE`).
