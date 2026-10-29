---
format: aep.planning-md/3
id: review-result:adversary-server-pass-1-report
kind: review-result
status: active
title: Adversary pass 1 on story:server
relations:
- reviews: story:server
revision: 1
---
unit: story:server (adversary), worktree ~/.local/state/worktree/trees/b10x/a2a/a2a-w3-server at wave/3 6520aef plus its uncommitted diff and untracked crates/a2a-server/
verdict: NEEDS-CHANGE
cases: executed 55→71, red 9
origin: introduced 12 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths: ~/.cache/a2a-gen/wave3/adv-server/ (185M, mostly mut-target/), ~/.cache/a2a-gen/wave3/adversary-server.md, ~/.cache/b10x-target/a2a/tmp/adv-not-json-card.json
needs-coordinator: task check, clippy on the adv files and a cargo-built suite run were not run because / had 35-39G free, under the 40G floor. Also decide whether :subscribe answers POST too: spec text says POST, proto and openapi say GET

## 1. What I touched

`git --no-pager diff --stat` (run 15:41):

```
 AGENTS.md                                          |    2 +
 CHANGELOG.md                                       |   15 +
 Cargo.lock                                         |  209 +++
 Taskfile.yml                                       |    2 +-
 crates/a2a-xtask/src/main.rs                       |    3 +
 generated/rust/a2a/.ess-output/state.json          |    2 +-
 .../rust/a2a/src/server/a2a-agent.openapi.json     | 1395 +++++++++--------
 generated/rust/a2a/src/server/a2a_agent.rs         | 1607 ++++++++++----------
 generated/rust/a2a/src/server/http.rs              |  543 ++++++-
 9 files changed, 2310 insertions(+), 1468 deletions(-)
```

All nine rows are the implementor's tracked edits. My files sit in the untracked `crates/a2a-server/`, which the diff cannot show. Proof of the bound:

- My only files are `crates/a2a-server/tests/adv_{wire,availability,agent,binary,routes}.rs`.
- Every other file under `crates/a2a-server/` was last modified at or before 14:17:42 (`router.rs`). My first file is from 14:48.
- `tests/common/mod.rs` was last modified at 13:25:50 and is byte-equal to my 14:xx scratch copy.

Two disclosures:

- I ran `rustfmt` on my own adv files after the case runs. It changed only line breaks in `adv_wire.rs` and `adv_availability.rs`; the diffs it applied are in `~/.cache/a2a-gen/wave3/adv-server/fmt-all.log`. The line numbers in the quoted panics refer to the layout before formatting.
- One scratch build started at 39G free, below the 40G floor: the first defines_none mutant in `mut-target/`. It failed to compile within seconds. After that I ran no build.

## 2. Cases added

Each red output below is from a run of that case alone (`run-case.sh`, `cargo test -p a2a-server --test <file> <case> -- --exact`). The excerpts drop blank lines and the `failures:` list. The full logs are `~/.cache/a2a-gen/wave3/adv-server/case-<name>.log`.

| file | case | asserts | now |
|---|---|---|---|
| adv_wire.rs | adv_list_tasks_status_timestamp_after_includes_a_task_stamped_at_that_instant | `statusTimestampAfter` = a task's own stamp lists it (proto :694-696 "greater than or equal") | red |
| adv_wire.rs | adv_list_tasks_second_page_does_not_repeat_the_first_when_a_task_arrives_between | page 2 does not repeat page 1 when a task is created between them (spec :254 cursor, :258 not offset) | red |
| adv_wire.rs | adv_status_timestamps_have_millisecond_precision | `status.timestamp` has 3 fractional digits (spec :1236) | red |
| adv_wire.rs | adv_subscribe_answers_the_post_route_the_specification_text_names | `POST /tasks/{id}:subscribe` streams (spec :1169, :2795) | red |
| adv_wire.rs | adv_get_task_refuses_a_negative_history_length_as_list_tasks_does | GetTask `historyLength=-1` is 400, as ListTasks answers (spec :467) | red |
| adv_wire.rs | adv_list_tasks_ignores_an_unrecognized_query_field | `GET /tasks?futureField=1` is 200 (spec :1275) | red |
| adv_wire.rs | adv_get_task_ignores_an_unrecognized_query_field | `GET /tasks/{id}?futureField=1` is 200 | red |
| adv_wire.rs | adv_send_message_ignores_an_unrecognized_body_field | an extra body field is ignored | green |
| adv_availability.rs | adv_a_slow_client_does_not_hold_back_another_client | a second client gets the card within 2 s while another client sends 1 byte every 500 ms | red |
| adv_availability.rs | adv_subscribers_that_hang_up_do_not_take_the_server_down | after abandoned subscribe streams to an input-required task, the binary (under `ulimit -n 64`) still answers | red |
| adv_agent.rs | adv_an_agent_cannot_move_a_completed_task_on | steps after Complete are dropped | green |
| adv_agent.rs | adv_an_appended_chunk_extends_its_artifact_and_a_replacement_replaces_it | append and replace semantics | green |
| adv_binary.rs | 3 cases: bad `--listen`, missing `--card` path, non-JSON card | exits non-zero and names the input | green |
| adv_routes.rs | adv_tenant_twins_answer_what_their_plain_routes_answer | `/acme/...` twins of 5 GET routes return the same status and body | green; kills a mutant the 55 let through (§4 F11) |

