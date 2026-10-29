---
format: aep.planning-md/3
id: review-result:adversary-testkit-wire-events-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:testkit-wire-events
relations:
- reviews: story:testkit-wire-events
revision: 1
---
# Adversary pass 1 on story:testkit-wire-events

```findings
[
{"file":"crates/a2a-testkit/src/target.rs","line":1213,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"begin() forgot every task the endpoint had answered, so a message answered with a task an earlier scenario created read as TaskCreated and SendMessage/outcome/task-created passed (adv_w5_wire_events.rs a_message_answered_with_a_task_an_earlier_scenario_created_is_not_task_created). Fixed in 74cc462: run-wide answered task ids; such an answer is no declared outcome and the step fails."},
{"file":"crates/a2a-testkit/src/target.rs","line":805,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"A SubscribeToTask stream opening with another task than request.id read as subscribed (a_subscription_that_streams_another_task_is_not_subscribed). Fixed in 74cc462: a success answer holding another task than the request's id (GetTask, CancelTask, SubscribeToTask) is no declared outcome."},
{"file":"crates/a2a-testkit/src/target.rs","line":1119,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"context_of filled a row's context_id from the scenario's request when the endpoint answered none (a_task_answered_without_a_context_does_not_pass_a_claim_on_its_context). Fixed in 74cc462: such a row is unsupported."},
{"file":"crates/a2a-testkit/src/target.rs","line":1240,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"A gap A command that succeeds on another tenant's task is read as the expected outcome; it stays unsupported only because the push-config events are not carried. The adversary's gap_a guard pins it; no change."}
]
```
