---
format: aep.planning-md/3
id: review-result:security-view-tenant-scope-pass-1
kind: review-result
status: active
title: 'Security review of story:view-tenant-scope: no tenant leak, two notes'
relations:
- reviews: story:view-tenant-scope
revision: 1
---
# Security review of story:view-tenant-scope: no tenant leak, two notes

```findings
[
{"file":"crates/a2a-conformance/src/server.rs","line":104,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"REARRANGED's comment said no claim reads a configuration's tenant, but Delete outcome/deleted claims it. Fixed in 9c7e12c: REARRANGED lists Get only."},
{"file":"ess/scenarios/push-config-view-of-task.yaml","line":37,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The Create input repeated the key tenant. Fixed in 9c7e12c."}
]
```
