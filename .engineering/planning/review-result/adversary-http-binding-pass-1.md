---
format: aep.planning-md/3
id: review-result:adversary-http-binding-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:http-binding
relations:
- reviews: story:http-binding
revision: 1
---
unit: story:http-binding (adversary), uncommitted working tree on e9e8742 in a2a-w2-http-binding
verdict: red (NEEDS-CHANGE: the gate checks the (method, path) set only, so a route bound to the wrong command or the wrong input field passes task check)
cases: executed 5→8 (cargo test --workspace), red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: ~/.cache/a2a-gen/wave2/adv-http-binding/, ~/.cache/b10x-target/a2a/tmp/adv-http-binding/, this report
needs-coordinator: yes (decide whether finding 1 holds the unit; the fix is named in finding 1)

## 1. Diff stat (worktree)

```
 AGENTS.md                                 |  11 +-
 CHANGELOG.md                              |   8 +
 Taskfile.yml                              |  15 +-
 crates/a2a-xtask/src/main.rs              | 290 ++++++++++++++++++++++++++++++
 generated/rust/a2a/.ess-output/state.json |   2 +-
 5 files changed, 320 insertions(+), 6 deletions(-)
?? crates/a2a-xtask/tests/adv_http_binding.rs   (mine, the only path I added; a test file)
```
The stat is the implementor's, unchanged. I edited no implementation, spec or generated file.

## 2. Cases added

`crates/a2a-xtask/tests/adv_http_binding.rs`, 3 tests. Each takes the committed projection, applies the exact bytes `ess generate --http` emits for a mutated `ess/http.yaml` (ESS validates the mutated binding: `10 route(s), 0 open, valid`), and runs the built `a2a-xtask http-paths --projected <file>`.

| Test | Asserts | Now |
|---|---|---|
| `control_the_committed_projection_passes` | committed projection passes | green |
| `a_route_bound_to_the_wrong_command_is_drift` | operationIds of `post /message:send` and `post /message:stream` exchanged: refused | red |
| `path_variables_bound_to_the_wrong_input_fields_are_drift` | `get /tasks/{taskId}/pushNotificationConfigs/{id}` with `{taskId: input.id, id: input.task_id}`: refused | red |

The red output from the run of that file alone (`cargo test --locked -p a2a-xtask --test adv_http_binding`, exit 101):
```
test a_route_bound_to_the_wrong_command_is_drift ... FAILED
test path_variables_bound_to_the_wrong_input_fields_are_drift ... FAILED
post /message:send now runs a2a.tasks.SendStreamingMessage (upstream operationId A2AService_SendMessage), and http-paths passed:
~/.cache/b10x-target/a2a/tmp/adv-http-binding/swapped-commands.yaml: 9 (method, path) pairs equal to upstream/openapi/a2a.openapi.yaml, 25 differences named in ess/http-unmapped.tsv
get /tasks/{taskId}/pushNotificationConfigs/{id} now binds {taskId} to input.id and {id} to input.task_id, and http-paths passed:
~/.cache/b10x-target/a2a/tmp/adv-http-binding/swapped-parameters.yaml: 9 (...) pairs equal ..., 25 differences named ...
test result: FAILED. 1 passed; 2 failed
```
I also ran the same mutations end to end through ESS in scratch (`binding-plants.sh`). I added a third swap there: GET runs DeleteTaskPushNotificationConfig and DELETE runs Get. All three gave ESS validate exit 0, generate exit 0 and http-paths exit 0.

## 3. Suite

`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only b10x-build-slot task check`, **EXIT=201** (log: `~/.cache/a2a-gen/wave2/adv-http-binding/check.log`)
- spec, projections-drift (including `openapi-http current`), drift, http-paths (`9 ... equal, 25 differences named`), fmt and clippy all pass.
- `cargo test --workspace`: the 5 unit tests pass. `adv_http_binding` has 1 passed and 2 failed (the two cases above).

Conform, run in scratch with the Taskfile's two commands: `168 scenarios: 152 passed, 0 failed, 16 error, 0 unsupported`, the same as before. `git diff --stat -- generated/openapi generated/asyncapi generated/schema` is empty.

## 4. Findings

