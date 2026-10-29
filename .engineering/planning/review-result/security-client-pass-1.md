---
format: aep.planning-md/3
id: review-result:security-client-pass-1
kind: review-result
status: active
title: Security review pass 1 on story:client
relations:
- reviews: story:client
revision: 1
---
Pass on wave/4-int 6fb9943 (unit e0fc75a); cases in commit 60e80bd (crates/a2a-client/tests/sec_*.rs), 44 to 56 executed, 7 red. Every finding fixed in 72233ed, merged as 199f4eb. S1 decided by the coordinator: the token goes with every operation under the base path, never with the public card; the assertion was rewritten with story:client in its message.

```findings
[
{"file":"crates/a2a-client/src/lib.rs","line":474,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"S1: the bearer token was sent with every operation, the public card at the host root included"},
{"file":"crates/a2a-client/src/lib.rs","line":456,"category":"integrity","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"S2: a tenant or id of ., .. or empty becomes a dot or empty path segment, so a server-chosen id can redirect the next request on the same host"},
{"file":"crates/a2a-client/src/lib.rs","line":140,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"S3: a bearer token that is not an RFC 6750 b64token is sent instead of refused"},
{"file":"crates/a2a-client/src/transport.rs","line":170,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"S4: the timeout bounds each read, not the whole call, so a trickling server holds a call indefinitely"},
{"file":"crates/a2a-client/src/transport.rs","line":389,"category":"boundary","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"S5: the chunked trailer section is read without a line or byte limit"},
{"file":"crates/a2a-client/src/transport.rs","line":51,"category":"integrity","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"S6: refusing a base URL repeats it in full, password included"},
{"file":"crates/a2a-client/src/transport.rs","line":40,"category":"integrity","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"S7: refusing a base URL with a control byte writes the raw CR/LF into the error text"}
]
```