```
=== case-ts-after.log
test adv_list_tasks_status_timestamp_after_includes_a_task_stamped_at_that_instant ... FAILED
thread 'adv_list_tasks_status_timestamp_after_includes_a_task_stamped_at_that_instant' (281942) panicked at crates/a2a-server/tests/adv_wire.rs:30:5:
statusTimestampAfter=2026-10-22T12:48:58.863714561Z (the task's own status timestamp) left the task out: {"tasks":[],"nextPageToken":"","pageSize":50,"totalSize":0}
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
EXIT=101
=== case-paging.log
test adv_list_tasks_second_page_does_not_repeat_the_first_when_a_task_arrives_between ... FAILED
thread 'adv_list_tasks_second_page_does_not_repeat_the_first_when_a_task_arrives_between' (281981) panicked at crates/a2a-server/tests/adv_wire.rs:54:5:
assertion `left == right` failed: page 2 (token `1`) after one new task: {"tasks":[{"id":"62d8a305-4f93-448d-9cb0-a31b146a87f8","contextId":"ed08350f-03f6-4bb7-964e-e66a4ad65612","status":{"state":"TASK_STATE_COMPLETED","timestamp":"2026-10-22T12:48:58.906091264Z"},"history":[{"messageId":"68b668c9-cc3b-4b9b-919a-7e76f5dd46e2","contextId":"ed08350f-03f6-4bb7-964e-e66a4ad65612","taskId":"62d8a305-4f93-448d-9cb0-a31b146a87f8","role":"ROLE_USER","parts":[{"text":"hello"}]}]}],"nextPageToken":"2","pageSize":1,"totalSize":3}
  left: ["62d8a305-4f93-448d-9cb0-a31b146a87f8"]
 right: ["616265a1-807b-4040-988e-aaaa30dd03c1"]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
EXIT=101
=== case-millis.log
test adv_status_timestamps_have_millisecond_precision ... FAILED
thread 'adv_status_timestamps_have_millisecond_precision' (282951) panicked at crates/a2a-server/tests/adv_wire.rs:74:5:
assertion `left == right` failed: `2026-10-22T12:49:03.122057181Z` is not YYYY-MM-DDTHH:mm:ss.sssZ
  left: 9
 right: 3
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
EXIT=101
=== case-post-subscribe.log
test adv_subscribe_answers_the_post_route_the_specification_text_names ... FAILED
thread 'adv_subscribe_answers_the_post_route_the_specification_text_names' (282984) panicked at crates/a2a-server/tests/adv_wire.rs:102:5:
assertion `left == right` failed: POST /tasks/{id}:subscribe: [("Content-Type", "application/a2a+json"), ("Content-Length", "140"), ("Allow", "GET"), ("Connection", "close")]
  left: (405, "application/a2a+json")
 right: (200, "text/event-stream")
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
EXIT=101
=== case-neg-history.log
test adv_get_task_refuses_a_negative_history_length_as_list_tasks_does ... FAILED
thread 'adv_get_task_refuses_a_negative_history_length_as_list_tasks_does' (380903) panicked at crates/a2a-server/tests/adv_wire.rs:118:5:
assertion `left == right` failed: ListTasks refuses historyLength=-1 but GetTask answers: {"id":"6c9f746d-4b89-4079-afcd-4176f5218108","contextId":"31b60c79-757a-4f03-81ff-62e772a099ef","status":{"state":"TASK_STATE_COMPLETED","timestamp":"2026-10-22T12:56:52.735459760Z"},"artifacts":[{"artifactId":"0049db76-7e0d-4d61-a0f2-9eba0f7be89f","name":"echo","parts":[{"text":"hello"}]}],"history":[{"messageId":"0cd99b6c-99d6-48ec-ba47-db7979203761","contextId":"31b60c79-757a-4f03-81ff-62e772a099ef","taskId":"6c9f746d-4b89-4079-afcd-4176f5218108","role":"ROLE_USER","parts":[{"text":"hello"}]}]}
  left: 200
 right: 400
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
EXIT=101
=== case-list-unknown.log
test adv_list_tasks_ignores_an_unrecognized_query_field ... FAILED
thread 'adv_list_tasks_ignores_an_unrecognized_query_field' (380932) panicked at crates/a2a-server/tests/adv_wire.rs:131:5:
assertion `left == right` failed: {"error":{"code":400,"status":"INVALID_ARGUMENT","message":"query.futureField: no field of this request is named so","details":[]}}
  left: 400
 right: 200
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
EXIT=101
=== case-get-unknown.log
test adv_get_task_ignores_an_unrecognized_query_field ... FAILED
thread 'adv_get_task_ignores_an_unrecognized_query_field' (380959) panicked at crates/a2a-server/tests/adv_wire.rs:140:5:
assertion `left == right` failed: {"error":{"code":400,"status":"INVALID_ARGUMENT","message":"query.futureField: no field of this request is named so","details":[]}}
  left: 400
 right: 200
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
EXIT=101
=== case-slowloris.log
test adv_a_slow_client_does_not_hold_back_another_client ... FAILED
thread 'adv_a_slow_client_does_not_hold_back_another_client' (716752) panicked at crates/a2a-server/tests/adv_availability.rs:40:5:
a second client waited 4.801247918s for the agent card behind a client sending one byte every 500 ms
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 5.00s
EXIT=101
=== case-emfile.log
test adv_subscribers_that_hang_up_do_not_take_the_server_down ... FAILED
subscribe streams opened and hung up: 60
thread 'adv_subscribers_that_hang_up_do_not_take_the_server_down' (716303) panicked at crates/a2a-server/tests/adv_availability.rs:155:5:
after 60 subscribers to a waiting task hung up, the server exited (Some(ExitStatus(unix_wait_status(256))), stderr "a2a-server: 127.0.0.1:32823: Too many open files (os error 24)\n") or no longer answers the card (None)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.52s
EXIT=101
```

