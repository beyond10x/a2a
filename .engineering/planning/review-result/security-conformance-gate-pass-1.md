---
format: aep.planning-md/3
id: review-result:security-conformance-gate-pass-1
kind: review-result
status: active
title: Security review pass 1 on story:conformance-gate
relations:
- reviews: story:conformance-gate
revision: 1
---
Pass on wave/4-int 5abc010 (unit 3c654bb); cases in commit 4a6d114 (crates/a2a-server/tests/sec_cg_inprocess.rs, crates/a2a-conformance/tests/sec_tenant_guards.rs), 148 to 156 executed, 3 red. Fixed in d1755ca, merged as 8b1cd83. F2 decided by the coordinator: InProcess is a documented trusted harness entry; the test now asserts the binary routes no view, with story:conformance-gate in its message; the specification fix is story:view-tenant-scope.

```findings
[
{"file":"crates/a2a-server/src/lib.rs","line":264,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"F1: InProcess had no Access and returned the extended card to a caller presenting no token"},
{"file":"crates/a2a-server/src/ports.rs","line":1358,"category":"integrity","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"F2: InProcess::call runs the TaskPushNotificationConfigs view, which has no tenant scope; the binary routes no view (story:view-tenant-scope)"},
{"file":"crates/a2a-conformance/src/server.rs","line":414,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"F3: removing GetTaskPushNotificationConfig's tenant guard left the gate green, every cross-tenant Get scenario going unsupported at its Create arrangement"}
]
```
