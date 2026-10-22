---
format: aep.planning-md/3
id: decision-blocker:http-binding-shape
kind: decision-blocker
status: cleared
title: HTTP+JSON binding projection and UNMAPPED routes
relations:
- blocks: story:http-binding
revision: 2
transitions:
- {from: "open", to: "cleared", at: "2026-10-22T08:54:41Z", actor: "human:timo", revision: 2}
---
## Question

story:http-binding: does the A2A HTTP+JSON binding replace `generated/openapi`, and what happens to routes ESS cannot express?

## Decision

- The binding projects into a new `generated/openapi-http`; `generated/openapi` stays.
- 13 of 22 upstream (method, path) pairs stay UNMAPPED, each named with its ESS refusal in `ess/http-unmapped.tsv`; the gate refuses an unnamed or stale difference.
- The non-A2A view TaskPushNotificationConfigs moves to `/views/tasks/pushNotificationConfigs`, because its derived route collides with an upstream route (ESS-HTTP-011).
- Where spec 11.3 and a2a.proto disagree (subscribe method, variable names), the proto wins: the acceptance names the proto-derived OpenAPI.

Taken by the wave coordinator under approval-record:first-release-scope.