The emfile case was red 10 times in 10 runs once it read the full head and first frame before hanging up.

## 3. Suite run, after the cases existed

The command was `~/.cache/a2a-gen/wave3/adv-server/run-prebuilt.sh full-suite-prebuilt`, run at 15:35. It executes every prebuilt a2a-server test binary from the package root, as `cargo test` does, without compiling: / had 37G free, under the 40G floor.

- The binaries are current. The source was last changed at 14:17 and the original test binaries were built at 14:25.
- The adv binaries were built before my rustfmt pass, which changed whitespace only.
- The first entry, `a2a_server-21772b829eb5fbb3`, is the server binary itself, not a test target. I killed it by PID (exit 143).

Result: the 55 original tests pass, 7 of my 16 pass and 9 are red. That is 71 tests executed. Full output:

```
     Running ~/.cache/b10x-target/a2a/debug/deps/a2a_server-21772b829eb5fbb3
a2a-server listening on http://127.0.0.1:8080
BINARY-EXIT=143
     Running ~/.cache/b10x-target/a2a/debug/deps/a2a_server-7ae573910e460b45

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

BINARY-EXIT=0
     Running ~/.cache/b10x-target/a2a/debug/deps/a2a_server-7439e0b443994ec5

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

BINARY-EXIT=0
     Running ~/.cache/b10x-target/a2a/debug/deps/binary-055b625a8e4eb317

running 3 tests
test the_binary_refuses_a_card_that_is_not_an_agent_card ... ok
test the_binary_serves_the_card_it_is_given ... ok
test the_binary_serves_the_built_in_card_at_the_address_it_bound ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

BINARY-EXIT=0
     Running ~/.cache/b10x-target/a2a/debug/deps/defines_none-278621538a3fa45d

running 1 test
test the_server_declares_no_type_the_generated_crate_exports ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

BINARY-EXIT=0
     Running ~/.cache/b10x-target/a2a/debug/deps/obligations-b3e4bb08b15c714d

running 34 tests
test errors_are_google_rpc_status_objects ... ok
test cancel_task_the_agent_cannot_cancel_now_is_not_cancelable ... ok
test cancel_task_of_an_unknown_task_is_task_not_found ... ok
test cancel_task_on_completed_task_is_not_cancelable ... ok
test extended_agent_card_without_the_capability_is_unsupported_operation ... ok
test extended_agent_card_declared_but_not_configured_is_not_configured ... ok
test cancel_task_cancels_a_task_waiting_for_input ... ok
test extended_agent_card_is_answered_when_declared_and_configured ... ok
test get_task_history_length_keeps_the_latest_messages ... ok
test send_message_history_length_zero_answers_no_history ... ok
test send_message_creates_a_completed_task_holding_the_echo_artifact ... ok
test get_task_of_an_unknown_task_is_task_not_found ... ok
test send_message_answers_a_direct_message_without_a_task ... ok
test list_tasks_answers_every_task_newest_first_without_artifacts ... ok
test send_message_with_a_malformed_body_is_invalid_argument ... ok
test send_message_returning_immediately_answers_the_task_before_the_agent_works ... ok
test list_tasks_with_a_page_size_out_of_range_is_invalid_argument ... ok
test push_notification_config_of_an_unknown_task_is_task_not_found ... ok
test push_notification_config_without_the_capability_is_push_notification_not_supported ... ok
test send_message_with_a_context_the_agent_cannot_accept_is_context_not_accepted ... ok
test send_message_in_a_version_the_card_does_not_declare_is_version_not_supported ... ok
test send_message_continues_a_task_waiting_for_input ... ok
test send_message_to_a_terminal_task_is_unsupported_operation ... ok
test send_message_with_a_part_of_an_unaccepted_media_type_is_content_type_not_supported ... ok
test send_message_to_an_unknown_task_is_task_not_found ... ok
test get_task_answers_the_task ... ok
test list_tasks_filters_by_context_and_status_and_includes_artifacts_on_request ... ok
test the_public_agent_card_is_served_at_the_well_known_path ... ok
test send_message_without_a_version_is_0_3_and_the_query_parameter_names_one ... ok
test list_tasks_pages_with_the_next_page_token ... ok
test send_message_without_a_required_extension_is_extension_support_required ... ok
test send_message_with_another_context_than_its_task_is_context_mismatch ... ok
test push_notification_config_is_created_read_listed_and_deleted ... ok
test tenant_routes_answer_as_the_plain_routes_do ... ok

test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

BINARY-EXIT=0
     Running ~/.cache/b10x-target/a2a/debug/deps/routes-f0f74e95241ece67

running 7 tests
test the_upstream_document_is_read ... ok
test every_hand_route_has_its_unmapped_row_and_every_unmapped_row_is_served ... ok
test every_upstream_route_is_served ... ok
test every_bound_route_is_a_route_of_the_generated_layer ... ok
test an_unknown_path_is_not_found_and_another_method_is_not_allowed ... ok
test the_generated_layers_own_routes_are_not_exposed ... ok
test every_served_route_answers_and_none_is_a_route_miss ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

BINARY-EXIT=0
     Running ~/.cache/b10x-target/a2a/debug/deps/streaming-95f2aa097fc91fa8

running 10 tests
test send_streaming_message_closes_at_an_interrupted_state ... ok
test subscribe_to_an_unknown_task_is_task_not_found ... ok
test send_streaming_message_without_the_capability_is_unsupported_operation ... ok
test send_streaming_message_errors_answer_before_the_stream_opens ... ok
test send_streaming_message_streams_the_task_then_its_updates_until_it_completes ... ok
test subscribe_to_task_sees_a_cancel ... ok
test send_streaming_message_answered_directly_is_one_message_event ... ok
test subscribe_without_the_capability_is_unsupported_operation ... ok
test subscribe_to_a_terminal_task_is_unsupported_operation ... ok
test subscribe_to_task_streams_the_task_then_what_another_request_does_until_it_completes ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

BINARY-EXIT=0
     Running ~/.cache/b10x-target/a2a/debug/deps/adv_agent-e3935c7018515382

running 2 tests
test adv_an_appended_chunk_extends_its_artifact_and_a_replacement_replaces_it ... ok
test adv_an_agent_cannot_move_a_completed_task_on ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

BINARY-EXIT=0
     Running ~/.cache/b10x-target/a2a/debug/deps/adv_availability-6ac2df0996d9c21e

running 2 tests
test adv_subscribers_that_hang_up_do_not_take_the_server_down ... FAILED
test adv_a_slow_client_does_not_hold_back_another_client ... FAILED

failures:

---- adv_subscribers_that_hang_up_do_not_take_the_server_down stdout ----
subscribe streams opened and hung up: 60

thread 'adv_subscribers_that_hang_up_do_not_take_the_server_down' (1350425) panicked at crates/a2a-server/tests/adv_availability.rs:155:5:
after 60 subscribers to a waiting task hung up, the server exited (Some(ExitStatus(unix_wait_status(256))), stderr "a2a-server: 127.0.0.1:38551: Too many open files (os error 24)\n") or no longer answers the card (None)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- adv_a_slow_client_does_not_hold_back_another_client stdout ----

thread 'adv_a_slow_client_does_not_hold_back_another_client' (1350424) panicked at crates/a2a-server/tests/adv_availability.rs:40:5:
a second client waited 4.803049999s for the agent card behind a client sending one byte every 500 ms


failures:
    adv_a_slow_client_does_not_hold_back_another_client
    adv_subscribers_that_hang_up_do_not_take_the_server_down

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.01s

BINARY-EXIT=101
     Running ~/.cache/b10x-target/a2a/debug/deps/adv_binary-09fc008441e0dd2a

running 3 tests
test adv_the_binary_refuses_a_listen_value_that_is_no_address ... ok
test adv_the_binary_refuses_a_card_path_that_does_not_exist ... ok
test adv_the_binary_refuses_a_card_that_is_not_json ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

BINARY-EXIT=0
     Running ~/.cache/b10x-target/a2a/debug/deps/adv_routes-3decb7ec98da78d4

running 1 test
test adv_tenant_twins_answer_what_their_plain_routes_answer ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

BINARY-EXIT=0
     Running ~/.cache/b10x-target/a2a/debug/deps/adv_wire-b343e920d5c852c0

running 8 tests
test adv_send_message_ignores_an_unrecognized_body_field ... ok
test adv_status_timestamps_have_millisecond_precision ... FAILED
test adv_list_tasks_ignores_an_unrecognized_query_field ... FAILED
test adv_get_task_refuses_a_negative_history_length_as_list_tasks_does ... FAILED
test adv_list_tasks_status_timestamp_after_includes_a_task_stamped_at_that_instant ... FAILED
test adv_list_tasks_second_page_does_not_repeat_the_first_when_a_task_arrives_between ... FAILED
test adv_get_task_ignores_an_unrecognized_query_field ... FAILED
test adv_subscribe_answers_the_post_route_the_specification_text_names ... FAILED

failures:

---- adv_status_timestamps_have_millisecond_precision stdout ----

thread 'adv_status_timestamps_have_millisecond_precision' (1351604) panicked at crates/a2a-server/tests/adv_wire.rs:74:5:
assertion `left == right` failed: `2026-10-22T13:39:16.414427459Z` is not YYYY-MM-DDTHH:mm:ss.sssZ
  left: 9
 right: 3

---- adv_list_tasks_ignores_an_unrecognized_query_field stdout ----

thread 'adv_list_tasks_ignores_an_unrecognized_query_field' (1351600) panicked at crates/a2a-server/tests/adv_wire.rs:131:5:
assertion `left == right` failed: {"error":{"code":400,"status":"INVALID_ARGUMENT","message":"query.futureField: no field of this request is named so","details":[]}}
  left: 400
 right: 200

---- adv_get_task_refuses_a_negative_history_length_as_list_tasks_does stdout ----

thread 'adv_get_task_refuses_a_negative_history_length_as_list_tasks_does' (1351599) panicked at crates/a2a-server/tests/adv_wire.rs:118:5:
assertion `left == right` failed: ListTasks refuses historyLength=-1 but GetTask answers: {"id":"e7e7e6da-0377-4c7d-85cd-1200cba65a1f","contextId":"7e0ceedd-d307-4f97-9ed5-f032fec93525","status":{"state":"TASK_STATE_COMPLETED","timestamp":"2026-10-22T13:39:16.414351022Z"},"artifacts":[{"artifactId":"070037e9-cda9-41e3-9033-33d0beca7c0a","name":"echo","parts":[{"text":"hello"}]}],"history":[{"messageId":"34d79091-bb3e-40b8-861b-c299262104b1","contextId":"7e0ceedd-d307-4f97-9ed5-f032fec93525","taskId":"e7e7e6da-0377-4c7d-85cd-1200cba65a1f","role":"ROLE_USER","parts":[{"text":"hello"}]}]}
  left: 200
 right: 400
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- adv_list_tasks_status_timestamp_after_includes_a_task_stamped_at_that_instant stdout ----

thread 'adv_list_tasks_status_timestamp_after_includes_a_task_stamped_at_that_instant' (1351602) panicked at crates/a2a-server/tests/adv_wire.rs:30:5:
statusTimestampAfter=2026-10-22T13:39:16.414735205Z (the task's own status timestamp) left the task out: {"tasks":[],"nextPageToken":"","pageSize":50,"totalSize":0}

---- adv_list_tasks_second_page_does_not_repeat_the_first_when_a_task_arrives_between stdout ----

thread 'adv_list_tasks_second_page_does_not_repeat_the_first_when_a_task_arrives_between' (1351601) panicked at crates/a2a-server/tests/adv_wire.rs:54:5:
assertion `left == right` failed: page 2 (token `1`) after one new task: {"tasks":[{"id":"560eb27d-f310-4d16-af78-d7da28c89ec1","contextId":"36332cb5-cb19-4df7-811b-68dc776699cb","status":{"state":"TASK_STATE_COMPLETED","timestamp":"2026-10-22T13:39:16.415132059Z"},"history":[{"messageId":"d49a7c02-6374-4774-9fd4-4c45dd67a53e","contextId":"36332cb5-cb19-4df7-811b-68dc776699cb","taskId":"560eb27d-f310-4d16-af78-d7da28c89ec1","role":"ROLE_USER","parts":[{"text":"hello"}]}]}],"nextPageToken":"2","pageSize":1,"totalSize":3}
  left: ["560eb27d-f310-4d16-af78-d7da28c89ec1"]
 right: ["cc3aca3e-ab01-4ad4-b523-a8b3b87e0006"]

---- adv_get_task_ignores_an_unrecognized_query_field stdout ----

thread 'adv_get_task_ignores_an_unrecognized_query_field' (1351598) panicked at crates/a2a-server/tests/adv_wire.rs:140:5:
assertion `left == right` failed: {"error":{"code":400,"status":"INVALID_ARGUMENT","message":"query.futureField: no field of this request is named so","details":[]}}
  left: 400
 right: 200

---- adv_subscribe_answers_the_post_route_the_specification_text_names stdout ----

thread 'adv_subscribe_answers_the_post_route_the_specification_text_names' (1351605) panicked at crates/a2a-server/tests/adv_wire.rs:102:5:
assertion `left == right` failed: POST /tasks/{id}:subscribe: [("Content-Type", "application/a2a+json"), ("Content-Length", "140"), ("Allow", "GET"), ("Connection", "close")]
  left: (405, "application/a2a+json")
 right: (200, "text/event-stream")


failures:
    adv_get_task_ignores_an_unrecognized_query_field
    adv_get_task_refuses_a_negative_history_length_as_list_tasks_does
    adv_list_tasks_ignores_an_unrecognized_query_field
    adv_list_tasks_second_page_does_not_repeat_the_first_when_a_task_arrives_between
    adv_list_tasks_status_timestamp_after_includes_a_task_stamped_at_that_instant
    adv_status_timestamps_have_millisecond_precision
    adv_subscribe_answers_the_post_route_the_specification_text_names

test result: FAILED. 1 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

BINARY-EXIT=101
EXIT=0
```