| # | file:line | Verdict / origin | Finding |
|---|---|---|---|
| 1 | crates/a2a-xtask/src/main.rs:196 | NEEDS-CHANGE / introduced | `compare` sees only (method, path), so binding the wrong command or input field passes `task check`. **Measured:** the 2 red cases, plus 3 ESS-validated plants in scratch. **Reached by:** any edit of `ess/http.yaml`, the workflow the gate's own error message prescribes. The acceptance asks for a binding that maps every command; the gate checks only the path set. **Fix (not applied):** for each equal pair, compare upstream `operationId` `A2AService_<Rpc>` with the projected operationId's last segment (one named rename: ListTasks to view `Tasks`). Also compare each upstream path parameter, snake-cased, with the projected `x-ess-field`, or with its name when that is absent. |
| 2 | ess/http.yaml:8 | CONFIRMED / introduced | The tenant-variant gap gives ESS-HTTP-005 and 009 as its cause, and that is incomplete: upstream's own routes overlap under ESS-HTTP-011. `get /{tenant}/tasks` overlaps `get /tasks/{id}` (GET /tasks/tasks). `get /{tenant}/tasks/{id}` overlaps the moved `get /views/tasks/pushNotificationConfigs`, so the move collides again once tenant routes exist. **Measured:** probes `overlap-probe` and `views-overlap-probe` on stand-in routes, both refused `ESS-HTTP-011 ... a literal takes no precedence over a variable`. **Reached by:** the ESS gap filing and the 11 TSV reasons, which name 005 and 009 only. Severity: note. |

## 5. Attacked, not broken

- **Comparator plants, all red (exit 1):** `{taskId}` renamed to `{task_id}`; trailing slash; method `GET:`; flow-style `get: {}`; a path key with a space before `:`; a path key with a trailing comment.
- **TSV:** a duplicate row is refused. Rows that are stale, for the wrong side, for an equal pair, or that differ from the pair only by a trailing slash are all refused. All 25 reasons match the projected operationIds, apart from finding 2.
- **Gate (scratch repository copies, exact Taskfile steps):**
  - Drift planted in the committed `generated/openapi-http`: red in `task projections-drift` and in `task http-paths` (exit 201).
  - Binding edited without regenerating: red in `projections-drift`.
  - Spec edited without updating the digest: `task spec` fails with `ESS-HTTP-002`.
  - `task --dry check` lists `http-paths`, and CI `check.yml` runs `task check`.
- **The 9 equal routes against the a2a.proto `google.api.http` options:**
  - All 9 bind the right command or view, and every path variable fills the right field: `{id}` is `GetTaskRequest.id`, `{taskId}` is `task_id` and `{id}` is the config id.
  - `/views/tasks/pushNotificationConfigs` overlaps nothing that exists today.
- **Regressions:** none. Conform is 152/0/16 and the existing projections are byte-identical.

## 6. Paths written outside the worktree

- `~/.cache/a2a-gen/wave2/adv-http-binding/`:
  - scripts: `plants.sh`, `binding-plants.sh`, `gate-plants.sh`
  - `plants/` (comparator fixtures)
  - `bind/` (sed files and 5 ESS copies with projections)
  - `gate/g0..g3/` (repository copies, without `.git`)
  - `gate/target/` (scratch cargo target)
  - `conf/` (suite.json, report.json, run.log)
  - `check.log`
- `~/.cache/b10x-target/a2a/tmp/adv-http-binding/` (fixtures written by the test; the `adv_http_binding` test binary is in the shared target)
- `~/.cache/a2a-gen/wave2/adversary-http-binding.md` (this report)

## 7. Findings block

```findings
[
  {"file": "crates/a2a-xtask/src/main.rs", "line": 196, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "http-paths compares only (method, path), so an ess/http.yaml binding a route to the wrong command or a path variable to the wrong input field validates, projects and passes task check (crates/a2a-xtask/tests/adv_http_binding.rs, 2 red)"},
  {"file": "ess/http.yaml", "line": 8, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the tenant-variant gap and its 11 TSV reasons name ESS-HTTP-005/009 only, but upstream's /{tenant}/ routes also overlap /tasks/{id} and the moved /views/tasks/pushNotificationConfigs under ESS-HTTP-011, as probes on stand-in routes show"}
]
```
