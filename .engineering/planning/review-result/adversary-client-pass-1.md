---
format: aep.planning-md/3
id: review-result:adversary-client-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:client
relations:
- reviews: story:client
revision: 1
---
Pass on wave/4-int 6fb9943 (unit e0fc75a); cases in commit 45794a3 (crates/a2a-client/tests/adv_*.rs), 44 to 73 executed, 8 red. Every finding fixed in 72233ed, merged as 199f4eb.

```findings
[
{"file":"crates/a2a-client/src/transport.rs","line":286,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"C1: a 1xx interim answer (100, 103) is taken as the final answer, so the 200 after it is never read"},
{"file":"crates/a2a-client/src/transport.rs","line":389,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"C2: chunked trailer lines count against no bound; a server kept the client reading 64 MiB of trailers"},
{"file":"crates/a2a-client/src/transport.rs","line":82,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"C3: segment leaves . and .. unencoded, so a tenant or id of .. becomes a dot segment"},
{"file":"crates/a2a-client/src/transport.rs","line":52,"category":"judgement","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"C4: the refusal of a base URL with user information quotes the whole URL, password included"},
{"file":"crates/a2a-client/src/transport.rs","line":294,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"C5: Content-Length +N is accepted, and of two differing Content-Length headers the first is used"},
{"file":"crates/a2a-client/src/events.rs","line":449,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"C6: a Content-Length stream that ends between events, short of its length, ends cleanly"},
{"file":"crates/a2a-client/src/lib.rs","line":247,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"C7: an empty task or config id is sent as an empty path segment instead of being refused"},
{"file":"crates/a2a-client/src/events.rs","line":447,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"C8: event-stream lines are split on LF only, so CR-only line breaks read as one endless line"},
{"file":"crates/a2a-client/tests","line":1,"category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"C9: the unit's 44 cases stayed green under ten mutants; the adv_* guards now catch each one"}
]
```
