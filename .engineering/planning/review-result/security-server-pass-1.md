---
format: aep.planning-md/3
id: review-result:security-server-pass-1
kind: review-result
status: active
title: Security review pass 1 on story:server
relations:
- reviews: story:server
revision: 1
---
unit: story:server (security-reviewer), uncommitted tree at 6520aef + untracked crates/a2a-server in ~/.local/state/worktree/trees/b10x/a2a/a2a-w3-server
verdict: NEEDS-CHANGE
cases: executed 90→106, red 6
origin: introduced 8 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths (~/.cache/a2a-gen/wave3/sec-server/, ~/.cache/b10x-target/a2a, ~/.cache/b10x-target/a2a-sec-plant, this report) and 1 worktree lease; listed in part 6
needs-coordinator: F7, whether `--extended-card` gets a credential check or is documented as public; F3 and F5, whether a reference server caps followers per task and stored state

Findings cover the working tree above: base 6520aef, the uncommitted diff, and the untracked
`crates/a2a-server/`. Every origin is `introduced`: `git show 6520aef:crates/a2a-server/Cargo.toml`
answers "exists on disk, but not in '6520aef'".

## 1. `git --no-pager diff --stat`

The tracked diff is the unit's own. It is byte-identical to the one present when this review started:
`git --no-pager diff | sha256sum` gave `4f3acc71…b8df8` at the start and at the end. My files are
untracked, under `crates/a2a-server/`, so `--stat` cannot show them. The fingerprint after the stat
proves the bound.

```
 AGENTS.md                                          |    7 +-
 CHANGELOG.md                                       |   40 +
 Cargo.lock                                         |  209 +++
 Taskfile.yml                                       |    2 +-
 crates/a2a-xtask/src/main.rs                       |    3 +
 ess/domains/tasks.yaml                             |   94 +-
 ess/http.yaml                                      |   10 +-
 generated/asyncapi/.ess-output/state.json          |    2 +-
 generated/asyncapi/asyncapi/a2a-agent.yaml         |    8 +-
 generated/openapi-http/.ess-output/state.json      |    2 +-
 generated/openapi-http/openapi/a2a-agent.yaml      |  159 +-
 generated/openapi/.ess-output/state.json           |    2 +-
 generated/openapi/openapi/a2a-agent.yaml           |  155 +-
 generated/rust/a2a/.ess-output/state.json          |    2 +-
 generated/rust/a2a/Cargo.toml                      |    4 +-
 generated/rust/a2a/PLAN.md                         |   18 +-
 generated/rust/a2a/plan.json                       |   18 +-
 generated/rust/a2a/src/actor.rs                    |    4 +-
 generated/rust/a2a/src/behaviour.rs                |    9 +-
 generated/rust/a2a/src/bin/a2a-agent-server.rs     |    4 +-
 generated/rust/a2a/src/discovery.rs                |    4 +-
 generated/rust/a2a/src/json.rs                     |    4 +-
 generated/rust/a2a/src/lib.rs                      |    4 +-
 generated/rust/a2a/src/obligation.rs               |    4 +-
 generated/rust/a2a/src/ports.rs                    |    4 +-
 generated/rust/a2a/src/ports/a2a_agent.rs          |   12 +-
 generated/rust/a2a/src/primitives.rs               |    4 +-
 generated/rust/a2a/src/server.rs                   |    4 +-
 generated/rust/a2a/src/server/a2a-agent.docs.md    |   54 +-
 .../rust/a2a/src/server/a2a-agent.openapi.json     | 1557 ++++++++++--------
 generated/rust/a2a/src/server/a2a_agent.rs         | 1701 ++++++++++----------
 generated/rust/a2a/src/server/entry.rs             |    4 +-
 generated/rust/a2a/src/server/http.rs              |  547 ++++++-
 generated/rust/a2a/src/server/json.rs              |    4 +-
 generated/rust/a2a/src/server/memory.rs            |    6 +-
 generated/rust/a2a/src/server/static_assets.rs     |    4 +-
 generated/rust/a2a/src/server/wire.rs              |   87 +-
 generated/rust/a2a/src/system.rs                   |    4 +-
 generated/rust/a2a/src/tasks.rs                    |   52 +-
 generated/schema/.ess-output/state.json            |    2 +-
 .../a2a.discovery.GetExtendedAgentCard.schema.json |    2 +-
 .../commands/a2a.tasks.CancelTask.schema.json      |    4 +-
 .../commands/a2a.tasks.CompleteTask.schema.json    |    4 +-
 ...ks.CreateTaskPushNotificationConfig.schema.json |    4 +-
 ...ks.DeleteTaskPushNotificationConfig.schema.json |    4 +-
 .../schema/commands/a2a.tasks.FailTask.schema.json |    4 +-
 .../schema/commands/a2a.tasks.GetTask.schema.json  |    4 +-
 ...tasks.GetTaskPushNotificationConfig.schema.json |    4 +-
 ...sks.ListTaskPushNotificationConfigs.schema.json |    4 +-
 .../commands/a2a.tasks.RejectTask.schema.json      |    4 +-
 .../commands/a2a.tasks.RequireAuth.schema.json     |    4 +-
 .../commands/a2a.tasks.RequireInput.schema.json    |    4 +-
 .../commands/a2a.tasks.SendMessage.schema.json     |    4 +-
 .../a2a.tasks.SendStreamingMessage.schema.json     |    4 +-
 .../commands/a2a.tasks.StartWorking.schema.json    |    4 +-
 .../commands/a2a.tasks.SubscribeToTask.schema.json |    4 +-
 .../commands/a2a.tasks.UpdateArtifact.schema.json  |    4 +-
 .../a2a.tasks.PushNotificationConfig.schema.json   |    4 +-
 .../schema/entities/a2a.tasks.Task.schema.json     |    7 +-
 ...very.ExtendedAgentCardNotConfigured.schema.json |    2 +-
 ....discovery.ExtensionSupportRequired.schema.json |    2 +-
 .../a2a.discovery.UnsupportedOperation.schema.json |    2 +-
 .../a2a.discovery.VersionNotSupported.schema.json  |    2 +-
 .../a2a.tasks.ContentTypeNotSupported.schema.json  |    2 +-
 .../errors/a2a.tasks.ContextMismatch.schema.json   |    2 +-
 .../a2a.tasks.ContextNotAccepted.schema.json       |    2 +-
 .../a2a.tasks.ExtensionSupportRequired.schema.json |    2 +-
 .../a2a.tasks.InvalidAgentResponse.schema.json     |    2 +-
 ....tasks.PushNotificationNotSupported.schema.json |    2 +-
 .../errors/a2a.tasks.TaskNotCancelable.schema.json |    2 +-
 .../errors/a2a.tasks.TaskNotFound.schema.json      |    2 +-
 .../errors/a2a.tasks.TaskStateConflict.schema.json |    2 +-
 .../a2a.tasks.UnsupportedOperation.schema.json     |    2 +-
 .../a2a.tasks.VersionNotSupported.schema.json      |    2 +-
 .../a2a.tasks.MessageStreamResponse.schema.json    |    2 +-
 ...tasks.PushNotificationConfigCreated.schema.json |    2 +-
 ...tasks.PushNotificationConfigDeleted.schema.json |    2 +-
 .../a2a.tasks.TaskArtifactUpdated.schema.json      |    2 +-
 .../events/a2a.tasks.TaskCreated.schema.json       |    2 +-
 .../events/a2a.tasks.TaskStatusUpdated.schema.json |    2 +-
 .../a2a.tasks.TaskSubscriptionResponse.schema.json |    2 +-
 .../a2a.discovery.GetExtendedAgentCard.schema.json |    2 +-
 .../responses/a2a.tasks.CancelTask.schema.json     |    4 +-
 ...ks.CreateTaskPushNotificationConfig.schema.json |    4 +-
 .../schema/responses/a2a.tasks.GetTask.schema.json |    4 +-
 ...tasks.GetTaskPushNotificationConfig.schema.json |    4 +-
 ...sks.ListTaskPushNotificationConfigs.schema.json |    4 +-
 .../responses/a2a.tasks.SendMessage.schema.json    |    4 +-
 .../a2a.tasks.SendStreamingMessage.schema.json     |    4 +-
 .../a2a.tasks.SubscribeToTask.schema.json          |    4 +-
 .../a2a.discovery.APIKeySecurityScheme.schema.json |    2 +-
 .../a2a.discovery.AgentCapabilities.schema.json    |    2 +-
 .../types/a2a.discovery.AgentCard.schema.json      |    2 +-
 .../a2a.discovery.AgentCardSignature.schema.json   |    2 +-
 .../types/a2a.discovery.AgentExtension.schema.json |    2 +-
 .../types/a2a.discovery.AgentInterface.schema.json |    2 +-
 .../types/a2a.discovery.AgentProvider.schema.json  |    2 +-
 .../types/a2a.discovery.AgentSkill.schema.json     |    2 +-
 ...iscovery.AuthorizationCodeOAuthFlow.schema.json |    2 +-
 ...iscovery.ClientCredentialsOAuthFlow.schema.json |    2 +-
 .../a2a.discovery.DeviceCodeOAuthFlow.schema.json  |    2 +-
 ...2a.discovery.HTTPAuthSecurityScheme.schema.json |    2 +-
 .../a2a.discovery.ImplicitOAuthFlow.schema.json    |    2 +-
 ...a.discovery.MutualTlsSecurityScheme.schema.json |    2 +-
 .../a2a.discovery.OAuth2SecurityScheme.schema.json |    2 +-
 .../types/a2a.discovery.OAuthFlows.schema.json     |    2 +-
 ...scovery.OpenIdConnectSecurityScheme.schema.json |    2 +-
 .../a2a.discovery.PasswordOAuthFlow.schema.json    |    2 +-
 .../a2a.discovery.SecurityRequirement.schema.json  |    2 +-
 .../types/a2a.discovery.SecurityScheme.schema.json |    2 +-
 .../types/a2a.discovery.StringList.schema.json     |    2 +-
 .../schema/types/a2a.tasks.Artifact.schema.json    |    2 +-
 .../types/a2a.tasks.AuthenticationInfo.schema.json |    2 +-
 .../schema/types/a2a.tasks.ContextId.schema.json   |    2 +-
 .../schema/types/a2a.tasks.Message.schema.json     |    2 +-
 .../schema/schema/types/a2a.tasks.Part.schema.json |    2 +-
 ....tasks.PushNotificationConfig.State.schema.json |    2 +-
 .../a2a.tasks.PushNotificationConfigId.schema.json |    2 +-
 ...a2a.tasks.PushNotificationConfigRow.schema.json |    2 +-
 .../schema/schema/types/a2a.tasks.Role.schema.json |    2 +-
 .../a2a.tasks.SendMessageConfiguration.schema.json |    2 +-
 .../schema/types/a2a.tasks.Task.State.schema.json  |    2 +-
 .../a2a.tasks.TaskArtifactUpdateEvent.schema.json  |    2 +-
 .../schema/types/a2a.tasks.TaskId.schema.json      |    2 +-
 ...2a.tasks.TaskPushNotificationConfig.schema.json |    2 +-
 .../types/a2a.tasks.TaskSnapshot.schema.json       |    2 +-
 .../schema/types/a2a.tasks.TaskState.schema.json   |    2 +-
 .../schema/types/a2a.tasks.TaskStatus.schema.json  |    2 +-
 .../a2a.tasks.TaskStatusUpdateEvent.schema.json    |    2 +-
 website/docs/reference/downloads.md                |   12 +-
 .../docs/reference/specification/a2a-discovery.md  |    2 +-
 website/docs/reference/specification/a2a-tasks.md  |   44 +-
 website/docs/reference/specification/crossings.md  |    2 +-
 website/docs/reference/specification/index.md      |    2 +-
 .../docs/reference/specification/interactions.md   |    2 +-
 website/docs/reference/specification/topology.md   |    2 +-
 website/docs/reference/unmapped.md                 |   58 +-
 website/static/downloads/SHA256SUMS                |    8 +-
 website/static/downloads/a2a-http.openapi.yaml     |  159 +-
 website/static/downloads/a2a.asyncapi.yaml         |    8 +-
 website/static/downloads/a2a.openapi.yaml          |  155 +-
 website/static/downloads/a2a.schema.json           |  233 +--
 142 files changed, 3765 insertions(+), 1969 deletions(-)
```