`EXIT=0` is the runner script's own line. The per-binary exit statuses are the `BINARY-EXIT` lines.

`task check` was not run. It builds (clippy, `cargo test --workspace`, build of generated) and / stayed at 35-39G. With my red cases in place it would be red anyway. Its non-build parts were run separately; see §5.

## 4. Findings

All findings cover the worktree named in the header. Every origin is `introduced`: `crates/a2a-server/` does not exist at 6520aef (`git show 6520aef:crates/a2a-server/Cargo.toml` gives "exists on disk, but not in '6520aef'"). The generated query refusal arrived with this unit's `--http` switch: at 6520aef, `generated/.../server/http.rs:399` reads "A key nothing declares is ignored".

| # | file:line | what was measured | what reaches it | verdict | severity |
|---|---|---|---|---|---|
| F1 | crates/a2a-server/src/ports.rs:1166 | `at > after` drops the task stamped at that instant; case ts-after, exit 101 | the documented ListTasks `statusTimestampAfter` parameter. Proto :694-696 says "greater than or equal" | NEEDS-CHANGE | warning |
| F2 | crates/a2a-server/src/ports.rs:1205 | the page token is an offset; page 2 repeats page 1 after one task is created between the calls; case paging, exit 101 | any client paging ListTasks while tasks are created (newest-first order shifts offsets). Spec :254 "MUST use cursor-based pagination", :258 "rather than offset-based" | CONFIRMED | warning |
| F3 | crates/a2a-server/src/ports.rs:207 | `{:09}` gives 9 fractional digits; case millis, exit 101 | every `status.timestamp` the server writes. Spec :1236 "Millisecond precision **SHOULD** be used" | CONFIRMED | note |
| F4 | crates/a2a-server/src/router.rs:79 | `POST /tasks/{id}:subscribe` gives 405 with `Allow: GET`; case post-subscribe, exit 101 | a client written from spec text :1169 (§5.3) or :2795 (§11.3.2), which say POST. Proto :76-82 and openapi :289-290 say GET, and proto is normative (:107). Upstream contradicts itself; I cannot settle it | CONFIRMED | note |
| F5 | crates/a2a-server/src/ports.rs:575 | GetTask `historyLength=-1` gives 200 with the full history, while ListTasks gives 400 for the same value; case neg-history, exit 101 | any GetTask client sending a negative length. Spec :467 asks for "consistent semantics across all operations" | CONFIRMED | note |
| F6 | crates/a2a-server/src/router.rs:768 | the hand ListTasks parser refuses unknown query keys with 400 "query.futureField: no field of this request is named so" (:641 does the same for :subscribe); case list-unknown, exit 101 | any client that adds a query parameter: a newer protocol revision's field or a proxy's cache-buster. Spec :1275 says implementations "SHOULD ignore unrecognized fields" | NEEDS-CHANGE | warning |
| F7 | ess/http.yaml:51 | bound GET routes are generated with `flat_object(.., ignore=false, "query")` (`generated/.../a2a_agent.rs:2117`), so GetTask refuses unknown keys (`http.rs:685`); case get-unknown, exit 101. The binding at :51, :77, :84 and :95 lacks `unknown: ignore`, which ESS offers (`current-features.md:110`) | the same clients as F6, on GetTask and the other bound GET routes. Reached since this unit added `--http` (`crates/a2a-xtask/src/main.rs:101`) | NEEDS-CHANGE | warning |
| F8 | crates/a2a-server/src/lib.rs:59 | `serve` handles one connection at a time. A client sending 1 byte every 500 ms held a second client for 4.80 s; case slowloris, exit 101. The 1 s per-read timeout (`generated/.../http.rs:48`) never fires at that spacing | any TCP client of the binary. The documented bind is `127.0.0.1:8080` | CONFIRMED | warning |
| F9 | crates/a2a-server/src/router.rs:384 | a subscribe stream to an input-required task stays in `followers` after its client hangs up. It is pruned only when an event write fails or the task ends (:408-423). On the prebuilt binary at the default `ulimit -n 1000000`, 100 abandoned streams took the process from 4 to 104 open descriptors, with the card still 200 (`fd-leak.log`) | the echo agent answers an empty text with INPUT_REQUIRED, and any client that subscribes and disconnects (a reconnect loop, for example) leaks one descriptor per stream | NEEDS-CHANGE | warning |
| F10 | crates/a2a-server/src/lib.rs:60 | `router.connection(connection?)` returns from `serve` at the first accept error, and the binary exits with status 1 ("Too many open files (os error 24)"); case emfile, exit 101 | the descriptor limit, reached through F9. I built it with `ulimit -n 64` (60 streams). On this machine (limit 1000000) it would take about a million; at a 1024 soft limit, about a thousand (inferred, not run) | NEEDS-CHANGE | warning |
| F11 | crates/a2a-server/tests/obligations.rs:586 | a mutant that dispatches the GetConfig tenant twin as ListConfigs (in `router.rs` `table()`, scratch copy) passed all 55 tests (`mutant-tenant-twin-config.log`, EXIT=0). `routes.rs:136` checks only that each route answers. My `adv_routes.rs` kills the mutant (`mutant-tenant-twin-config-killed.log`, EXIT=101) | future edits to `table()`. Nothing at run time: the unit answers correctly (adv_routes is green) | CONFIRMED | note |
| F12 | crates/a2a-server/tests/defines_none.rs:36 | the scanner reads only `pub `, `pub(crate) `, `pub(super) ` or a bare kind at line start. I planted `pub unsafe trait AgentCard {}` and `pub(in crate::agent) struct Task(..)` in a copy of `agent.rs`; the test stayed ok (`mutant-defines-none.log`), while a control plant `pub struct Message;` was caught (`-control.log`). `rustfmt --check` accepts the plant. I did not compile it (disk floor) | the AGENTS.md row added by this unit says "Defines no model type; `tests/defines_none.rs` fails if it does", which overstates the guard | CONFIRMED | note |

