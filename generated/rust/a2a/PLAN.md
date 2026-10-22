<!--
  generated from a2a v1
  model digest 340c397069089debcfcb2cf241e311c06d40ac443ecea2a196508e49c5fb6859
  contract digest f01a7ba8cee02bc352a5a026f681d10fa8fa5237c90807e3a38f323f46cb67ea
  do not edit: regenerate with `ess synthesize --layout crate`
-->
# Synthesis plan — a2a v1

Scope: `component-skeletons`, laid out as `crate`, planned by `ess-synth`. Regenerate with `ess synthesize --layout crate`.

94 capabilities: **84 generated**, **10 obligations**, **0 refused**. An obligation is yours to implement against its contract; a refusal is a fact about this synthesis scope, not about the specification.

## Generated

| capability | source |
| --- | --- |
| domain type | `a2a.discovery.APIKeySecurityScheme` |
| domain type | `a2a.discovery.AgentCapabilities` |
| domain type | `a2a.discovery.AgentCard` |
| domain type | `a2a.discovery.AgentCardSignature` |
| domain type | `a2a.discovery.AgentExtension` |
| domain type | `a2a.discovery.AgentInterface` |
| domain type | `a2a.discovery.AgentProvider` |
| domain type | `a2a.discovery.AgentSkill` |
| domain type | `a2a.discovery.AuthorizationCodeOAuthFlow` |
| domain type | `a2a.discovery.ClientCredentialsOAuthFlow` |
| domain type | `a2a.discovery.DeviceCodeOAuthFlow` |
| domain type | `a2a.discovery.HTTPAuthSecurityScheme` |
| domain type | `a2a.discovery.ImplicitOAuthFlow` |
| domain type | `a2a.discovery.MutualTlsSecurityScheme` |
| domain type | `a2a.discovery.OAuth2SecurityScheme` |
| domain type | `a2a.discovery.OAuthFlows` |
| domain type | `a2a.discovery.OpenIdConnectSecurityScheme` |
| domain type | `a2a.discovery.PasswordOAuthFlow` |
| domain type | `a2a.discovery.SecurityRequirement` |
| domain type | `a2a.discovery.SecurityScheme` |
| domain type | `a2a.discovery.StringList` |
| domain type | `a2a.tasks.Artifact` |
| domain type | `a2a.tasks.AuthenticationInfo` |
| domain type | `a2a.tasks.ContextId` |
| domain type | `a2a.tasks.Message` |
| domain type | `a2a.tasks.Part` |
| domain type | `a2a.tasks.PushNotificationConfig.State` |
| domain type | `a2a.tasks.PushNotificationConfigId` |
| domain type | `a2a.tasks.Role` |
| domain type | `a2a.tasks.SendMessageConfiguration` |
| domain type | `a2a.tasks.Task.State` |
| domain type | `a2a.tasks.TaskId` |
| domain type | `a2a.tasks.TaskPushNotificationConfig` |
| domain type | `a2a.tasks.TaskSnapshot` |
| domain type | `a2a.tasks.TaskState` |
| domain type | `a2a.tasks.TaskStatus` |
| entity lifecycle | `a2a.tasks.PushNotificationConfig` |
| entity lifecycle | `a2a.tasks.Task` |
| command contract | `a2a.discovery.GetExtendedAgentCard` |
| command contract | `a2a.tasks.CancelTask` |
| command contract | `a2a.tasks.CompleteTask` |
| command behaviour | `a2a.tasks.CompleteTask` |
| command contract | `a2a.tasks.CreateTaskPushNotificationConfig` |
| command contract | `a2a.tasks.DeleteTaskPushNotificationConfig` |
| command contract | `a2a.tasks.FailTask` |
| command behaviour | `a2a.tasks.FailTask` |
| command contract | `a2a.tasks.GetTask` |
| command contract | `a2a.tasks.GetTaskPushNotificationConfig` |
| command contract | `a2a.tasks.RejectTask` |
| command behaviour | `a2a.tasks.RejectTask` |
| command contract | `a2a.tasks.RequireAuth` |
| command behaviour | `a2a.tasks.RequireAuth` |
| command contract | `a2a.tasks.RequireInput` |
| command behaviour | `a2a.tasks.RequireInput` |
| command contract | `a2a.tasks.SendMessage` |
| command contract | `a2a.tasks.SendStreamingMessage` |
| command contract | `a2a.tasks.StartWorking` |
| command behaviour | `a2a.tasks.StartWorking` |
| command contract | `a2a.tasks.SubscribeToTask` |
| event type | `a2a.tasks.PushNotificationConfigCreated` |
| event type | `a2a.tasks.PushNotificationConfigDeleted` |
| event type | `a2a.tasks.TaskCreated` |
| event type | `a2a.tasks.TaskStatusUpdated` |
| error type | `a2a.discovery.ExtendedAgentCardNotConfigured` |
| error type | `a2a.discovery.UnsupportedOperation` |
| error type | `a2a.tasks.ContentTypeNotSupported` |
| error type | `a2a.tasks.ContextMismatch` |
| error type | `a2a.tasks.ExtensionSupportRequired` |
| error type | `a2a.tasks.PushNotificationNotSupported` |
| error type | `a2a.tasks.TaskNotCancelable` |
| error type | `a2a.tasks.TaskNotFound` |
| error type | `a2a.tasks.TaskStateConflict` |
| error type | `a2a.tasks.UnsupportedOperation` |
| error type | `a2a.tasks.VersionNotSupported` |
| view type | `a2a.tasks.PushNotificationConfigs` |
| view query | `a2a.tasks.PushNotificationConfigs` |
| view type | `a2a.tasks.TaskPushNotificationConfigs` |
| view type | `a2a.tasks.Tasks` |
| view query | `a2a.tasks.Tasks` |
| actor grants | `a2a.discovery.Client` |
| actor grants | `a2a.tasks.Agent` |
| actor grants | `a2a.tasks.Client` |
| component port | `a2a-agent` |
| component transport | `a2a-agent` |