Proof that I added only `sec_*.rs` files:

- `sha256sum` of every file under `crates/a2a-server/`, taken at the start
  (`fp-start-crate.txt`) and at the end (`fp-end-crate.txt`). The diff between the two lists is
  exactly the four added lines below: no existing file changed, and none was removed.

  ```
  > 87215be7c88db895855890c0c73c3f97f2678e8e5ab3970003685c6433f9db42  crates/a2a-server/tests/sec_binary.rs
  > a0ffd3df8cc723548661edbea6775911d5f8a8e5b2390a9309bfccb07443a1c0  crates/a2a-server/tests/sec_bounds.rs
  > 2a94c0b45bafa5f36ba5250e7de36434e42c12ce01ac9c8b49d959a83fbf4187  crates/a2a-server/tests/sec_input.rs
  > 888f0a7393b402223aa8153370811c0071f19231260112a5424d45de56556c9e  crates/a2a-server/tests/sec_tenancy.rs
  ```

- `find . -path ./.git -prune -o -type f -newer <start marker>` over the whole worktree lists
  exactly those four files.
- All 30 untracked paths are under `crates/a2a-server/`: the unit's 26 and my 4.

No implementation file was touched. Plants ran in a copy (part 2).

## 2. Cases added (16), in the order written

| # | file | case | asserts | now |
|---|---|---|---|---|
| 1 | sec_bounds.rs | `sec_clients_that_keep_sending_after_their_answer_do_not_lock_out_another_client` | Up to 300 clients each get the card and then send 1 byte every 300 ms. A new client 1.5 s later still gets the card. | **red** (F1) |
| 2 | sec_bounds.rs | `sec_subscribers_to_one_task_do_not_lock_out_another_client` | Up to 300 `:subscribe` streams on one waiting task each read their first event. A new client still gets the card. | **red** (F3) |
| 3 | sec_bounds.rs | `sec_listing_one_task_does_not_hold_back_another_client` | After 48 tasks of 1 MB text, `GET /tasks?pageSize=1` runs. A card request sent 100 ms later answers 200 within 1 s. | **red** (F4, F5) |
| 4 | sec_bounds.rs | `sec_reading_one_long_task_does_not_hold_back_another_client` | One task gets 40 messages of 1 MB metadata and 2000 configs of 4 KB. `GET /tasks/<id>?historyLength=1` does not hold the card back 1 s. It prints the memory it measured. | green (red under p15); its numbers back F5 |
| 5 | sec_input.rs | `sec_a_message_nested_within_the_parse_limit_leaves_its_task_readable` | A SendMessage that the parser accepts (nesting ≤ 64), followed by GetTask and ListTasks, answers 200 each time. | **red** (F2) |
| 6 | sec_input.rs | `sec_a_body_nested_past_the_limit_is_refused_and_the_server_survives` | 300,000 nested `[`: answered 400, and the process lives. | green (p11) |
| 7 | sec_input.rs | `sec_a_body_or_head_over_its_limit_is_refused` | The over-limit requests get these answers: body 1 MiB+1 → 413; `Content-Length` 2^64−1 → 413; head > 1 MiB+4096 → 431; 101 headers → 431; chunked → 411. | **red** (F6; p12 and p16 change its entries) |
| 8 | sec_input.rs | `sec_encoded_line_breaks_in_ids_and_tenants_add_no_header` | `%0D%0AX-Injected:%20yes` in ids, tenants, config ids, 404 and 405 paths and page tokens, and a CRLF in a messageId, add no header. | green (p06) |
| 9 | sec_input.rs | `sec_malformed_requests_do_not_end_the_server` | About 60 malformed requests (bad numbers, tokens, percent-escapes, bodies, heads and UTF-8), after which the process lives and the card answers. | green (p07) |
| 10 | sec_tenancy.rs | `sec_tenants_named_like_route_words_or_with_encoded_slashes_stay_apart` | Tenants named `tasks`, `message:send`, `.well-known`, `extendedAgentCard`, `%2F` and `%2E%2E` are isolated from the plain routes. `acme%2Fbeta` is isolated from `acme` and from `acme%252Fbeta`. Each tenant reads its own tasks. | green (p01, p03) |
| 11 | sec_tenancy.rs | `sec_page_tokens_and_totals_stay_within_their_tenant` | `totalSize` is 3 under acme and 1 under beta. acme's token under beta yields only beta's tasks, and under no tenant it yields none. | green (p02, p03) |
| 12 | sec_tenancy.rs | `sec_a_push_config_url_is_never_called` | Configs point at a local listener, and their tasks complete or are canceled. 1.5 s later the listener has had no connection. | green (p08) |
| 13 | sec_tenancy.rs | `sec_the_extended_card_is_not_served_to_a_caller_without_credentials` | `GET /extendedAgentCard` without credentials is not answered 200. | **red** (F7) |
| 14 | sec_binary.rs | `sec_the_binary_listens_on_loopback_by_default` | `--help` names `[default: 127.0.0.1:8080]`. | green (p09) |
| 15 | sec_binary.rs | `sec_a_card_file_that_does_not_decode_is_not_echoed` | Three bad card files are refused, and their content never reaches stdout or stderr. | green (p10) |
| 16 | sec_bounds.rs | `sec_a_request_head_still_arriving_after_the_timeout_is_answered_408` | A head trickled 1 byte every 250 ms under `--request-timeout 1` is answered 408 within 4 s. | green (red under p13c); closes F8 |