## 5. Attacked and could not break

- Illegal agent moves: steps after Complete are dropped, the states run SUBMITTED, WORKING, COMPLETED, and only artifact "first" is kept (adv_agent.rs, green).
- Artifact append and replace (adv_agent.rs, green).
- Binary start-up: a bad `--listen`, a missing `--card` and a non-JSON card each exit non-zero and name the input (adv_binary.rs, 3 green).
- Unknown fields in a SendMessage body are ignored (adv_wire.rs, green).
- Tenant twins of 5 GET routes return the same status and body as the plain routes (adv_routes.rs, green on the unit).
- Route coverage and the hand YAML parser: `routes.rs:73` pins 22 parsed routes, and every upstream route is served. The prebuilt xtask `http-paths` passed (`http-paths.log`, EXIT=0); a route renamed in a copy of the projection failed it (`http-paths-plant.log`, EXIT=1).
- `generated/` matches a fresh synthesis byte for byte: the prebuilt xtask `drift` passed (`drift.log`, "no drift from ess/", EXIT=0); a one-word edit in a copy of `http.rs` failed it ("edited: src/server/http.rs", EXIT=1). Producer: ess 0.67.0, matching `state.json`. The binary contains the `--http`/`ess/http.yaml` strings, so it was built from this unit's xtask.
- conform-interpreted, run as its two `ess verify conform` commands into my scratch dir: "168 scenarios: 152 passed, 0 failed, 16 error, 0 unsupported", EXIT=3. That matches the implementor's figure. `ess/` has no change against 6520aef.
- The Taskfile change from `cargo fmt --all --check` to `cargo fmt --check`: the plain form checks the same 17 workspace-member files. `--all` adds only the 2 targets of `generated/rust/a2a`, now a path dependency of a2a-server, and fails on them with 441 diffs (`fmt-all.log`). At 6520aef no member depended on `generated`, so nothing lost coverage. Plant: my own unformatted adv files made `cargo fmt --check` exit 1 before I formatted them.
- Tenant isolation is agent-defined (spec §13.1) and the default card declares no tenant, so I did not raise it.
- Push-config ids colliding across tasks: task ids are server-made UUIDs. INFEASIBLE; not raised.

