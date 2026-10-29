---
format: aep.planning-md/3
id: story:view-tenant-scope
kind: story
status: active
title: TaskPushNotificationConfigs view scoped to the caller's tenant
summary: The push-config view answers a client only its own tenant's configs; found by wave 4 reviews of story:conformance-gate.
relations:
- decomposes: epic:a2a-delivery
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T14:49:08Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-29T14:49:08Z", actor: "human:timo", revision: 5}
---
## Outcome

The `a2a.tasks.TaskPushNotificationConfigs` view answers a client only the push configs of the
tenant it reads under, so no caller of the specification's surface reads another tenant's webhook
token. A configuration made under tenant A is a row of the view read under A only; one made
without a tenant only of a read without one (A2A v1.0.1 section 13.1), as the `Tasks` view already
filters tasks.

## Acceptance

1. Specification: `ess/domains/tasks.yaml` gives the view an optional `tenant` parameter and
   filters its rows by it with the `Tasks` view's rule (both defined and equal, or both absent);
   `ess specify validate` and `ess specify http validate` pass with ESS 0.75.0, `ess/http.yaml`
   pinning the new digest. The ESS pin does not move.
2. Derived files: `task drift`, `task projections-drift` and `task docs-check` pass on the
   regenerated `generated/` and `website/`.
3. Server: `crates/a2a-server`'s `TaskPushNotificationConfigsQuery` filters rows by the tenant
   parameter, and `crates/a2a-server/tests/view_tenancy.rs` passes: through `InProcess::call` a
   client under another tenant, a different spelling of it, or none reads no row of tenant A's
   task (and a tenant reads no row of a tenant-less task), while A reads its row with its token;
   over HTTP no route under any other tenant prefix, or none, answers A's configuration. On
   `main` (0.5.0) the two `InProcess` cases fail.
4. Scenarios: the two assertions of `ess/scenarios/push-config-view-of-task.yaml` pass
   `tenant: tenant-view`; the new `ess/scenarios/push-config-view-other-tenant.yaml` reads the
   view under `tenant-b` and under no tenant with `counts: {at_most: 0}` and under its own tenant
   with `at_most: 1`. `task conform` passes with `crates/a2a-conformance/unsupported.tsv`
   unchanged, and `crates/a2a-conformance/tests/sec_tenant_guards.rs` still turns the gate red
   for each push-config command without its tenant guard.
5. Documentation: `grep -rn "no tenant scope" AGENTS.md crates/a2a-server/src
   crates/a2a-server/tests` finds nothing; the `crates/a2a-server` row of `AGENTS.md` and the
   `InProcess` docs state the view's tenant rule.

## Context

Found by the wave 4 security review and adversary on story:conformance-gate (F2, J5): through
`InProcess`, a client of no tenant read tenant A's config with its token. Wave 5 confirmed it
against 0.5.0 (`view_tenancy.rs`, 2 cases red) and confirmed that the binary routes no view, so
nothing reached it over HTTP.

ESS gap A stays open and is out of scope: the tenant guard of the push-config commands
(`ess/domains/tasks.yaml`, the `UNMAPPED` markers under Create/Get/DeleteTaskPushNotificationConfig)
cannot be expressed on ESS 0.75.0, so the synthesized cross-tenant scenarios stay `unsupported`.
The guard is enforced in `crates/a2a-server` and held by `tests/tenants.rs`,
`tests/sec_cg_inprocess.rs` and `crates/a2a-conformance/tests/sec_tenant_guards.rs`. With the
view's parameter, synthesis arranges the configurations of the Get and Delete scenarios under
tenants of its own; the in-process target sends those arrangements under the task's tenant, so
the gate keeps measuring both guards.

## Scope

`ess/domains/tasks.yaml`, `ess/http.yaml`, `ess/ess-inputs.yaml`,
`ess/scenarios/push-config-view-*.yaml`, `generated/`, `website/docs/reference/`,
`website/static/downloads/`, `crates/a2a-server/src/{ports.rs,lib.rs}`,
`crates/a2a-server/tests/{view_tenancy.rs,sec_cg_inprocess.rs}`,
`crates/a2a-conformance/src/server.rs`, `crates/a2a-conformance/tests/`, `AGENTS.md`,
`CHANGELOG.md`.
