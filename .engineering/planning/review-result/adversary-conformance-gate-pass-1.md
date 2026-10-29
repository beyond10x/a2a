---
format: aep.planning-md/3
id: review-result:adversary-conformance-gate-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:conformance-gate
relations:
- reviews: story:conformance-gate
revision: 1
---
Pass on wave/4-int 5abc010 (unit 3c654bb); cases in commit b74324e (crates/a2a-conformance/tests/adv_*.rs), 31 to 43 executed, 6 red; 35 of 51 planted server mutants passed the gate. Fixed in d1755ca, merged as 8b1cd83: J1-J4, J6-J8 in code and five authored scenarios; J5 is pre-existing and is story:view-tenant-scope.

```findings
[
{"file":"crates/a2a-conformance/src/server.rs","line":67,"category":"judgement","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"J1: ACROSS_TENANTS made all four scenarios reading the TaskPushNotificationConfigs view unsupported before the read"},
{"file":"crates/a2a-server/src/lib.rs","line":345,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"J2: InProcess::issue duplicated steps::run/issue, so the gate certified a copy the binary never runs"},
{"file":"crates/a2a-conformance/src/interpret.rs","line":168,"category":"judgement","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"J3: a scenario reported only its first unsupported reason"},
{"file":"crates/a2a-conformance/src/server.rs","line":110,"category":"mutant","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"J4: the target configured the extended card equal to the public card"},
{"file":"crates/a2a-server/src/ports.rs","line":1358,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"J5: the TaskPushNotificationConfigs view has no tenant input, so a client reads another tenant's push configs in process; the binary routes no view (story:view-tenant-scope)"},
{"file":"crates/a2a-server/src/ports.rs","line":1306,"category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"J6/J7: no scenario read the Tasks view across tenants or under a status filter"},
{"file":"crates/a2a-server/src/ports.rs","line":914,"category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"J8: no scenario bounded GetTask history_length or the config list page_size"}
]
```
