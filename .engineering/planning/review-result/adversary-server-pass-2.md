---
format: aep.planning-md/3
id: review-result:adversary-server-pass-2
kind: review-result
status: active
title: Adversary pass 2 on story:server
relations:
- reviews: story:server
revision: 1
---
unit: story:server (adversary pass 2), worktree ~/.local/state/worktree/trees/b10x/a2a/a2a-w3-server at wave/3 6520aef plus its uncommitted diff and untracked crates/a2a-server/ (fix pass 1)
verdict: NEEDS-CHANGE
cases: executed 75→82, red 7
origin: introduced 6 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths: ~/.cache/a2a-gen/wave3/adv-server-2/ (2M logs), ~/.cache/b10x-target/a2a-adv2-plant/ (227M plant build), ~/.cache/b10x-target/a2a/ (the brief's target dir), ~/.cache/a2a-gen/wave3/adversary-server-2.md
needs-coordinator: F6, whether tenant scopes tasks (stored at ports.rs:295, never read; the spec leaves it to the server). F3, whether a connection cap or a request-head deadline is in this unit's scope

## 1. What I touched

`git --no-pager diff --stat=120` (run 17:05):

```
 AGENTS.md                                                                   |    4 +-
 CHANGELOG.md                                                                |   23 +
 Cargo.lock                                                                  |  209 +++++
 Taskfile.yml                                                                |    2 +-
 crates/a2a-xtask/src/main.rs                                                |    3 +
 ess/domains/tasks.yaml                                                      |   15 +
 ess/http.yaml                                                               |   10 +-
 generated/asyncapi/.ess-output/state.json                                   |    2 +-
 generated/asyncapi/asyncapi/a2a-agent.yaml                                  |    8 +-
 generated/openapi-http/.ess-output/state.json                               |    2 +-
 generated/openapi-http/openapi/a2a-agent.yaml                               |   29 +-
 generated/openapi/.ess-output/state.json                                    |    2 +-
 generated/openapi/openapi/a2a-agent.yaml                                    |   29 +-
 generated/rust/a2a/.ess-output/state.json                                   |    2 +-
 generated/rust/a2a/Cargo.toml                                               |    4 +-
 generated/rust/a2a/PLAN.md                                                  |    4 +-
 generated/rust/a2a/plan.json                                                |    4 +-
 generated/rust/a2a/src/actor.rs                                             |    4 +-
 generated/rust/a2a/src/behaviour.rs                                         |    5 +-
 generated/rust/a2a/src/bin/a2a-agent-server.rs                              |    4 +-
 generated/rust/a2a/src/discovery.rs                                         |    4 +-
 generated/rust/a2a/src/json.rs                                              |    4 +-
 generated/rust/a2a/src/lib.rs                                               |    4 +-
 generated/rust/a2a/src/obligation.rs                                        |    4 +-
 generated/rust/a2a/src/ports.rs                                             |    4 +-
 generated/rust/a2a/src/ports/a2a_agent.rs                                   |    4 +-
 generated/rust/a2a/src/primitives.rs                                        |    4 +-
 generated/rust/a2a/src/server.rs                                            |    4 +-
 generated/rust/a2a/src/server/a2a-agent.docs.md                             |   24 +-
 generated/rust/a2a/src/server/a2a-agent.openapi.json                        | 1426 ++++++++++++++++---------------
 generated/rust/a2a/src/server/a2a_agent.rs                                  | 1613 ++++++++++++++++++-----------------
 generated/rust/a2a/src/server/entry.rs                                      |    4 +-
 generated/rust/a2a/src/server/http.rs                                       |  547 +++++++++++-
 generated/rust/a2a/src/server/json.rs                                       |    4 +-
 generated/rust/a2a/src/server/memory.rs                                     |    4 +-
 generated/rust/a2a/src/server/static_assets.rs                              |    4 +-
 generated/rust/a2a/src/server/wire.rs                                       |   22 +-
 generated/rust/a2a/src/system.rs                                            |    4 +-
 generated/rust/a2a/src/tasks.rs                                             |    8 +-
 generated/schema/.ess-output/state.json                                     |    2 +-
 .../schema/schema/commands/a2a.discovery.GetExtendedAgentCard.schema.json   |    2 +-
 generated/schema/schema/commands/a2a.tasks.CancelTask.schema.json           |    4 +-
 generated/schema/schema/commands/a2a.tasks.CompleteTask.schema.json         |    4 +-
 .../schema/commands/a2a.tasks.CreateTaskPushNotificationConfig.schema.json  |    4 +-
 .../schema/commands/a2a.tasks.DeleteTaskPushNotificationConfig.schema.json  |    4 +-
 generated/schema/schema/commands/a2a.tasks.FailTask.schema.json             |    4 +-
 generated/schema/schema/commands/a2a.tasks.GetTask.schema.json              |    4 +-
 .../schema/commands/a2a.tasks.GetTaskPushNotificationConfig.schema.json     |    4 +-
 .../schema/commands/a2a.tasks.ListTaskPushNotificationConfigs.schema.json   |    4 +-
 generated/schema/schema/commands/a2a.tasks.RejectTask.schema.json           |    4 +-
 generated/schema/schema/commands/a2a.tasks.RequireAuth.schema.json          |    4 +-
 generated/schema/schema/commands/a2a.tasks.RequireInput.schema.json         |    4 +-
 generated/schema/schema/commands/a2a.tasks.SendMessage.schema.json          |    4 +-
 generated/schema/schema/commands/a2a.tasks.SendStreamingMessage.schema.json |    4 +-
 generated/schema/schema/commands/a2a.tasks.StartWorking.schema.json         |    4 +-
 generated/schema/schema/commands/a2a.tasks.SubscribeToTask.schema.json      |    4 +-
 generated/schema/schema/commands/a2a.tasks.UpdateArtifact.schema.json       |    4 +-
 .../schema/schema/entities/a2a.tasks.PushNotificationConfig.schema.json     |    4 +-
 generated/schema/schema/entities/a2a.tasks.Task.schema.json                 |    7 +-
 .../schema/errors/a2a.discovery.ExtendedAgentCardNotConfigured.schema.json  |    2 +-
 .../schema/schema/errors/a2a.discovery.ExtensionSupportRequired.schema.json |    2 +-
 .../schema/schema/errors/a2a.discovery.UnsupportedOperation.schema.json     |    2 +-
 .../schema/schema/errors/a2a.discovery.VersionNotSupported.schema.json      |    2 +-
 .../schema/schema/errors/a2a.tasks.ContentTypeNotSupported.schema.json      |    2 +-
 generated/schema/schema/errors/a2a.tasks.ContextMismatch.schema.json        |    2 +-
 generated/schema/schema/errors/a2a.tasks.ContextNotAccepted.schema.json     |    2 +-
 .../schema/schema/errors/a2a.tasks.ExtensionSupportRequired.schema.json     |    2 +-
 generated/schema/schema/errors/a2a.tasks.InvalidAgentResponse.schema.json   |    2 +-
 .../schema/schema/errors/a2a.tasks.PushNotificationNotSupported.schema.json |    2 +-
 generated/schema/schema/errors/a2a.tasks.TaskNotCancelable.schema.json      |    2 +-
 generated/schema/schema/errors/a2a.tasks.TaskNotFound.schema.json           |    2 +-
 generated/schema/schema/errors/a2a.tasks.TaskStateConflict.schema.json      |    2 +-
 generated/schema/schema/errors/a2a.tasks.UnsupportedOperation.schema.json   |    2 +-
 generated/schema/schema/errors/a2a.tasks.VersionNotSupported.schema.json    |    2 +-
 generated/schema/schema/events/a2a.tasks.MessageStreamResponse.schema.json  |    2 +-
 .../schema/events/a2a.tasks.PushNotificationConfigCreated.schema.json       |    2 +-
 .../schema/events/a2a.tasks.PushNotificationConfigDeleted.schema.json       |    2 +-
 generated/schema/schema/events/a2a.tasks.TaskArtifactUpdated.schema.json    |    2 +-
 generated/schema/schema/events/a2a.tasks.TaskCreated.schema.json            |    2 +-
 generated/schema/schema/events/a2a.tasks.TaskStatusUpdated.schema.json      |    2 +-
 .../schema/schema/events/a2a.tasks.TaskSubscriptionResponse.schema.json     |    2 +-
 .../schema/schema/responses/a2a.discovery.GetExtendedAgentCard.schema.json  |    2 +-
 generated/schema/schema/responses/a2a.tasks.CancelTask.schema.json          |    4 +-
 .../schema/responses/a2a.tasks.CreateTaskPushNotificationConfig.schema.json |    4 +-
 generated/schema/schema/responses/a2a.tasks.GetTask.schema.json             |    4 +-
 .../schema/responses/a2a.tasks.GetTaskPushNotificationConfig.schema.json    |    4 +-
 .../schema/responses/a2a.tasks.ListTaskPushNotificationConfigs.schema.json  |    4 +-
 generated/schema/schema/responses/a2a.tasks.SendMessage.schema.json         |    4 +-
 .../schema/schema/responses/a2a.tasks.SendStreamingMessage.schema.json      |    4 +-
 generated/schema/schema/responses/a2a.tasks.SubscribeToTask.schema.json     |    4 +-
 .../schema/schema/types/a2a.discovery.APIKeySecurityScheme.schema.json      |    2 +-
 generated/schema/schema/types/a2a.discovery.AgentCapabilities.schema.json   |    2 +-
 generated/schema/schema/types/a2a.discovery.AgentCard.schema.json           |    2 +-
 generated/schema/schema/types/a2a.discovery.AgentCardSignature.schema.json  |    2 +-
 generated/schema/schema/types/a2a.discovery.AgentExtension.schema.json      |    2 +-
 generated/schema/schema/types/a2a.discovery.AgentInterface.schema.json      |    2 +-
 generated/schema/schema/types/a2a.discovery.AgentProvider.schema.json       |    2 +-
 generated/schema/schema/types/a2a.discovery.AgentSkill.schema.json          |    2 +-
 .../schema/types/a2a.discovery.AuthorizationCodeOAuthFlow.schema.json       |    2 +-
 .../schema/types/a2a.discovery.ClientCredentialsOAuthFlow.schema.json       |    2 +-
 generated/schema/schema/types/a2a.discovery.DeviceCodeOAuthFlow.schema.json |    2 +-
 .../schema/schema/types/a2a.discovery.HTTPAuthSecurityScheme.schema.json    |    2 +-
 generated/schema/schema/types/a2a.discovery.ImplicitOAuthFlow.schema.json   |    2 +-
 .../schema/schema/types/a2a.discovery.MutualTlsSecurityScheme.schema.json   |    2 +-
 .../schema/schema/types/a2a.discovery.OAuth2SecurityScheme.schema.json      |    2 +-
 generated/schema/schema/types/a2a.discovery.OAuthFlows.schema.json          |    2 +-
 .../schema/types/a2a.discovery.OpenIdConnectSecurityScheme.schema.json      |    2 +-
 generated/schema/schema/types/a2a.discovery.PasswordOAuthFlow.schema.json   |    2 +-
 generated/schema/schema/types/a2a.discovery.SecurityRequirement.schema.json |    2 +-
 generated/schema/schema/types/a2a.discovery.SecurityScheme.schema.json      |    2 +-
 generated/schema/schema/types/a2a.discovery.StringList.schema.json          |    2 +-
 generated/schema/schema/types/a2a.tasks.Artifact.schema.json                |    2 +-
 generated/schema/schema/types/a2a.tasks.AuthenticationInfo.schema.json      |    2 +-
 generated/schema/schema/types/a2a.tasks.ContextId.schema.json               |    2 +-
 generated/schema/schema/types/a2a.tasks.Message.schema.json                 |    2 +-
 generated/schema/schema/types/a2a.tasks.Part.schema.json                    |    2 +-
 .../schema/schema/types/a2a.tasks.PushNotificationConfig.State.schema.json  |    2 +-
 .../schema/schema/types/a2a.tasks.PushNotificationConfigId.schema.json      |    2 +-
 .../schema/schema/types/a2a.tasks.PushNotificationConfigRow.schema.json     |    2 +-
 generated/schema/schema/types/a2a.tasks.Role.schema.json                    |    2 +-
 .../schema/schema/types/a2a.tasks.SendMessageConfiguration.schema.json      |    2 +-
 generated/schema/schema/types/a2a.tasks.Task.State.schema.json              |    2 +-
 generated/schema/schema/types/a2a.tasks.TaskArtifactUpdateEvent.schema.json |    2 +-
 generated/schema/schema/types/a2a.tasks.TaskId.schema.json                  |    2 +-
 .../schema/schema/types/a2a.tasks.TaskPushNotificationConfig.schema.json    |    2 +-
 generated/schema/schema/types/a2a.tasks.TaskSnapshot.schema.json            |    2 +-
 generated/schema/schema/types/a2a.tasks.TaskState.schema.json               |    2 +-
 generated/schema/schema/types/a2a.tasks.TaskStatus.schema.json              |    2 +-
 generated/schema/schema/types/a2a.tasks.TaskStatusUpdateEvent.schema.json   |    2 +-
 website/docs/reference/downloads.md                                         |   12 +-
 website/docs/reference/specification/a2a-discovery.md                       |    2 +-
 website/docs/reference/specification/a2a-tasks.md                           |   14 +-
 website/docs/reference/specification/crossings.md                           |    2 +-
 website/docs/reference/specification/index.md                               |    2 +-
 website/docs/reference/specification/interactions.md                        |    2 +-
 website/docs/reference/specification/topology.md                            |    2 +-
 website/docs/reference/unmapped.md                                          |   55 +-
 website/static/downloads/SHA256SUMS                                         |    8 +-
 website/static/downloads/a2a-http.openapi.yaml                              |   29 +-
 website/static/downloads/a2a.asyncapi.yaml                                  |    8 +-
 website/static/downloads/a2a.openapi.yaml                                   |   29 +-
 website/static/downloads/a2a.schema.json                                    |  233 ++---
 142 files changed, 2811 insertions(+), 1879 deletions(-)
```

All 142 rows are the implementor's. My files sit in the untracked `crates/a2a-server/`, which the diff cannot show. Proof of the bound:

| check | at start (16:40) | at end (17:05) |
|---|---|---|
| sha256 of `git --no-pager diff` | e19cae83…a8ff | e19cae83…a8ff |
| sha256 of every file under `crates/a2a-server/` (sorted) | 21 rows | the same 21 rows, plus 3 new: `tests/adv2_{concurrency,exhaustion,paging}.rs` |
| files under the tree newer than the start fingerprint (`find -newer`) | — | only those 3 |

Fingerprints: `adv-server-2/tree-fingerprint-start.txt`, `tree-fingerprint-final.txt`. No non-test path was written.

## 2. Cases added

All are red now. Each was run alone (the test binary with `--exact <name>`) before the suite. The line numbers below are from before `rustfmt`; after it, they are :61, :80, :105, :114, :158, :69 and :143.

| file | case | asserts |
|---|---|---|
| adv2_paging.rs | `adv2_a_list_tasks_page_token_is_not_taken_as_a_list_configs_page_token` | ListConfigs given a ListTasks token answers 400, or the full list |
| adv2_paging.rs | `adv2_a_list_tasks_page_token_the_server_never_issued_is_refused` | `00` gets 400 (control); `0a` must also get 400 |
| adv2_paging.rs | `adv2_a_list_configs_page_token_the_server_never_issued_is_refused` | `zz` gets 400 (control); `%2B0` must also get 400 |
| adv2_exhaustion.rs | `adv2_subscribers_that_send_a_byte_and_hang_up_do_not_take_the_server_down` | under `ulimit -n 64`, after 80 subscribers to a waiting task each send 1 byte and hang up, the card still answers 200 |
| adv2_exhaustion.rs | `adv2_clients_sending_their_heads_slowly_do_not_lock_out_another_client` | under `ulimit -n 64`, with 70 clients sending their heads 1 byte every 500 ms, the card answers within 2 s |
| adv2_concurrency.rs | `adv2_a_client_that_does_not_read_its_answer_does_not_hold_back_another_client` | the card waits less than the time to read the answer in full + 2 s while one 9 MB ListTasks answer goes unread |
| adv2_concurrency.rs | `adv2_a_stream_whose_client_does_not_read_does_not_hold_back_another_client` | the same, while a 7 MB `message:stream` goes unread |

Red output of each case run alone, verbatim (`red-paging.log`, `red-exhaustion.log`, `red-concurrency.log`):

```
panicked at crates/a2a-server/tests/adv2_paging.rs:60:5:
ListConfigs took ListTasks' page token and answered 200 with ["3-cfg", "4-cfg", "5-cfg", "6-cfg"] of ["1-cfg", "2-cfg", "3-cfg", "4-cfg", "5-cfg", "6-cfg"]

panicked at crates/a2a-server/tests/adv2_paging.rs:79:5:
assertion `left == right` failed: ListTasks took the page token `0a` (a lone newline), which it never issues: {"tasks":[],"nextPageToken":"","pageSize":50,"totalSize":2}
  left: 200
 right: 400

panicked at crates/a2a-server/tests/adv2_paging.rs:104:5:
assertion `left == right` failed: ListConfigs took the page token `+0`, which the server never writes: {"configs":[],"nextPageToken":""}
  left: 200
 right: 400

panicked at crates/a2a-server/tests/adv2_exhaustion.rs:114:5:
after 60 subscribers to a waiting task each read the first event, sent one byte and hung up, and 1 s of pruning ticks, the server exited (None) or no longer answers the card (None)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 5.03s

panicked at crates/a2a-server/tests/adv2_exhaustion.rs:155:5:
while 70 clients each sent their head one byte every 500 ms, another client got no agent card within 2 s (None)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 6.02s

panicked at crates/a2a-server/tests/adv2_concurrency.rs:59:5:
a second client waited 30.436858156s for the agent card while one client held a 9002494 byte ListTasks answer unread (the same answer reads in full in 437.182301ms)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 31.90s

panicked at crates/a2a-server/tests/adv2_concurrency.rs:128:5:
a second client waited 20.20322029s for the agent card while one client read none of a 7202531 byte event stream (the same stream reads in full in 274.465068ms)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 20.98s
```

## 3. Suite run, after the cases existed

`cargo test -p a2a-server --locked --no-fail-fast` (`suite.log`). The red suite is the expected outcome:

| binary | result |
|---|---|
| adv2_concurrency | 0/2, FAILED (:143 waited 20.263250293s; :69 waited 31.109183157s) |
| adv2_exhaustion | 0/2, FAILED (:114, :158) |
| adv2_paging | 0/3, FAILED (:61, :80, :105) |
| lib 2, main 0, adv_agent 2, adv_availability 2, adv_binary 3, adv_routes 1, adv_wire 8, binary 3, defines_none 2, obligations 34, routes 7, streaming 11 | all ok (75) |
| total | 82 run, 75 passed, 7 failed, `error: 3 targets failed`, EXIT=101 |

The other checks:

| check | result | log |
|---|---|---|
| `task check` | EXIT=201. It fails only at `cargo test --workspace`, on the adv2 cases. spec, projections-drift, drift, http-paths, docs-check, `cargo fmt --check` and clippy `-D warnings` all passed before that step | task-check.log |
| `cargo test --workspace --locked --no-fail-fast -- --skip adv2_` | TEST_EXIT=0. a2a-docs 18 + adv_docs_site 3, a2a-xtask 13, adv_http_binding 3, adv_http_binding_pass2 2, a2a-server 75 | check-rest.log |
| `cargo build` of `generated/rust/a2a --features server` | BUILD_EXIT=0 | check-rest.log |
| clippy `-p a2a-server --all-targets -D warnings` | EXIT=0 | clippy.log |
| conform-interpreted (its two `ess verify conform` commands, SCRATCH in my dir) | "168 scenarios: 152 passed, 0 failed, 16 error, 0 unsupported", EXIT=201. Same as the fix report | conform.log |

Plants, each in an rsync copy of the tree (`plant-tree`, removed afterwards; target `~/.cache/b10x-target/a2a-adv2-plant`), from `plants.log`:

| plant | check | result |
|---|---|---|
| A: router.rs:870 `<=` changed to `<` | adv_wire | red, `adv_list_tasks_second_page_does_not_repeat_the_first_when_a_task_arrives_between` (adv_wire.rs:50), exit 101 |
| B: a line appended to website/docs/reference/specification/a2a-tasks.md | docs-check | "stale or edited specification/a2a-tasks.md", exit 1 |
| C: a line appended to generated/rust/a2a/src/tasks.rs | drift | "edited: src/tasks.rs", exit 1 |

## 4. Findings

Findings cover the tree named in the header. Every origin is `introduced`, because `crates/a2a-server/` does not exist at 6520aef.

| # | file:line | what was measured | what reaches it | verdict | severity |
|---|---|---|---|---|---|
| F1 | crates/a2a-server/src/router.rs:374 | The router thread writes every plain answer (:374) and every SSE event (`deliver`, :434) synchronously. One unread 9.0 MB ListTasks answer held the card for 30.4 s; one unread 7.2 MB stream held it for 20.2 s (cases above). On the prebuilt binary, an honest reader at 256 KiB/s held the card for 13.98 s (`probe2.log` (b)). lib.rs:61-62 says "a slow client holds only its own connection" | any client reading a multi-MB answer slowly or not at all: ListTasks with `includeArtifacts=true` over large artifacts, or an agent streaming large artifacts. Pass-1 F8 moved off the read side only | NEEDS-CHANGE | warning |
| F2 | crates/a2a-server/src/router.rs:937 | `open` counts a stream as open when its peek returns a byte. A subscriber that sends 1 byte and hangs up keeps the byte ahead of EOF, so `prune` never drops it. 20 such subscribers left fds 24 and CLOSE-WAIT 20, unchanged 5 s later (`probe2.log` (e)). The same 20 hanging up without a byte were pruned (fds 4). Under `ulimit -n 64`, 60 of them stopped the card (case above) | any client that can connect, on purpose. I found no ordinary client that sends bytes after its request. The follower is released only by an event for that task, which may never come for an input-required task. This bypasses the pass-1 F9 fix | NEEDS-CHANGE | warning |
| F3 | crates/a2a-server/src/lib.rs:111 | One thread and one fd per connection, with no cap. `READ_TIMEOUT` (1 s, generated http.rs:48) is per read, so a 1-byte-per-500-ms head is held indefinitely. 200 such clients gave 202 threads and 204 fds, and the card still answered in 0.85 ms at this machine's limit (`probe2.log` (c)). Under `ulimit -n 64`, 70 such clients locked out the card (case above). Everything is released when they stop | any client opening about the process's descriptor limit in connections. I did not run it at a 1024 soft limit | CONFIRMED | note |
| F4 | crates/a2a-server/src/router.rs:973 | `precheck` refuses only ListConfigs tokens that do not decode. A ListTasks token on ListConfigs dropped 1-cfg and 2-cfg (it decodes as a config id, :934 `<= after`). `+0` was taken, because paging.rs:17 `u8::from_str_radix` accepts a sign. router.rs:951-952 says "a page token is one this server issued" | a client that mixes up the two lists' tokens, or forges one. Nothing beyond the unpaged list is exposed | CONFIRMED | note |
| F5 | crates/a2a-server/src/router.rs:928 | `cursor` accepts an empty timestamp, which no issued token has. `0a` gave 200 with an empty page and `totalSize: 2`, while `00` and `zz` get 400 | only a forged token | CONFIRMED | note |
| F6 | crates/a2a-server/src/ports.rs:295 | `tenant` is stored on TaskData but never read. router.rs:786 ignores it on ListTasks; GetTask, Cancel and Subscribe do not check it. A task created under `/acme` was read (200), listed and cancelled (200) under `/beta` (`probe1.log`) | a deployment whose `--card` declares tenants. Per proto a2a.proto:344-350 and spec §13.1, tenant semantics are the server's to define, so whether this is wrong is a decision | CONFIRMED | note |

Not findings:
- `system: true` on the agent actor (ess/domains/tasks.yaml:409) was already at the base (:406 in `git show 6520aef:ess/domains/tasks.yaml`). The fix report lists it among the pass's changes; that is a reporting slip only.

## 5. Attacked and could not break

- Equal-timestamp tie-break: INFEASIBLE. ports.rs:198-218 stamps `max(now_ms, last+1)`, so no two commands share a stamp.
- The cursor's task deleted between pages: INFEASIBLE, because the server has no delete operation.
- pageSize changed between pages: the cursor is a position, so page 2 starts after the cursor whatever the size.
- A task updated between pages moves ahead of the cursor and is skipped. That follows from the spec's order (:262).
- Cancel racing a running agent: INFEASIBLE. One router thread runs both.
- One stamp per command: `next_command` is called at all 3 entry points (router.rs:603, :720, steps.rs:43), and streaming.rs:92 checks that a stored status's stamp equals its event's.
- The blocking SendMessage answer is byte-identical to GetTask of the same task (`probe1.log`).
- SSE order and exactly-once: the existing streaming tests cover them, and they are green.
- A refused agent step publishes nothing, as steps.rs:27-29 documents.
- `unknown: ignore`: unknown body fields on Cancel and CreateConfig get 200, and a body `taskId` that duplicates the path still gets 400 (`probe1.log`).
- Idle connections: 200 of them gave 202 threads, back to 2 within 2.5 s (`probe2.log` (c2)).
- A half-closed subscriber (`ncat`, write side shut) is pruned and sees 1 event (`probe2.log` (d)). I take it as hang-up semantics, and did not raise it.
- An honest slow reader gets the whole answer: 9,002,593 B at 512 KiB every 2 s (`probe3.log`). Nothing is truncated; it only stalls others (F1).
- The 250 ms tick with no followers runs `retain` on an empty list. Follower reconnect was not measured on its own.
- Request ordering per connection: each connection carries one request (`Connection: close`).

## 6. Paths written outside the worktree

- `~/.cache/a2a-gen/wave3/adversary-server-2.md` (this report).
- `~/.cache/a2a-gen/wave3/adv-server-2/`, 2.0M:
  - logs: red-paging.log, red-exhaustion.log, red-concurrency.log, suite.log, task-check.log, check-rest.log, clippy.log, conform.log, plants.log, build-1.log, probe1.log, probe2.log, probe3.log
  - probe output: probe1-configs.txt, probe{1,2,3}-server.{out,err}, probe2-halfclosed.out
  - fingerprints: tree-fingerprint-start.txt, tree-fingerprint-end.txt, tree-fingerprint-final.txt, diff-stat.txt
  - conf/ (suite.json, report.json)
  - The probe scripts (probe1.sh, probe2.sh, probe3.sh, plants.sh) are deleted. The plant copy `plant-tree/` was removed by the script.
- `~/.cache/b10x-target/a2a-adv2-plant/`, 227M: the plant build, including `a2a-xtask-drift/`. It can be deleted.
- `~/.cache/b10x-target/a2a/`, the brief's target dir. It holds the adv2 test binaries, and the runs rewrote `a2a-xtask-drift/`, `a2a-docs/` and `tmp/*.json`, which the repository's own tests and checks write.

```findings
- {"file": "crates/a2a-server/src/router.rs", "line": 374, "category": "concurrency", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the router thread writes plain answers and SSE events synchronously, so one client that reads a multi-MB answer slowly or not at all holds every other client (card delayed 30.4 s, 20.2 s, and 14 s at 256 KiB/s), against lib.rs:61-62"}
- {"file": "crates/a2a-server/src/router.rs", "line": 937, "category": "concurrency", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "open() counts a subscriber that sent one byte and hung up as open, so prune keeps its socket in CLOSE-WAIT until an event for its task arrives, and 60 such subscribers under ulimit -n 64 stop the server answering"}
- {"file": "crates/a2a-server/src/lib.rs", "line": 111, "category": "concurrency", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "connections are uncapped and READ_TIMEOUT bounds each read, not the head, so clients trickling their heads hold a thread and a descriptor each for as long as they like and lock out others at the descriptor limit"}
- {"file": "crates/a2a-server/src/router.rs", "line": 973, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "ListConfigs takes any token that hex-decodes, so a ListTasks token silently skips configs and a signed +0 is accepted, though router.rs:951-952 says a page token is one this server issued"}
- {"file": "crates/a2a-server/src/router.rs", "line": 928, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the ListTasks cursor accepts an empty timestamp no issued token carries, so the forged token 0a answers 200 with an empty page instead of 400"}
- {"file": "crates/a2a-server/src/ports.rs", "line": 295, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "tenant is stored on every task and never read, so a task created under one tenant is read, listed and cancelled under another"}
```
