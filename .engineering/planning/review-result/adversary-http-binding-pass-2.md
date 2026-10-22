---
format: aep.planning-md/3
id: review-result:adversary-http-binding-pass-2
kind: review-result
status: active
title: Adversary pass 2 on story:http-binding
relations:
- reviews: story:http-binding
revision: 1
---
unit: story:http-binding (adversary, pass 2), uncommitted working tree on e9e8742 in a2a-w2-http-binding
verdict: red (2 red cases, INFEASIBLE: the path-field check falls back to the parameter name for a view path parameter; plus 1 CONFIRMED note on 5 TSV reasons; pass 1's 3 swaps and 5 new swaps are all refused)
cases: executed 14→16 (cargo test -p a2a-xtask: 11 unit + 3 pass-1 + 2 pass-2), red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths (~/.cache/a2a-gen/wave2/adv-http-binding/pass2/, ~/.cache/b10x-target/a2a/tmp/adv-http-binding-pass2/, this report); inside the tree, ignored by git: target/ (73M), created by my direct cargo runs
needs-coordinator: yes (1. whether finding 1, INFEASIBLE today, holds the unit; 2. the worktree's target/ is mine, left in place because the rules forbid deleting a target/ by hand)

## 1. Diff stat (worktree)

```
 AGENTS.md                                 |  11 +-
 CHANGELOG.md                              |   8 +
 Taskfile.yml                              |  15 +-
 crates/a2a-xtask/src/main.rs              | 607 +++++++++++++++++++++++++++++-
 generated/rust/a2a/.ess-output/state.json |   2 +-
 5 files changed, 636 insertions(+), 7 deletions(-)
?? crates/a2a-xtask/tests/adv_http_binding.rs        (pass 1, unchanged)
?? crates/a2a-xtask/tests/adv_http_binding_pass2.rs  (pass 2, mine, 105 lines)
```
The tracked stat is entirely the implementor's. The only file I added is a test file. Running `cargo` outside the Taskfile also created the ignored `target/` in the tree (see the header).

## 2. Cases added

`crates/a2a-xtask/tests/adv_http_binding_pass2.rs`. Fixture: ESS 0.67.0 projects a view path parameter (`parameters: {taskId: param.task_id}` on view `a2a.tasks.TaskPushNotificationConfigs`, probe `p2-view-param-probe`, validate exit 0) with no `x-ess-field` and the description "Read by the view's filter as `param.task_id`; carried in the path rather than the query." `operations()` (main.rs:351, `p.marked.or(p.described).unwrap_or_else(|| p.name.clone())`) reads neither, so it compares the parameter's name with itself. Each case puts that ESS-written parameter on `get /tasks/{taskId}/pushNotificationConfigs/{id}`.

| Test | Asserts | Now |
|---|---|---|
| `a_view_style_path_parameter_filling_the_wrong_field_is_drift` | `{id}` filling `task_id` (view style) is refused | red |
| `a_view_style_path_parameter_filling_the_right_field_passes` | `{taskId}` filling `task_id` (view style) passes | red |

Run alone (`cargo test --locked -p a2a-xtask --test adv_http_binding_pass2`, EXIT=101):
```
test a_view_style_path_parameter_filling_the_wrong_field_is_drift ... FAILED
test a_view_style_path_parameter_filling_the_right_field_passes ... FAILED
{id} fills `task_id` (as ESS writes a view path parameter), and http-paths passed:
.../view-style-wrong-field.yaml: 9 (method, path) pairs equal to upstream/openapi/a2a.openapi.yaml, each bound to upstream's operation and path fields, 25 differences named in ess/http-unmapped.tsv
{taskId} fills `task_id` (as ESS writes a view path parameter), and http-paths refused it:
  get /tasks/{taskId}/pushNotificationConfigs/{id}: path parameter `{taskId}` fills upstream field `task_id`, the projection fills `taskId`
test result: FAILED. 0 passed; 2 failed
```

## 3. Suite

`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only b10x-build-slot task check`: **EXIT=201** (log: `~/.cache/a2a-gen/wave2/adv-http-binding/pass2/check.log`).
- spec, projections-drift (4 × current, including `openapi-http`), drift, http-paths ("9 … each bound to upstream's operation and path fields, 25 differences"), fmt and clippy all pass.
- Tests: 11 unit and 3 pass-1 pass. The 2 pass-2 cases are the only failures.

## 4. Findings

| # | file:line | Verdict / origin | Finding |
|---|---|---|---|
| 1 | crates/a2a-xtask/src/main.rs:351 | INFEASIBLE / introduced | **Measured:** the 2 red cases above. ESS writes a view path parameter's field only as `` `param.<field>` ``, so the check falls back to the name. Effects: a single-word upstream name (`{id}`) passes whatever it fills, and a camel-case one (`{taskId}`) is refused even when right. **Reached by:** nothing today. The only view on an equal pair is `get /tasks` (RENAMED), which has no path variable. It is reached once RENAMED gains a view with a path variable, for example GetTask as a by-identity view, or ListTaskPushNotificationConfigs as view `TaskPushNotificationConfigs`, which is already "List push notification configs". **Fix (not applied):** also read `` Read by the view's filter as `param.<field>` ``, and refuse a projected path parameter that has neither a marker nor a readable description. Keep the name fallback for upstream only, as the doc comment at :236 already says. |
| 2 | ess/http-unmapped.tsv:2 | CONFIRMED / introduced | The pass-2 reason "ESS-HTTP-011 /{tenant}/... overlaps /tasks/{id} and /views/tasks/pushNotificationConfigs" was stamped on all 11 tenant rows. It is false for 5 of them: lines 2, 3, 9, 10 and 11 (`post …/message:send`, `post …/message:stream`, `post …/{taskId}/pushNotificationConfigs`, and `get` and `delete …/{taskId}/pushNotificationConfigs/{id}`). ESS-HTTP-011 is per method, and no other route of the same method has the same segment count. **Measured:** stand-ins `post /{id}/message:send`, `post /{taskId}/tasks/x/pushNotificationConfigs`, `get /{taskId}/tasks/x/pushNotificationConfigs/{id}` and `delete` of the same path all validate (exit 0, no 011). Control `get /{taskId}/extendedAgentCard` is refused with ESS-HTTP-011 against `get /tasks/{id}`. **Reached by:** the gap filing and anyone reading the allow-list. Severity: note. |

## 5. Attacked, not broken

- **Pass 1's 3 swaps re-planted end to end** (ESS validate 0, generate 0): Get↔Delete push config, `{taskId: input.id, id: input.task_id}`, and send↔stream all give http-paths EXIT=1 with both sides named. The pass-1 tests now pass 3/3.
- **RENAMED** can neither admit another view nor hide a swap. It is keyed on the RPC and compares the full id.
  - Tasks↔AllTasks paths swapped (11 routes, valid): EXIT=1, "get /tasks: … binds a2a.tasks.AllTasks".
  - GetExtendedAgentCard↔Tasks paths swapped: EXIT=1 on both routes.
- **New swaps, all EXIT=1:**
  - DELETE path fields swapped.
  - GetTask↔ListTaskPushNotificationConfigs, paths and fields: both the operation and the field are flagged.
- **Description fallback on commands:** ESS writes "The input field `X`" on all 7 command path parameters. The only single-word, required, path-bindable field is `id`, and every other field carries `x-ess-field`, so no command binding falls back to the name. A description naming the wrong field would need a hand edit of `generated/openapi-http`, which projections-drift refuses.
- **operationId compared on its last segment only:** no two operations share a local name across `discovery` and `tasks`, so this is unreachable.
- **Regressions:** none. projections-drift is current for all 4 projections. Conform was not re-run: pass 2 touched no domain file.

## 6. Paths written

- `~/.cache/a2a-gen/wave2/adv-http-binding/pass2/`:
  - `plants.sh`, 13 `.sed` plants
  - `out/<plant>/` (ESS copies and projections)
  - `probes.log`, `plants.log`, `case-alone.log`, `check.log`
- `~/.cache/b10x-target/a2a/tmp/adv-http-binding-pass2/` (test fixtures from `task check`)
- Inside the worktree, ignored by git: `target/` (73M, my direct `cargo build`/`cargo test` runs, including `target/tmp/adv-http-binding*/`). Not deleted.
- `~/.cache/a2a-gen/wave2/adversary-http-binding-pass-2.md` (this report)

## 7. Findings block

```findings
[
  {"file": "crates/a2a-xtask/src/main.rs", "line": 351, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "a projected path parameter with neither x-ess-field nor an 'The input field' description falls back to its own name, which is how ESS writes every view path parameter, so a view-bound {id} passes whatever field it fills and a correct {taskId} is refused (crates/a2a-xtask/tests/adv_http_binding_pass2.rs, 2 red); no equal route binds a view with a path variable today"},
  {"file": "ess/http-unmapped.tsv", "line": 2, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the ESS-HTTP-011 overlap reason is stamped on all 11 tenant rows, but ESS-HTTP-011 is per method and rows 2, 3, 9, 10 and 11 overlap nothing, as ESS-validated stand-ins for each show"}
]
```