## Ports — yours to provide

What the specification fully determines is generated; what it cannot determine is an obligation. A generated command behaviour or view query reads and writes through the ports below, and they are yours to provide: synthesis generates each port's contract and never an implementation of one, so where instances live stays your decision.

| port | what it answers |
| --- | --- |
| storage | one per entity a generated behaviour or query reads or writes: the instance stored under an identity; storing, replacing and removing one; and every stored instance, in the order the store keeps them |
| context | where a generated behaviour asks it: the caller's attributes, every identity and value the specification says the implementation assigns, and whether each `external:` branch is taken |

## Obligations — yours to implement

| capability | source | why not generated | contract |
| --- | --- | --- | --- |
| command behaviour | `a2a.discovery.GetExtendedAgentCard` | kept an obligation by a typed response (`response:`) | given `a2a.discovery.GetExtendedAgentCard` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `not-supported` externally decided (the public agent card does not declare capabilities.extendedAgentCard), error `a2a.discovery.UnsupportedOperation`; `not-configured` externally decided (the agent declares the capability but has no extended card configured), error `a2a.discovery.ExtendedAgentCardNotConfigured`; `returned` otherwise |
| command behaviour | `a2a.tasks.CancelTask` | kept an obligation by a typed response (`response:`) | given `a2a.tasks.CancelTask` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `not-found` for an identity no record carries, error `a2a.tasks.TaskNotFound`; `not-cancelable-now` externally decided (the agent cannot cancel the task at its current stage), error `a2a.tasks.TaskNotCancelable`; `canceled` otherwise, takes `cancel` of `a2a.tasks.Task`, emits `a2a.tasks.TaskStatusUpdated`; `not-cancelable` from a state no declared move starts in, error `a2a.tasks.TaskNotCancelable`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired` |
| command behaviour | `a2a.tasks.CreateTaskPushNotificationConfig` | kept an obligation by a typed response (`response:`) | given `a2a.tasks.CreateTaskPushNotificationConfig` input, decide and enact exactly one outcome. Selection precedence: on commands with `when_related:`, check `existing_instance` then `exists: false` before input-guarded refusals; choose the first declared input refusal whose guard holds; then check addressed-row existence (`unknown_instance`, and `existing_instance` on commands without `when_related:`); then the held state (`when_subject_state` and `when_subject`), with `wrong_state` only if the selected branch moves from a state the row does not hold; then accepting and external branches in declaration order. An accepting branch that moves nothing answers in every state. Related-presence predicates do not precede input-guarded refusals. Declared outcomes (declaration order, not selection precedence): `task-not-found` when no `a2a.tasks.Task` carries the identity `input.task_id` names, error `a2a.tasks.TaskNotFound`; `created` otherwise, creates `a2a.tasks.PushNotificationConfig`, emits `a2a.tasks.PushNotificationConfigCreated`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired`; `push-not-supported` externally decided (the agent card does not declare capabilities.pushNotifications), error `a2a.tasks.PushNotificationNotSupported` |
| command behaviour | `a2a.tasks.DeleteTaskPushNotificationConfig` | kept an obligation by `external:` beside `when_related:` in one command | given `a2a.tasks.DeleteTaskPushNotificationConfig` input, decide and enact exactly one outcome. Selection precedence: on commands with `when_related:`, check `existing_instance` then `exists: false` before input-guarded refusals; choose the first declared input refusal whose guard holds; then check addressed-row existence (`unknown_instance`, and `existing_instance` on commands without `when_related:`); then the held state (`when_subject_state` and `when_subject`), with `wrong_state` only if the selected branch moves from a state the row does not hold; then accepting and external branches in declaration order. An accepting branch that moves nothing answers in every state. Related-presence predicates do not precede input-guarded refusals. Declared outcomes (declaration order, not selection precedence): `task-not-found` when no `a2a.tasks.Task` carries the identity `input.task_id` names, error `a2a.tasks.TaskNotFound`; `already-gone` for an identity no record carries; `deleted` otherwise, deletes `a2a.tasks.PushNotificationConfig`, emits `a2a.tasks.PushNotificationConfigDeleted`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired`; `push-not-supported` externally decided (the agent card does not declare capabilities.pushNotifications), error `a2a.tasks.PushNotificationNotSupported` |
| command behaviour | `a2a.tasks.GetTask` | kept an obligation by a typed response (`response:`) | given `a2a.tasks.GetTask` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `not-found` for an identity no record carries, error `a2a.tasks.TaskNotFound`; `found` otherwise, preserves `a2a.tasks.Task`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired` |
| command behaviour | `a2a.tasks.GetTaskPushNotificationConfig` | kept an obligation by a typed response (`response:`) | given `a2a.tasks.GetTaskPushNotificationConfig` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `not-found` for an identity no record carries, error `a2a.tasks.TaskNotFound`; `other-task` when the existing subject's stored fields satisfy `task_id != input.task_id`, error `a2a.tasks.TaskNotFound`; `found` otherwise, preserves `a2a.tasks.PushNotificationConfig`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired`; `push-not-supported` externally decided (the agent card does not declare capabilities.pushNotifications), error `a2a.tasks.PushNotificationNotSupported` |
| command behaviour | `a2a.tasks.SendMessage` | kept an obligation by a typed response (`response:`) | given `a2a.tasks.SendMessage` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `task-not-found` externally decided (message.taskId names no task the caller can access (section 3.4.2)), error `a2a.tasks.TaskNotFound`; `task-terminal` externally decided (message.taskId names a task in Completed, Failed, Canceled or Rejected), error `a2a.tasks.UnsupportedOperation`; `context-mismatch` externally decided (message.contextId differs from the contextId of the task message.taskId names), error `a2a.tasks.ContextMismatch`; `content-type-not-supported` externally decided (a part's media type is not supported by the agent), error `a2a.tasks.ContentTypeNotSupported`; `continued` externally decided (message.taskId names a non-terminal task, and the agent takes the message into it); `direct-message` externally decided (the agent answers with a message and creates no task); `task-created` otherwise, creates `a2a.tasks.Task`, emits `a2a.tasks.TaskCreated`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired` |
| command behaviour | `a2a.tasks.SendStreamingMessage` | kept an obligation by a typed response (`response:`) | given `a2a.tasks.SendStreamingMessage` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `task-not-found` externally decided (message.taskId names no task the caller can access (section 3.4.2)), error `a2a.tasks.TaskNotFound`; `task-terminal` externally decided (message.taskId names a task in Completed, Failed, Canceled or Rejected), error `a2a.tasks.UnsupportedOperation`; `context-mismatch` externally decided (message.contextId differs from the contextId of the task message.taskId names), error `a2a.tasks.ContextMismatch`; `content-type-not-supported` externally decided (a part's media type is not supported by the agent), error `a2a.tasks.ContentTypeNotSupported`; `continued` externally decided (message.taskId names a non-terminal task, and the agent takes the message into it); `direct-message` externally decided (the agent answers with a message and creates no task); `task-created` otherwise, creates `a2a.tasks.Task`, emits `a2a.tasks.TaskCreated`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired`; `streaming-not-supported` externally decided (the agent card does not declare capabilities.streaming), error `a2a.tasks.UnsupportedOperation` |
| command behaviour | `a2a.tasks.SubscribeToTask` | kept an obligation by a typed response (`response:`) | given `a2a.tasks.SubscribeToTask` input, decide and enact exactly one outcome. Declared outcomes (declaration order, not selection precedence): `not-found` for an identity no record carries, error `a2a.tasks.TaskNotFound`; `task-terminal` when the existing subject's stored fields satisfy `state in [Completed, Failed, Canceled, Rejected]`, error `a2a.tasks.UnsupportedOperation`; `subscribed` otherwise, preserves `a2a.tasks.Task`; `version-not-supported` externally decided (the A2A-Version service parameter names a version the agent does not support), error `a2a.tasks.VersionNotSupported`; `extension-required` externally decided (the agent card marks an extension required and the request does not declare it), error `a2a.tasks.ExtensionSupportRequired`; `streaming-not-supported` externally decided (the agent card does not declare capabilities.streaming), error `a2a.tasks.UnsupportedOperation` |
| view query | `a2a.tasks.TaskPushNotificationConfigs` | kept an obligation by a view parameter (`params:`), which a generated query does not apply | a query answering `a2a.tasks.TaskPushNotificationConfigs` with rows projected from `a2a.tasks.PushNotificationConfig` at `read_your_writes` consistency, containing instances where `task_id == param.task_id` |

## Refused — not represented by this synthesis

| capability | source | stage | why |
| --- | --- | --- | --- |