Notes on the solo runs:

- **Case 1:** the solo run used `cargo` without `b10x-build-slot`, because the build was already
  done ("Finished … in 0.04s").
- **Case 2:** the first solo run failed for the wrong reason: `common::read_head` panicked on
  `ConnectionReset`. I changed the case to read the status line itself and ran it again; the log
  below is that second run.
- **Case 3:** this solo run measured a 3.0 s wait for the card; the suite run in part 3 measured 3.6 s.
- **Case 7:** after its first solo run I added the 101-headers and chunked legs, then ran it again.
  The log below is that run.
- **Case 4:** the first solo run passed in 14.08 s. I then made the case print its measurements,
  without changing the assertion, and ran it again. The log below is that run.
- **Case 16:** I wrote it after the first suite run (part 3) showed that no case measures the
  deadline. It was run alone and then the suite was run again.

Solo output follows, verbatim. Each case was run from the worktree with
`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_TARGET_DIR=~/.cache/b10x-target/a2a timeout <300|500|580> b10x-build-slot cargo test -p a2a-server --locked --test <file> -- --exact <case> > <log> 2>&1; echo EXIT=$? >> <log>`.
Two runs differ: case 1 ran without `b10x-build-slot`, and case 4 added `--nocapture`, so its log
prints the measurement.
the home-directory prefix is rewritten to `~/`, and nothing else is changed.


#### Case 1: `solo-1-trickle.log`

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/sec_bounds.rs (~/.cache/b10x-target/a2a/debug/deps/sec_bounds-d36973d0ef3d5d2c)

running 1 test
test sec_clients_that_keep_sending_after_their_answer_do_not_lock_out_another_client ... FAILED

failures:

---- sec_clients_that_keep_sending_after_their_answer_do_not_lock_out_another_client stdout ----

thread 'sec_clients_that_keep_sending_after_their_answer_do_not_lock_out_another_client' (650214) panicked at crates/a2a-server/tests/sec_bounds.rs:122:5:
256 clients each got the agent card and then sent one byte every 300 ms; 1.5 s later another client got no agent card (None)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    sec_clients_that_keep_sending_after_their_answer_do_not_lock_out_another_client

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 1.81s

error: test failed, to rerun pass `-p a2a-server --test sec_bounds`
EXIT=101
```

#### Case 2: `solo-2-followers.log`

```
b10x-build-slot: slot 2 of 3, 63G free
   Compiling a2a-server v0.3.0 (~/.local/state/worktree/trees/b10x/a2a/a2a-w3-server/crates/a2a-server)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.28s
     Running tests/sec_bounds.rs (~/.cache/b10x-target/a2a/debug/deps/sec_bounds-d36973d0ef3d5d2c)

running 1 test
test sec_subscribers_to_one_task_do_not_lock_out_another_client ... FAILED

failures:

---- sec_subscribers_to_one_task_do_not_lock_out_another_client stdout ----

thread 'sec_subscribers_to_one_task_do_not_lock_out_another_client' (653271) panicked at crates/a2a-server/tests/sec_bounds.rs:171:5:
256 clients subscribed to one task waiting for input and read its first event; another client then got no agent card (None)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    sec_subscribers_to_one_task_do_not_lock_out_another_client

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.15s

error: test failed, to rerun pass `-p a2a-server --test sec_bounds`
EXIT=101
```

#### Case 3: `solo-3-list.log`

```
b10x-build-slot: slot 2 of 3, 63G free
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/sec_bounds.rs (~/.cache/b10x-target/a2a/debug/deps/sec_bounds-d36973d0ef3d5d2c)

running 1 test
test sec_listing_one_task_does_not_hold_back_another_client ... FAILED

failures:

---- sec_listing_one_task_does_not_hold_back_another_client stdout ----

thread 'sec_listing_one_task_does_not_hold_back_another_client' (654482) panicked at crates/a2a-server/tests/sec_bounds.rs:218:5:
after 48 tasks of a 1 MB message each, resident memory 4936 KiB -> 111324 KiB; `GET /tasks?pageSize=1` answered Some("HTTP/1.1 200 OK") in 3.111617259s, and another client sent 100 ms later waited 2.997083271s for the agent card (Some("HTTP/1.1 200 OK"))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    sec_listing_one_task_does_not_hold_back_another_client

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 9.29s

error: test failed, to rerun pass `-p a2a-server --test sec_bounds`
EXIT=101
```

#### Case 4: `solo-sec_reading_one_long_task_does_not_hold_back_another_client.log`

```
b10x-build-slot: slot 2 of 3, 62G free
   Compiling a2a-server v0.3.0 (~/.local/state/worktree/trees/b10x/a2a/a2a-w3-server/crates/a2a-server)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.34s
     Running tests/sec_bounds.rs (~/.cache/b10x-target/a2a/debug/deps/sec_bounds-d36973d0ef3d5d2c)

running 1 test
one task took 40 of 40 messages of 1 MB metadata and stayed waiting, resident memory 5408 KiB -> 126520 KiB; it took 2000 of 2000 configs with a 4 KB url, -> 126708 KiB; `GET /tasks/<id>?historyLength=1` answered Some("HTTP/1.1 200 OK") in 5.26549ms, and another client sent 100 ms later waited 335.281µs for the agent card (Some("HTTP/1.1 200 OK"))
test sec_reading_one_long_task_does_not_hold_back_another_client ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 19.13s

EXIT=0
```

#### Case 5: `solo-sec_a_message_nested_within_the_parse_limit_leaves_its_task_readable.log`

```
b10x-build-slot: slot 2 of 3, 63G free
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/sec_input.rs (~/.cache/b10x-target/a2a/debug/deps/sec_input-7b0c101d30b72655)

running 1 test
test sec_a_message_nested_within_the_parse_limit_leaves_its_task_readable ... FAILED

failures:

---- sec_a_message_nested_within_the_parse_limit_leaves_its_task_readable stdout ----

thread 'sec_a_message_nested_within_the_parse_limit_leaves_its_task_readable' (660596) panicked at crates/a2a-server/tests/sec_input.rs:101:5:
the deepest metadata SendMessage took was 62 arrays under `metadata.k`; it answered 500 ({"error":{"code":500,"status":"INTERNAL","message":"the generated layer answered no JSON","details":[]}}); then GetTask answered 500 ({"error":{"code":500,"status":"INTERNAL","message":"the generated layer answered no JSON","details":[]}}) and ListTasks 500 ({"error":{"code":500,"status":"INTERNAL","message":"the Tasks view answered no rows","details":[]}})
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    sec_a_message_nested_within_the_parse_limit_leaves_its_task_readable

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p a2a-server --test sec_input`
EXIT=101
```

#### Case 6: `solo-sec_a_body_nested_past_the_limit_is_refused_and_the_server_survives.log`

