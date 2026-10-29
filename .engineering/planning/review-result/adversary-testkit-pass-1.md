---
format: aep.planning-md/3
id: review-result:adversary-testkit-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:testkit
relations:
- reviews: story:testkit
revision: 1
---
Pass on wave/4-int fa86a97 (unit f9eceb6); cases in commit 11a07aa (crates/a2a-testkit/tests/adv_*.rs), 25 to 43 executed, 14 red; the CLI caught 5 of 11 planted endpoint defects. Every finding fixed in 3c88dec, merged as 0bca02d. F5 and F10 decided by the coordinator: exit 1 when no scenario passed; --bearer-file.

```findings
[
{"file":"crates/a2a-testkit/src/target.rs","line":91,"category":"mutant","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F1: a 200 to CancelTask or DeleteTaskPushNotificationConfig of an unknown id was reported unsupported before expect_outcome ran"},
{"file":"crates/a2a-testkit/src/target.rs","line":80,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F2: any external-outcome refusal made the scenario unsupported, so a stub creating no task got a2a-server's verdict"},
{"file":"crates/a2a-testkit/src/target.rs","line":153,"category":"mutant","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F3: the wrong refusal for an unknown id was reported unsupported as ambiguous instead of failed"},
{"file":"crates/a2a-testkit/src/target.rs","line":399,"category":"mutant","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F4: a refusal was read from the ErrorInfo reason alone, so TASK_NOT_FOUND on 500 INTERNAL passed"},
{"file":"crates/a2a-testkit/src/main.rs","line":450,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F5: a run in which no scenario passed exited 0"},
{"file":"crates/a2a-testkit/src/expect.rs","line":129,"category":"mutant","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F6: expect_task_state accepted a different task than the one requested"},
{"file":"crates/a2a-testkit/src/expect.rs","line":153,"category":"mutant","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F7: expect_error_reason accepted the reason on any HTTP status"},
{"file":"crates/a2a-testkit/src/expect.rs","line":197,"category":"mutant","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F8: expect_stream_states accepted a stream with no task event and updates of other tasks"},
{"file":"crates/a2a-testkit/src/expect.rs","line":318,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F9: expect_push_config_roundtrip ignored a dropped authentication and a missing task_id"},
{"file":"crates/a2a-testkit/src/main.rs","line":464,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F10: the CLI sent no credential, so a server with a token-protected extended card was reported in error"},
{"file":"crates/a2a-testkit/src/main.rs","line":470,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"F11: an unwritable --results path ran the suite and printed none of its lines"}
]
```
