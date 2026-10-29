---
format: aep.planning-md/3
id: review-result:plan-critics-wave-5
kind: review-result
status: active
title: Plan-critic panel on wave 5 stories
relations:
- reviews: story:view-tenant-scope
- reviews: story:testkit-wire-events
revision: 1
---
# Plan-critic panel on wave 5 stories

```findings
[
{"file":".engineering/planning/story/testkit-wire-events.md","line":null,"category":"acceptance","severity":"blocker","verdict":"needs-revision","origin":"introduced","message":"item 2 let a listed scenario come out unsupported, so the story could close with no new pass; fixed: the listed scenarios must pass, a scenario leaves the list only by a new revision naming the reason"},
{"file":".engineering/planning/story/testkit-wire-events.md","line":null,"category":"acceptance","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"item 1 bundled three derivations and the unsupported rule; fixed: items 1-3, one stub-endpoint test each"},
{"file":".engineering/planning/story/testkit-wire-events.md","line":null,"category":"acceptance","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"baseline 177/7/170 had no source; fixed: measured on wave/5-int with a2a-testkit run, 183 scenarios, 7 passed, 176 unsupported, 118 at SendMessage task-created"},
{"file":".engineering/planning/story/view-tenant-scope.md","line":null,"category":"acceptance","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"docs item named no check; fixed: item 5 names the grep that finds nothing"},
{"file":".engineering/planning/story/view-tenant-scope.md","line":null,"category":"acceptance","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"scenario item unclear and zero rows not exact; fixed: item 4 names tenant-view on both assertions and counts at_most 0"},
{"file":".engineering/planning/story/view-tenant-scope.md","line":null,"category":"acceptance","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"items bundled outcomes and the unsupported.tsv clause was not checkable; fixed: split into specification, derived files, server, scenarios, documentation; unsupported.tsv unchanged"},
{"file":".engineering/planning/story/testkit-wire-events.md","line":null,"category":"scope","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"stream events derived but no stream scenario promised; fixed: Outcome names streams and item 6 names SendStreamingMessage task-created and SubscribeToTask subscribed"},
{"file":".engineering/planning/story/testkit-wire-events.md","line":null,"category":"scope","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"silently amended story:testkit acceptance 1; fixed: Context names the amendment and keeps adversary-testkit-pass-1 F1 and F2, held by items 3 and 7"},
{"file":".engineering/planning/story/testkit-wire-events.md","line":null,"category":"scope","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"depends_on view-tenant-scope had no reason; fixed: the dependency is the suite it runs (183 scenarios)"},
{"file":".engineering/planning/story/view-tenant-scope.md","line":null,"category":"scope","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"HTTP clause forbade A's own route; fixed: under any other tenant prefix, or none"},
{"file":".engineering/planning/story/view-tenant-scope.md","line":null,"category":"scope","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"unsupported.tsv delta unbounded; fixed: unchanged, and sec_tenant_guards stays red without each guard"}
]
```