## 6. Paths written outside the worktree

- `~/.cache/a2a-gen/wave3/adversary-server.md` (this report).
- `~/.cache/a2a-gen/wave3/adv-server/`, about 185M in total:
  - scripts: run-case.sh, run-mutant.sh, run-prebuilt.sh, fd-leak.sh
  - logs: case-*.log (16), mutant-tenant-twin.log, mutant-tenant-twin-config.log, mutant-tenant-twin-config-killed.log, mutant-defines-none.log, mutant-defines-none-control.log, full-suite-prebuilt.log, fmt-all.log, fmt-all-files.txt, fmt-plain-files.txt, drift.log, drift-plant.log, http-paths.log, http-paths-plant.log, fd-leak.log
  - directories: fd-leak/ (server output and send.json); conf/ (suite.json, report.json, run.log); drift/a2a-xtask-drift/ (fresh synthesis); drift-plant/ (copies of ess/, upstream/, generated/rust/a2a and generated/openapi-http, with plants); mut/ (mutant workspace; its src/agent.rs still carries the defines_none plant); mut-target/ (180M build directory)
- `~/.cache/b10x-target/a2a/tmp/adv-not-json-card.json` (adv_binary.rs; the missing-card case removes its own path). The adv test binaries were built into the brief's `~/.cache/b10x-target/a2a/`.

