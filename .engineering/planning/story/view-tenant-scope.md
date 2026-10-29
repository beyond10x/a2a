---
format: aep.planning-md/3
id: story:view-tenant-scope
kind: story
status: draft
title: TaskPushNotificationConfigs view scoped to the caller's tenant
summary: The push-config view answers a client only its own tenant's configs; found by wave 4 reviews of story:conformance-gate.
relations:
- decomposes: epic:a2a-delivery
revision: 1
---
## Outcome

The `a2a.tasks.TaskPushNotificationConfigs` view answers a client only the push configs of its
own tenant, so no caller of the specification's surface reads another tenant's webhook token.

## Acceptance

`ess/domains/tasks.yaml` gives the view a tenant input and filters rows by it (or withdraws the
`a2a.tasks.Client` read grant); `crates/a2a-server`'s `InProcess::call` answers a client of
tenant B no row of tenant A's task, and a conformance scenario claims it.

## Context

Found by the wave 4 security review and adversary on story:conformance-gate (F2, J5): through
`InProcess`, a client of no tenant read tenant A's config with its token. The binary routes no
view, so nothing reaches it over HTTP; `InProcess` is documented as a trusted harness entry until
this lands.