```
b10x-build-slot: slot 2 of 3, 63G free
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/sec_input.rs (~/.cache/b10x-target/a2a/debug/deps/sec_input-7b0c101d30b72655)

running 1 test
test sec_a_body_nested_past_the_limit_is_refused_and_the_server_survives ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.30s

EXIT=0
```

#### Case 7: `solo-sec_a_body_or_head_over_its_limit_is_refused.log`

```
b10x-build-slot: slot 2 of 3, 63G free
   Compiling a2a-server v0.3.0 (~/.local/state/worktree/trees/b10x/a2a/a2a-w3-server/crates/a2a-server)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.41s
     Running tests/sec_input.rs (~/.cache/b10x-target/a2a/debug/deps/sec_input-7b0c101d30b72655)

running 1 test
test sec_a_body_or_head_over_its_limit_is_refused ... FAILED

failures:

---- sec_a_body_or_head_over_its_limit_is_refused stdout ----

thread 'sec_a_body_or_head_over_its_limit_is_refused' (685560) panicked at crates/a2a-server/tests/sec_input.rs:163:5:
a body of 1 MiB + 1, a Content-Length of 2^64 - 1, a head over 1 MiB + 4096, 101 headers and a chunked body answered ["HTTP/1.1 413 Content Too Large", "HTTP/1.1 413 Content Too Large", "HTTP/1.1 500 Internal Server Error", "HTTP/1.1 500 Internal Server Error", "HTTP/1.1 500 Internal Server Error"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    sec_a_body_or_head_over_its_limit_is_refused

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p a2a-server --test sec_input`
EXIT=101
```

#### Case 8: `solo-sec_encoded_line_breaks_in_ids_and_tenants_add_no_header.log`

```
b10x-build-slot: slot 2 of 3, 63G free
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/sec_input.rs (~/.cache/b10x-target/a2a/debug/deps/sec_input-7b0c101d30b72655)

running 1 test
test sec_encoded_line_breaks_in_ids_and_tenants_add_no_header ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

EXIT=0
```

#### Case 9: `solo-sec_malformed_requests_do_not_end_the_server.log`

```
b10x-build-slot: slot 2 of 3, 63G free
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/sec_input.rs (~/.cache/b10x-target/a2a/debug/deps/sec_input-7b0c101d30b72655)

running 1 test
test sec_malformed_requests_do_not_end_the_server ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.32s

EXIT=0
```

#### Case 10: `solo-sec_tenants_named_like_route_words_or_with_encoded_slashes_stay_apart.log`

```
b10x-build-slot: slot 2 of 3, 63G free
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/sec_tenancy.rs (~/.cache/b10x-target/a2a/debug/deps/sec_tenancy-174762b810b53cad)

running 1 test
test sec_tenants_named_like_route_words_or_with_encoded_slashes_stay_apart ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.01s

EXIT=0
```

#### Case 11: `solo-sec_page_tokens_and_totals_stay_within_their_tenant.log`

```
b10x-build-slot: slot 2 of 3, 63G free
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/sec_tenancy.rs (~/.cache/b10x-target/a2a/debug/deps/sec_tenancy-174762b810b53cad)

running 1 test
test sec_page_tokens_and_totals_stay_within_their_tenant ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

EXIT=0
```

#### Case 12: `solo-sec_a_push_config_url_is_never_called.log`

```
b10x-build-slot: slot 2 of 3, 63G free
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running tests/sec_tenancy.rs (~/.cache/b10x-target/a2a/debug/deps/sec_tenancy-174762b810b53cad)

running 1 test
test sec_a_push_config_url_is_never_called ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.50s

EXIT=0
```

#### Case 13: `solo-sec_the_extended_card_is_not_served_to_a_caller_without_credentials.log`

```
b10x-build-slot: slot 2 of 3, 63G free
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/sec_tenancy.rs (~/.cache/b10x-target/a2a/debug/deps/sec_tenancy-174762b810b53cad)

running 1 test
test sec_the_extended_card_is_not_served_to_a_caller_without_credentials ... FAILED

failures:

---- sec_the_extended_card_is_not_served_to_a_caller_without_credentials stdout ----