```findings
- {"file": "crates/a2a-server/src/ports.rs", "line": 1166, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "statusTimestampAfter filters with a strict greater-than, so a task stamped at exactly that instant is left out where proto 694-696 says greater than or equal"}
- {"file": "crates/a2a-server/src/ports.rs", "line": 1205, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "ListTasks page tokens are offsets, so a task created between two page calls repeats an item, against the spec's MUST for cursor-based pagination"}
- {"file": "crates/a2a-server/src/ports.rs", "line": 207, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "status timestamps carry nine fractional digits where the spec says millisecond precision SHOULD be used"}
- {"file": "crates/a2a-server/src/router.rs", "line": 79, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "POST on tasks/{id}:subscribe answers 405 while spec sections 5.3 and 11.3.2 name POST and proto and openapi name GET, an upstream contradiction"}
- {"file": "crates/a2a-server/src/ports.rs", "line": 575, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "GetTask answers historyLength=-1 with the full history while ListTasks refuses the same value with 400"}
- {"file": "crates/a2a-server/src/router.rs", "line": 768, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the hand ListTasks and subscribe query parsers refuse an unrecognized query key with 400 where the spec says implementations SHOULD ignore unrecognized fields"}
- {"file": "ess/http.yaml", "line": 51, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the query-encoded bindings lack unknown: ignore, so since the --http switch the generated GetTask route refuses an unrecognized query key with 400"}
- {"file": "crates/a2a-server/src/lib.rs", "line": 59, "category": "concurrency", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "one client trickling a request a byte every 500 ms holds every other client, measured 4.8 s for the agent card"}
- {"file": "crates/a2a-server/src/router.rs", "line": 384, "category": "concurrency", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a subscribe stream to an input-required task keeps its descriptor after the client hangs up, measured 4 to 104 open descriptors after 100 abandoned streams"}
- {"file": "crates/a2a-server/src/lib.rs", "line": 60, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "serve returns at the first accept error, so reaching the descriptor limit through leaked subscribers exits the binary with Too many open files"}
- {"file": "crates/a2a-server/tests/obligations.rs", "line": 586, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "a mutant dispatching the GetConfig tenant twin as ListConfigs passed all 55 existing tests and is killed only by the added adv_routes case"}
- {"file": "crates/a2a-server/tests/defines_none.rs", "line": 36, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the defines-none scanner misses pub unsafe trait and pub(in path) declarations, so a planted AgentCard trait and Task struct pass it while AGENTS.md says the test fails on any model type"}
```
