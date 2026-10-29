---
format: aep.planning-md/3
id: review-result:plan-critics-wave-4
kind: review-result
status: active
title: Plan-critic panel on wave 4 stories
relations:
- reviews: story:client
- reviews: story:conformance-gate
- reviews: story:testkit
prose_only: four critic reports, findings fixed in the story text
revision: 1
---
## Panel

One plan-critic round on story:client, story:conformance-gate and story:testkit (revision 3 of
each), 2026-10-29: acceptance, design, parallel-safety and scope. All four returned
`needs-revision`. Every finding was fixed in the story text (revision 5); none was skipped and no
story was dropped.

## Findings and fixes

- acceptance, client item 1 (blocker): the public card path is not in the upstream OpenAPI.
  Fixed: named as the one exception; sent pairs read from a recording socket.
- acceptance, client item 2: no asserted value per method. Fixed: one value per method.
- acceptance, client item 4: truncated stream not told apart from a closed stream. Fixed:
  defined as a stream ending inside an event.
- acceptance, conformance-gate item 4 (blocker): unnamed scenario set. Fixed: the four
  `a2a.tasks.Task/state/*/refuses/a2a.tasks.CancelTask` ids named, `CancelTask/outcome/canceled`
  must still pass.
- acceptance and design, conformance-gate item 3 / testkit item 4: suite path unspecified.
  Fixed: `task conform` writes `$HOME/.cache/a2a-gen/conform/` fresh; the testkit test
  synthesizes its own suite.
- acceptance, testkit items 4 and 5 (blocker): pass and fail sets unnamed. Fixed: the two
  `not-found` scenarios named; state the wire cannot reset is `unsupported`.
- design, Target seam: no way for a target to say "unsupported". Fixed: every `Target` method
  returns `TargetError::Unsupported(reason)`, tested with a stub target.
- design, testkit: command-to-wire mapping owner unstated. Fixed: derived from `ess/http.yaml`
  inside `a2a-testkit`.
- parallel-safety, client and conformance-gate share `Cargo.lock`, `CHANGELOG.md`, `README.md`,
  `AGENTS.md`. Fixed: stated in both Scope sections; the coordinator resolves the merges.
- scope, testkit narrows "any A2A endpoint" to `http://` HTTP+JSON. Fixed: stated in the
  Outcome as a later milestone for TLS and JSON-RPC.
