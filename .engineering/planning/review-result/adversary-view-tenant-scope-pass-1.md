---
format: aep.planning-md/3
id: review-result:adversary-view-tenant-scope-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:view-tenant-scope
relations:
- reviews: story:view-tenant-scope
revision: 1
---
# Adversary pass 1 on story:view-tenant-scope

```findings
[
{"file":"ess/scenarios/push-config-view-other-tenant.yaml","line":52,"category":"mutant","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"No scenario held the filter's both-absent branch; a view answering tenant-less reads no row passed task conform (adv_w5_view_tenant.rs). Fixed: authored scenario push-config-view-without-tenant."},
{"file":"crates/a2a-conformance/src/server.rs","line":104,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"REARRANGED's premise was false for Delete: outcome/deleted claims the PushNotificationConfigs rows' tenant. Fixed: REARRANGED lists Get only; sec_tenant_guards still turns red for each of the four guards."},
{"file":"ess/scenarios/push-config-view-of-task.yaml","line":37,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"Duplicate tenant key in the Create input mapping, accepted silently by ESS 0.75.0. Fixed: duplicate removed."}
]
```