thread 'sec_the_extended_card_is_not_served_to_a_caller_without_credentials' (700126) panicked at crates/a2a-server/tests/sec_tenancy.rs:172:5:
GET /extendedAgentCard with no Authorization header answered 200 {"name":"the extended card","description":"The reference agent of beyond10x/a2a:
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    sec_the_extended_card_is_not_served_to_a_caller_without_credentials

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p a2a-server --test sec_tenancy`
EXIT=101
```

#### Case 14: `solo-sec_the_binary_listens_on_loopback_by_default.log`

```
b10x-build-slot: slot 1 of 3, 63G free
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/sec_binary.rs (~/.cache/b10x-target/a2a/debug/deps/sec_binary-02fed6c6df367d28)

running 1 test
test sec_the_binary_listens_on_loopback_by_default ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

EXIT=0
```

#### Case 15: `solo-sec_a_card_file_that_does_not_decode_is_not_echoed.log`

```
b10x-build-slot: slot 1 of 3, 63G free
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/sec_binary.rs (~/.cache/b10x-target/a2a/debug/deps/sec_binary-02fed6c6df367d28)

running 1 test
test sec_a_card_file_that_does_not_decode_is_not_echoed ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

EXIT=0
```

#### Case 16: `solo-sec_a_request_head_still_arriving_after_the_timeout_is_answered_408.log`

```
b10x-build-slot: waiting (all 3 slots busy or under 20G free)
b10x-build-slot: slot 3 of 3, 26G free
   Compiling a2a-server v0.3.0 (~/.local/state/worktree/trees/b10x/a2a/a2a-w3-server/crates/a2a-server)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.46s
     Running tests/sec_bounds.rs (~/.cache/b10x-target/a2a/debug/deps/sec_bounds-d36973d0ef3d5d2c)

running 1 test
test sec_a_request_head_still_arriving_after_the_timeout_is_answered_408 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 2.28s

EXIT=0
```

### Planted defects

For each invariant reported as holding, I planted a defect and ran a case to show that it catches
the defect. The plants ran in a copy, `~/.cache/a2a-gen/wave3/sec-server/plant/tree/`, which holds
`Cargo.toml`, `Cargo.lock`, `crates/`, `generated/rust/a2a`, `upstream/openapi/a2a.openapi.yaml` and
`ess/http-unmapped.tsv` from the worktree. The copy built with
`CARGO_TARGET_DIR=~/.cache/b10x-target/a2a-sec-plant`. Each plant changed one line in the copy, ran
`cargo test -p a2a-server --locked --test <file> [-- --exact <case>]`, and restored the file from
`plant/orig/`. Every run log ends `TREE RESTORED`. Each plant's diff and full output are in
`plant/<plant>.diff` and `plant/<plant>.log`. The worktree was never edited.

| plant | line changed (copy) | case run | result (from the run log) | what the failure said |
|---|---|---|---|---|
| p01 | router.rs:528, tenant segment decoded twice | sec_tenancy route-word case | 0 passed; 1 failed | "a task made under /acme%2Fbeta: GET /acme%252Fbeta/tasks/d496d2f3-… answered 200 {…task…}" |
| p02 | ports.rs:1112, no tenant matches every tenant | sec_tenancy page-token case | 0 passed; 1 failed | acme's token on the plain route: "under no tenant 200 {\"tasks\":[…2 tasks…],…\"totalSize\":4}"; expected no tasks |
| p03 | ports.rs:397, `same_tenant` → `true` | all of sec_tenancy | 1 passed; 3 failed | route-word and page-token cases fail (acme's token under beta returns acme's 3 tasks); the extended-card case was already red |
| p03b | same as p03 | all of tenants.rs (pre-existing) | 0 passed; 4 failed | tenants.rs:85 "GET /acme/tasks/ee309a10-… answers otherwise than GET /acme/tasks/no-such-task" |
| p04 | router.rs:551, query `tenant` kept | all of tenants.rs | 3 passed; 1 failed | tenants.rs:181 `a_plain_route_takes_no_tenant_from_the_request`, left: 200 |
| p05 | router.rs:560, body `tenant` kept | all of tenants.rs | 3 passed; 1 failed | tenants.rs:188, same case, left: 200 |
| p06 | router.rs:516, decoded path in an `X-Path` header | CRLF case | 0 passed; 1 failed | "DELETE /tasks/a%0D%0AX-Injected:%20yes: \"HTTP/1.1 405 …\\r\\nX-Path: /tasks/a\\r\\nX-Injected: yes\\r\\n…\"" |
| p07 | router.rs:854, `pageSize` parsed with `.unwrap()` | malformed-requests case | 0 passed; 1 failed | "the server exited (Some(ExitStatus(unix_wait_status(25856))))": exit code 101, one panic on the router thread ends the process |
| p08 | ports.rs:890, `TcpStream::connect` to the config URL | push-config case | 0 passed; 1 failed | "the server connected to the config's URL: Some(\"127.0.0.1:52250\")" |
| p09 | main.rs:17, default `0.0.0.0:8080` | default-bind case | 0 passed; 1 failed | "the help does not name 127.0.0.1:8080 as the default address" |
| p10 | main.rs:98, card text appended to the error | card-echo case | 0 passed; 1 failed | stderr "…/not-json: at byte 0: expected a value: secret-value-one is not JSON" |
| p11 | json.rs:19, `DEPTH = usize::MAX` | nested-past-limit case | 0 passed; 1 failed | "answered \"<no answer>\"; the server exited: Some(ExitStatus(unix_wait_status(134)))": SIGABRT (signal 6 in the wait status) |
| p12 | http.rs:24 and :31, `MAX_BODY` 2 MiB, `MAX_HEADERS` 1000 | limits case | 0 passed; 1 failed | answers become `[400, 413, 500, 200, 500]` (baseline `[413, 413, 500, 500, 500]`) |
| p13 | connections.rs:250, the request deadline never expires | adv2_exhaustion.rs (pre-existing) | **2 passed; 0 failed** | stays green, so no pre-existing case measures the deadline: F8 |
| p13b | same as p13 | the whole 105-case suite | same 6 red as baseline | no case of the 105 catches it |
| p13c | same as p13 | the new 408 case (16) | 0 passed; 1 failed | "a client sending its head one byte every 250 ms under `--request-timeout 1` got \"\" after 4.091343646s" |
| p14 | connections.rs:23, `BACKLOG = usize::MAX` | followers.rs (pre-existing) | 1 passed; 1 failed | "243 events reached a client that read none while they came" |
| p15 | router.rs:596, 2 s sleep before a GetTask answer | long-task case (4) | 0 passed; 1 failed | "another client sent 100 ms later waited 1.905303168s for the agent card" |
| p16 | http.rs:39, `MAX_HEAD` 4 MiB | limits case | 0 passed; 1 failed | the third answer (head over 1 MiB + 4096) becomes `200 OK` |

The plant runs went to five logs: `plant/run.log` (p01–p15), then `run-2.log` to `run-5.log` (p03b,
p13b, p13c and p16). p13b first stopped at my script's 40G free-space guard (39G free). I lowered
the guard to the brief's 15G and ran it again in `run-3.log`.

## 3. Suite run

Command, from the worktree:

```
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_TARGET_DIR=~/.cache/b10x-target/a2a timeout 590 b10x-build-slot cargo test -p a2a-server --locked --no-fail-fast > ~/.cache/a2a-gen/wave3/sec-server/suite-2.log 2>&1; echo EXIT=$? >> ~/.cache/a2a-gen/wave3/sec-server/suite-2.log
```

Result: 106 executed, 100 passed, 6 failed, all 6 of them added by me, `EXIT=101`. Every case that
existed before passes. The count before was 90, from the implementor's fix-2 header ("a2a-server
executed 82→90"). In this log the targets other than `sec_*` add up to 90.

An earlier run, `suite.log`, had 105 cases (99 passed, the same 6 failed, `EXIT=101`). I wrote case
16 after it. Output of `suite-2.log` follows, verbatim except the home-directory prefix, rewritten to `~/`.

```
b10x-build-slot: slot 3 of 3, 22G free
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.35s
     Running unittests src/lib.rs (~/.cache/b10x-target/a2a/debug/deps/a2a_server-7439e0b443994ec5)

running 4 tests
test paging::tests::a_text_this_server_did_not_issue_names_nothing ... ok
test connections::tests::the_cap_leaves_room_under_the_descriptor_limit ... ok
test paging::tests::a_token_of_one_list_names_nothing_in_another ... ok
test paging::tests::a_token_names_its_key ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running unittests src/main.rs (~/.cache/b10x-target/a2a/debug/deps/a2a_server-7ae573910e460b45)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/adv2_concurrency.rs (~/.cache/b10x-target/a2a/debug/deps/adv2_concurrency-9dbfee32267e81d2)

running 2 tests
test adv2_a_stream_whose_client_does_not_read_does_not_hold_back_another_client ... ok
test adv2_a_client_that_does_not_read_its_answer_does_not_hold_back_another_client ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.18s

     Running tests/adv2_exhaustion.rs (~/.cache/b10x-target/a2a/debug/deps/adv2_exhaustion-af41f4868ac9a713)

running 2 tests
test adv2_subscribers_that_send_a_byte_and_hang_up_do_not_take_the_server_down ... ok
test adv2_clients_sending_their_heads_slowly_do_not_lock_out_another_client ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.07s

     Running tests/adv2_paging.rs (~/.cache/b10x-target/a2a/debug/deps/adv2_paging-8270d4a53648b3ab)

running 3 tests
test adv2_a_list_configs_page_token_the_server_never_issued_is_refused ... ok
test adv2_a_list_tasks_page_token_the_server_never_issued_is_refused ... ok
test adv2_a_list_tasks_page_token_is_not_taken_as_a_list_configs_page_token ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s

     Running tests/adv_agent.rs (~/.cache/b10x-target/a2a/debug/deps/adv_agent-e3935c7018515382)

running 2 tests
test adv_an_agent_cannot_move_a_completed_task_on ... ok
test adv_an_appended_chunk_extends_its_artifact_and_a_replacement_replaces_it ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

     Running tests/adv_availability.rs (~/.cache/b10x-target/a2a/debug/deps/adv_availability-6ac2df0996d9c21e)

running 2 tests
test adv_subscribers_that_hang_up_do_not_take_the_server_down ... ok
test adv_a_slow_client_does_not_hold_back_another_client ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.23s

     Running tests/adv_binary.rs (~/.cache/b10x-target/a2a/debug/deps/adv_binary-09fc008441e0dd2a)

running 3 tests
test adv_the_binary_refuses_a_listen_value_that_is_no_address ... ok
test adv_the_binary_refuses_a_card_that_is_not_json ... ok
test adv_the_binary_refuses_a_card_path_that_does_not_exist ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/adv_routes.rs (~/.cache/b10x-target/a2a/debug/deps/adv_routes-3decb7ec98da78d4)

running 1 test
test adv_tenant_twins_answer_what_their_plain_routes_answer ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s

     Running tests/adv_wire.rs (~/.cache/b10x-target/a2a/debug/deps/adv_wire-b343e920d5c852c0)

running 8 tests
test adv_send_message_ignores_an_unrecognized_body_field ... ok
test adv_subscribe_refuses_post_and_names_get_as_the_proto_binds_it ... ok
test adv_get_task_refuses_a_negative_history_length_as_list_tasks_does ... ok
test adv_get_task_ignores_an_unrecognized_query_field ... ok
test adv_status_timestamps_have_millisecond_precision ... ok
test adv_list_tasks_ignores_an_unrecognized_query_field ... ok
test adv_list_tasks_status_timestamp_after_includes_a_task_stamped_at_that_instant ... ok
test adv_list_tasks_second_page_does_not_repeat_the_first_when_a_task_arrives_between ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s

     Running tests/binary.rs (~/.cache/b10x-target/a2a/debug/deps/binary-055b625a8e4eb317)

running 3 tests
test the_binary_refuses_a_card_that_is_not_an_agent_card ... ok
test the_binary_serves_the_built_in_card_at_the_address_it_bound ... ok
test the_binary_serves_the_card_it_is_given ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/defines_none.rs (~/.cache/b10x-target/a2a/debug/deps/defines_none-278621538a3fa45d)

running 2 tests
test every_visibility_and_qualifier_is_read ... ok
test the_server_declares_no_type_the_generated_crate_exports ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/followers.rs (~/.cache/b10x-target/a2a/debug/deps/followers-a2cc11d9b1383604)

running 2 tests
test a_client_that_keeps_up_gets_every_event_of_a_burst ... ok
test a_client_that_falls_behind_loses_its_stream_and_holds_back_nobody ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.19s

     Running tests/obligations.rs (~/.cache/b10x-target/a2a/debug/deps/obligations-b3e4bb08b15c714d)

running 34 tests
test list_tasks_with_a_page_size_out_of_range_is_invalid_argument ... ok
test errors_are_google_rpc_status_objects ... ok
test extended_agent_card_declared_but_not_configured_is_not_configured ... ok
test get_task_of_an_unknown_task_is_task_not_found ... ok
test extended_agent_card_without_the_capability_is_unsupported_operation ... ok
test extended_agent_card_is_answered_when_declared_and_configured ... ok
test cancel_task_of_an_unknown_task_is_task_not_found ... ok
test send_message_answers_a_direct_message_without_a_task ... ok
test push_notification_config_of_an_unknown_task_is_task_not_found ... ok
test send_message_in_a_version_the_card_does_not_declare_is_version_not_supported ... ok
test send_message_with_a_malformed_body_is_invalid_argument ... ok
test send_message_with_a_context_the_agent_cannot_accept_is_context_not_accepted ... ok
test send_message_with_a_part_of_an_unaccepted_media_type_is_content_type_not_supported ... ok
test send_message_creates_a_completed_task_holding_the_echo_artifact ... ok
test send_message_history_length_zero_answers_no_history ... ok
test send_message_returning_immediately_answers_the_task_before_the_agent_works ... ok
test push_notification_config_without_the_capability_is_push_notification_not_supported ... ok
test send_message_continues_a_task_waiting_for_input ... ok
test get_task_answers_the_task ... ok
test send_message_to_a_terminal_task_is_unsupported_operation ... ok
test send_message_with_another_context_than_its_task_is_context_mismatch ... ok
test send_message_to_an_unknown_task_is_task_not_found ... ok
test send_message_without_a_required_extension_is_extension_support_required ... ok
test cancel_task_cancels_a_task_waiting_for_input ... ok
test send_message_without_a_version_is_0_3_and_the_query_parameter_names_one ... ok
test the_public_agent_card_is_served_at_the_well_known_path ... ok
test cancel_task_the_agent_cannot_cancel_now_is_not_cancelable ... ok
test list_tasks_answers_every_task_newest_first_without_artifacts ... ok
test list_tasks_filters_by_context_and_status_and_includes_artifacts_on_request ... ok
test cancel_task_on_completed_task_is_not_cancelable ... ok
test tenant_routes_answer_as_the_plain_routes_do ... ok
test get_task_history_length_keeps_the_latest_messages ... ok
test push_notification_config_is_created_read_listed_and_deleted ... ok
test list_tasks_pages_with_the_next_page_token ... ok

test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s

     Running tests/routes.rs (~/.cache/b10x-target/a2a/debug/deps/routes-f0f74e95241ece67)

running 7 tests
test the_upstream_document_is_read ... ok
test every_upstream_route_is_served ... ok
test every_hand_route_has_its_unmapped_row_and_every_unmapped_row_is_served ... ok
test every_bound_route_is_a_route_of_the_generated_layer ... ok
test an_unknown_path_is_not_found_and_another_method_is_not_allowed ... ok
test the_generated_layers_own_routes_are_not_exposed ... ok
test every_served_route_answers_and_none_is_a_route_miss ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s

     Running tests/sec_binary.rs (~/.cache/b10x-target/a2a/debug/deps/sec_binary-02fed6c6df367d28)

running 2 tests
test sec_the_binary_listens_on_loopback_by_default ... ok
test sec_a_card_file_that_does_not_decode_is_not_echoed ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/sec_bounds.rs (~/.cache/b10x-target/a2a/debug/deps/sec_bounds-d36973d0ef3d5d2c)

running 5 tests
test sec_subscribers_to_one_task_do_not_lock_out_another_client ... FAILED
test sec_a_request_head_still_arriving_after_the_timeout_is_answered_408 ... ok
test sec_clients_that_keep_sending_after_their_answer_do_not_lock_out_another_client ... FAILED
test sec_listing_one_task_does_not_hold_back_another_client ... FAILED
test sec_reading_one_long_task_does_not_hold_back_another_client ... ok

failures:

---- sec_subscribers_to_one_task_do_not_lock_out_another_client stdout ----

thread 'sec_subscribers_to_one_task_do_not_lock_out_another_client' (1020694) panicked at crates/a2a-server/tests/sec_bounds.rs:171:5:
256 clients subscribed to one task waiting for input and read its first event; another client then got no agent card (None)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- sec_clients_that_keep_sending_after_their_answer_do_not_lock_out_another_client stdout ----

thread 'sec_clients_that_keep_sending_after_their_answer_do_not_lock_out_another_client' (1020691) panicked at crates/a2a-server/tests/sec_bounds.rs:122:5:
256 clients each got the agent card and then sent one byte every 300 ms; 1.5 s later another client got no agent card (None)

---- sec_listing_one_task_does_not_hold_back_another_client stdout ----

thread 'sec_listing_one_task_does_not_hold_back_another_client' (1020692) panicked at crates/a2a-server/tests/sec_bounds.rs:278:5:
after 48 tasks of a 1 MB message each, resident memory 4944 KiB -> 111272 KiB; `GET /tasks?pageSize=1` answered Some("HTTP/1.1 200 OK") in 3.766451175s, and another client sent 100 ms later waited 3.649567656s for the agent card (Some("HTTP/1.1 200 OK"))


failures:
    sec_clients_that_keep_sending_after_their_answer_do_not_lock_out_another_client
    sec_listing_one_task_does_not_hold_back_another_client
    sec_subscribers_to_one_task_do_not_lock_out_another_client

test result: FAILED. 2 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.82s

error: test failed, to rerun pass `-p a2a-server --test sec_bounds`
     Running tests/sec_input.rs (~/.cache/b10x-target/a2a/debug/deps/sec_input-7b0c101d30b72655)

running 5 tests
test sec_a_body_or_head_over_its_limit_is_refused ... FAILED
test sec_encoded_line_breaks_in_ids_and_tenants_add_no_header ... ok
test sec_a_message_nested_within_the_parse_limit_leaves_its_task_readable ... FAILED
test sec_a_body_nested_past_the_limit_is_refused_and_the_server_survives ... ok
test sec_malformed_requests_do_not_end_the_server ... ok

failures:

---- sec_a_body_or_head_over_its_limit_is_refused stdout ----

thread 'sec_a_body_or_head_over_its_limit_is_refused' (1034899) panicked at crates/a2a-server/tests/sec_input.rs:163:5:
a body of 1 MiB + 1, a Content-Length of 2^64 - 1, a head over 1 MiB + 4096, 101 headers and a chunked body answered ["HTTP/1.1 413 Content Too Large", "HTTP/1.1 413 Content Too Large", "HTTP/1.1 500 Internal Server Error", "HTTP/1.1 500 Internal Server Error", "HTTP/1.1 500 Internal Server Error"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- sec_a_message_nested_within_the_parse_limit_leaves_its_task_readable stdout ----

thread 'sec_a_message_nested_within_the_parse_limit_leaves_its_task_readable' (1034900) panicked at crates/a2a-server/tests/sec_input.rs:101:5:
the deepest metadata SendMessage took was 62 arrays under `metadata.k`; it answered 500 ({"error":{"code":500,"status":"INTERNAL","message":"the generated layer answered no JSON","details":[]}}); then GetTask answered 500 ({"error":{"code":500,"status":"INTERNAL","message":"the generated layer answered no JSON","details":[]}}) and ListTasks 500 ({"error":{"code":500,"status":"INTERNAL","message":"the Tasks view answered no rows","details":[]}})


failures:
    sec_a_body_or_head_over_its_limit_is_refused
    sec_a_message_nested_within_the_parse_limit_leaves_its_task_readable

test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s

error: test failed, to rerun pass `-p a2a-server --test sec_input`
     Running tests/sec_tenancy.rs (~/.cache/b10x-target/a2a/debug/deps/sec_tenancy-174762b810b53cad)

running 4 tests
test sec_the_extended_card_is_not_served_to_a_caller_without_credentials ... FAILED
test sec_page_tokens_and_totals_stay_within_their_tenant ... ok
test sec_tenants_named_like_route_words_or_with_encoded_slashes_stay_apart ... ok
test sec_a_push_config_url_is_never_called ... ok

failures:

---- sec_the_extended_card_is_not_served_to_a_caller_without_credentials stdout ----

thread 'sec_the_extended_card_is_not_served_to_a_caller_without_credentials' (1035146) panicked at crates/a2a-server/tests/sec_tenancy.rs:172:5:
GET /extendedAgentCard with no Authorization header answered 200 {"name":"the extended card","description":"The reference agent of beyond10x/a2a:
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    sec_the_extended_card_is_not_served_to_a_caller_without_credentials

test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.97s

error: test failed, to rerun pass `-p a2a-server --test sec_tenancy`
     Running tests/streaming.rs (~/.cache/b10x-target/a2a/debug/deps/streaming-95f2aa097fc91fa8)

running 11 tests
test send_streaming_message_answered_directly_is_one_message_event ... ok
test send_streaming_message_errors_answer_before_the_stream_opens ... ok
test subscribe_to_an_unknown_task_is_task_not_found ... ok
test send_streaming_message_without_the_capability_is_unsupported_operation ... ok
test send_streaming_message_closes_at_an_interrupted_state ... ok
test subscribe_to_a_terminal_task_is_unsupported_operation ... ok
test subscribe_without_the_capability_is_unsupported_operation ... ok
test send_streaming_message_streams_the_task_then_its_updates_until_it_completes ... ok
test a_streamed_status_carries_the_timestamp_the_task_then_shows ... ok
test subscribe_to_task_sees_a_cancel ... ok
test subscribe_to_task_streams_the_task_then_what_another_request_does_until_it_completes ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s

     Running tests/tenants.rs (~/.cache/b10x-target/a2a/debug/deps/tenants-a1c86cae37e3e8dd)

running 4 tests
test a_plain_route_takes_no_tenant_from_the_request ... ok
test a_task_of_a_tenant_answers_on_a_plain_route_as_an_unknown_id ... ok
test a_task_made_on_a_plain_route_answers_under_a_tenant_as_an_unknown_id ... ok
test a_task_answers_under_another_tenant_as_an_unknown_id ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

   Doc-tests a2a_server

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 3 targets failed:
    `-p a2a-server --test sec_bounds`
    `-p a2a-server --test sec_input`
    `-p a2a-server --test sec_tenancy`
EXIT=101
```

## 4. Findings

There is no authentication anywhere in the server (F7), so "reaches it" below means any TCP client
that can connect to `--listen`. The default is `127.0.0.1:8080` (main.rs:17), so local processes
only. With `--listen 0.0.0.0:…` it is any host on the network. Every origin is `introduced`: the
crate is new in this unit. Timings come from a debug build.

| # | file:line | verdict | severity | category | finding |
|---|---|---|---|---|---|
| F1 | crates/a2a-server/src/connections.rs:348 | NEEDS-CHANGE | blocker | boundary | Clients that keep sending after their answer hold every seat for as long as they send |
| F2 | crates/a2a-server/src/router.rs:681 | NEEDS-CHANGE | blocker | integrity | One message nested 62 deep leaves its task, and its tenant's ListTasks, answering 500 |
| F3 | crates/a2a-server/src/connections.rs:219 | CONFIRMED | warning | boundary | 256 subscribers to one task fill the cap; nothing limits followers per task or per client |
| F4 | crates/a2a-server/src/router.rs:828 | CONFIRMED | warning | boundary | ListTasks builds every row before paging, on the router thread, and holds back every other client |
| F5 | crates/a2a-server/src/ports.rs:362 | CONFIRMED | warning | boundary | Tasks, history and push configs are kept without limit |
| F6 | crates/a2a-server/src/router.rs:318 | CONFIRMED | warning | contract-drift | The generated 431 and 411 refusals are answered as 500 INTERNAL |
| F7 | crates/a2a-server/src/router.rs:638 | CONFIRMED | warning | contract-drift | The extended card is served to a caller without credentials |
| F8 | crates/a2a-server/src/connections.rs:250 | CONFIRMED | note | mutant | No pre-existing case measures the request deadline; closed by case 16 |

**F1: connections.rs:348.** NEEDS-CHANGE, blocker, introduced.
- `listen` treats `Ok(_)` as "keep reading". It returns on a timeout only once `done` is set, so a
  client that sends one byte per read timeout after its answer keeps its connection open for as long as it sends (read in the code; measured for 1.5 s).
- `admit` (:205) evicts at the cap only connections still reading a request, so `?` at :219
  refuses every new client.
- Measured (case 1, suite-2): "256 clients each got the agent card and then sent one byte every
  300 ms; 1.5 s later another client got no agent card (None)".
- Reaches: one client with 256 sockets. The request deadline (`watch`, :250) does not cover this
  phase.
- This breaks coordinator decision 3 in `brief-server-fix-2.md`, "no client's slow or non-reading
  socket may delay another client".
- Correction to name: in `listen`, return on `Ok(_)` once `done` is set, or bound the post-answer
  read in time and bytes as `http::linger` does.

**F2: router.rs:681.** NEEDS-CHANGE, blocker, introduced.
- `reply` re-parses the generated envelope with `json::parse` (`DEPTH = 64`, json.rs:19). Intake
  accepts a message nested up to the same limit, and storing it inside task → history → message
  adds levels.
- Measured (case 5, suite-2): "the deepest metadata SendMessage took was 62 arrays under
  `metadata.k`". SendMessage answered 500 "the generated layer answered no JSON". The task was
  stored all the same: GetTask answered 500, not 404, and ListTasks answered 500 "the Tasks view
  answered no rows" (the view's re-parse, router.rs:913-921).
- Reaches: one request by any client. ListTasks fails for every caller of that tenant, or of the
  plain route if the task has no tenant. "For good" is read, not run: I found no operation that
  removes a task (see F5). Other tenants' lists do not hold the task, so they are not affected.
- Correction to name: refuse at intake a message that leaves no depth for its envelope, or stop
  re-parsing generated output under the intake limit.

**F3: connections.rs:219.** CONFIRMED, warning, introduced.
- Measured (case 2, suite-2): "256 clients subscribed to one task waiting for input and read its
  first event; another client then got no agent card (None)".
- Long-lived streams are legitimate, and decision 2 says to refuse over the cap. Even so, one
  client can take every seat with `:subscribe` on a single task, because nothing limits followers
  per task or per client.
- needs-coordinator: whether a reference server bounds this.

**F4: router.rs:828.** CONFIRMED, warning, introduced.
- `list` serializes and re-parses every task of the tenant (Tasks view rows, :913-921) before it
  pages. It runs on the single router thread.
- Measured (case 3, suite-2): "after 48 tasks of a 1 MB message each, resident memory 4944 KiB ->
  111272 KiB; `GET /tasks?pageSize=1` answered Some("HTTP/1.1 200 OK") in 3.766451175s, and
  another client sent 100 ms later waited 3.649567656s". The solo run measured 3.11 s and 3.00 s.
- Reaches: any client, by storing tasks (F5) and then listing with `pageSize=1`. The block grows
  with stored bytes. A release build would be faster; I did not measure one.

**F5: ports.rs:362.** CONFIRMED, warning, introduced.
- `record` appends to `history` without limit, `store_config` (:369) adds configs without limit,
  and no operation removes a task. The 1 MiB body limit caps one request, not the total.
- Measured (case 4, solo log): "one task took 40 of 40 messages of 1 MB metadata and stayed
  waiting, resident memory 5408 KiB -> 126520 KiB; it took 2000 of 2000 configs with a 4 KB url,
  -> 126708 KiB".
- Reaches: any client, until the process runs out of memory.
- needs-coordinator: whether a reference server caps stored state.

**F6: router.rs:318.** CONFIRMED, warning, introduced.
- `refusal_error` maps 400, 403, 404, 405, 413 and 501. Every other status the generated layer
  refuses with becomes `500 INTERNAL`, among them the 431 that `http::head_too_large` documents as
  "byte for byte what Go's net/http answers" (generated http.rs:314-317) and the 411 for a chunked
  body.
- Measured (case 7, suite-2): answers `["413", "413", "500", "500", "500"]` against the expected
  `[413, 413, 431, 431, 411]`.
- Reaches: any client. It tells a client the server failed when the client's request was at fault.
- Correction to name: pass 411 and 431 through, or answer `head_too_large` verbatim.

**F7: router.rs:638.** CONFIRMED, warning, introduced.
- `caller` gives every request a DiscoveryClient or TasksClient actor, and nothing checks a
  credential.
- Measured (case 13, suite-2): "GET /extendedAgentCard with no Authorization header answered 200
  {\"name\":\"the extended card\",…".
- The spec says otherwise: `ess/domains/discovery.yaml:368` "An authenticated caller of the
  agent's A2A interface", and `specification.md:406` "after the client has authenticated".
- Reaches: any client, whenever `--extended-card` is given.
- needs-coordinator: whether to add a credential check, or to document the card as public in
  AGENTS.md and the card.

**F8: connections.rs:250.** CONFIRMED, note, introduced.
- Plant p13 makes the request deadline never expire. `adv2_exhaustion.rs` stayed green (2 passed),
  and so did all 105 cases of the earlier suite, apart from the 6 already red.
- The deadline itself works: case 16 is green, and under p13c it is red ("got \"\" after
  4.091343646s").
- Closed by case 16; listed so the coverage gap is on record.

## 5. Reviewed, could not fault

Each "plant" below names the defect from part 2 that turned the case red.

| invariant | evidence | plant |
|---|---|---|
| Tenant isolation, GetTask, Cancel, Subscribe, push configs | tenants.rs (4 cases), case 10 | p03, p03b |
| Tenant segment decoded once; `%2F`, `%252F`, `%2E%2E`, route-word tenants stay apart | case 10 | p01 |
| Plain routes take no tenant from query or body | tenants.rs:181, :188 | p04, p05 |
| ListTasks rows and `totalSize` per tenant | case 11 (acme 3, beta 1) | p02, p03 |
| Page tokens, across tenants | case 11: acme's token answers beta's own tasks under beta and nothing on the plain route. A well-formed forged token is taken as a position, as decision 5 allows ("An HMAC is not required"); it reveals nothing beyond what the holder was already served | p02, p03 |
| No header injection from ids, tenants, config ids, page tokens or messageId | case 8 | p06 |
| No panic from about 60 malformed requests; a router panic would end the process (exit 101) | case 9 | p07 |
| Push config URLs are never connected to (no SSRF) | case 12 | p08 |
| Default bind is loopback | case 14, main.rs:17 | p09 |
| A card file is not echoed on error; no secret in logs (the only output is main.rs:53 error and :76 listening line) | case 15 | p10 |
| JSON nesting past 64 is refused with 400, not a crash | case 6 | p11 |
| Body 1 MiB and `Content-Length` 2^64−1 → 413; 101 headers and head over 1 MiB + 4096 refused (as 500, F6) | case 7 | p12, p16 |
| Writer queue bounded; a non-reading follower is dropped | followers.rs | p14 |
| GetTask with a long history does not hold back other clients | case 4 | p15 |
| Request-head deadline answers 408 | case 16 | p13c |
| Empty tenant segment (`//tasks`) matches no route | router.rs:146, read, not run | none |
| A tenant named `tasks` cannot list: `GET /tasks/tasks` is plain GetTask | router.rs `fit` :132, read, not run | none |

Not measured: a panic on a reader or writer thread.

Accepted, so not raised: `:subscribe` is GET only, push delivery is not implemented, and the ESS
gaps the brief lists.

## 6. Paths written outside the worktree

| path | what |
|---|---|
| `~/.cache/a2a-gen/wave3/sec-server/` | fingerprints (`fp-*`), `diff-stat.txt`, `build-1.log`, `solo-*.log`, `suite.log`, `suite-2.log`, `plant/` (tree copy, `orig/`, plant diffs and logs, `run*.log`), `report-parts/` |
| `~/.cache/b10x-target/a2a` | build output (shared CARGO_TARGET_DIR), and `tmp/sec-binary-cards/` created by case 15 |
| `~/.cache/b10x-target/a2a-sec-plant` | build output of the plant copy (331M), and `tmp/sec-binary-cards/`; left in place, being a target directory |
| `~/.cache/a2a-gen/wave3/security-server.md` | this report |
| worktree lease | session `a2a-w3-sec-server-a6ad8401` on the worktree, released on return |
| `~/.cache/claude-tmp/` | output files of background commands, written by the harness |

The plant scripts were written to `plant/` and deleted after use. In the worktree I added only
the four `crates/a2a-server/tests/sec_*.rs` files.

## 7. Findings block

```findings
- {file: "crates/a2a-server/src/connections.rs", line: 348, category: "boundary", severity: "blocker", verdict: "NEEDS-CHANGE", origin: "introduced", message: "listen keeps a connection open while its client sends one byte per read timeout after its answer; admit evicts only reading connections (:219), so 256 such clients lock out every other client (case sec_clients_that_keep_sending_after_their_answer_do_not_lock_out_another_client: another client got no agent card). Breaks fix-2 decision 3."}
- {file: "crates/a2a-server/src/router.rs", line: 681, category: "integrity", severity: "blocker", verdict: "NEEDS-CHANGE", origin: "introduced", message: "reply re-parses the generated envelope under the intake depth limit 64; a SendMessage with metadata nested 62 deep is stored but answers 500, and GetTask on it and ListTasks for its tenant then answer 500 (case sec_a_message_nested_within_the_parse_limit_leaves_its_task_readable)."}
- {file: "crates/a2a-server/src/connections.rs", line: 219, category: "boundary", severity: "warning", verdict: "CONFIRMED", origin: "introduced", message: "256 subscribers to one task fill the connection cap and another client gets no agent card; nothing limits followers per task or client (case sec_subscribers_to_one_task_do_not_lock_out_another_client). Coordinator: whether to bound."}
- {file: "crates/a2a-server/src/router.rs", line: 828, category: "boundary", severity: "warning", verdict: "CONFIRMED", origin: "introduced", message: "ListTasks builds every row of the tenant before paging, on the router thread: after 48 tasks of 1 MB, GET /tasks?pageSize=1 took 3.77 s and held another client 3.65 s, debug build (case sec_listing_one_task_does_not_hold_back_another_client)."}
- {file: "crates/a2a-server/src/ports.rs", line: 362, category: "boundary", severity: "warning", verdict: "CONFIRMED", origin: "introduced", message: "Tasks, history (record) and push configs (store_config :369) are kept without limit: one task took 40 messages of 1 MB, resident 5408 -> 126520 KiB, and 2000 configs (case sec_reading_one_long_task_does_not_hold_back_another_client). Coordinator: whether to cap."}
- {file: "crates/a2a-server/src/router.rs", line: 318, category: "contract-drift", severity: "warning", verdict: "CONFIRMED", origin: "introduced", message: "refusal_error turns the generated 431 (head too large, 101 headers) and 411 (chunked body) into 500 INTERNAL (case sec_a_body_or_head_over_its_limit_is_refused: 413, 413, 500, 500, 500)."}
- {file: "crates/a2a-server/src/router.rs", line: 638, category: "contract-drift", severity: "warning", verdict: "CONFIRMED", origin: "introduced", message: "No credential is checked; GET /extendedAgentCard without Authorization answers 200, while ess/domains/discovery.yaml:368 and specification.md:406 require an authenticated caller (case sec_the_extended_card_is_not_served_to_a_caller_without_credentials). Coordinator: check or document as public."}
- {file: "crates/a2a-server/src/connections.rs", line: 250, category: "mutant", severity: "note", verdict: "CONFIRMED", origin: "introduced", message: "With the request deadline planted to never expire, every pre-existing case stays green; closed by the added case sec_a_request_head_still_arriving_after_the_timeout_is_answered_408."}
```
