---
format: aep.planning-md/3
id: story:testkit-wire-events
kind: story
status: draft
title: a2a-testkit observes task events over the wire
summary: The wire target derives task event claims from the answers and streams that carry them, so more than 7 of 177 scenarios run against an endpoint.
relations:
- decomposes: epic:a2a-delivery
- depends_on: story:testkit
revision: 1
---
## Outcome

`a2a-testkit run` observes more of the suite over the wire: of 177 scenarios, 7 pass against
a2a-server and 170 are `unsupported`, 116 of them because the setup step's `TaskCreated` event is
not observable over HTTP+JSON.

## Acceptance

The wire target derives `TaskCreated`, `TaskStatusUpdated` and `TaskArtifactUpdated` claims from
the answer that carries them (the task in a `SendMessage` answer, the events of a
`SendStreamingMessage` or `SubscribeToTask` stream) where the binding defines that
correspondence, and stays `unsupported` where it does not; against a2a-server, at least the
SendMessage, GetTask, CancelTask and push-config outcome scenarios pass, and the planted-defect
tests still go red.
