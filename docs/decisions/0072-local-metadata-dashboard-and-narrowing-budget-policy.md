# ADR-0072: Use a loopback metadata dashboard and narrowing-only budget policy

- Status: Accepted; staged implementation
- Date: 2026-08-29
- Related PRD: [Local dashboard and budget control PRD](../product/local-dashboard-budget-control-prd.md)
- Related architecture: [Local dashboard and budget control ARD](../architecture/local-dashboard-budget-control-ard.md)

## Context

Operators need visible feedback about context activity and resource pressure.
The existing metadata-first audit events and hard request budgets provide a
bounded foundation, but a dashboard creates an HTTP/browser surface and budget
control can become an authority-expansion path if it is not strictly ordered
below existing limits. The core also explicitly forbids a dashboard beyond the
local trust boundary by default.

## Decision

Add a foreground, loopback-only local dashboard that reads only
validated audit metadata and a versioned local budget policy that can deny or
narrow requests but can never increase a governing limit.

- Serve bundled assets with an ephemeral process-local access capability,
  strict same-origin/CSRF/content-security controls, bounded streaming, and no
  outbound network, telemetry, persistence, or automatic browser launch.
- Display metadata and aggregates only. Do not expose source paths, names,
  queries, excerpts, packets, prompts, responses, credentials, or environment.
- Compute effective budgets as the field-wise minimum across immutable engine
  maxima, authorized policy, the local budget policy, and caller request.
- Make the engine—not the browser—the authority that validates policy identity
  and enforces effective limits.
- Use preview/apply/remove, optimistic concurrency, atomic exact-owned storage,
  deterministic precedence, receipts, and explicit rollback.
- Keep remote, hosted, organizational, billing, telemetry, and source-viewing
  dashboards outside this decision.

## Consequences

- Operators gain continuously updating local visibility and enforceable local
  ceilings without exporting repository-derived data.
- The project assumes a local HTTP/browser attack surface and must maintain
  strict XSS, CSRF, DNS-rebinding, resource, and lifecycle tests.
- Budget controls can reduce availability or evidence completeness by design;
  receipts and audit events must make the governing policy and reductions
  explicit.
- The dashboard is not a policy authoring authority, agent orchestrator, cloud
  control plane, or compatibility claim with another product.
- Any future remote access requires a new architecture, identity/tenancy model,
  threat model, retention policy, and external-data authorization.

## Rejected alternatives

- A remotely reachable server violates the local-default boundary.
- A dashboard that reads packets or source creates unnecessary sensitive-data
  duplication.
- Browser-selected arbitrary limits can conflict with caller and engine policy.
- A long-lived daemon increases background and HTTP authority without being
  necessary for an explicit local session.
- Provider token/cost estimates are not deterministic model-neutral hard budget
  units and cannot replace the existing serialized-byte authority.

## Acceptance record

The founder accepted this ADR on 2026-08-30. Implementation is staged so the
pure narrowing and metadata-projection boundary becomes independently testable
before storage or HTTP is added. Live browser rehearsal uses only synthetic
metadata and disposable local state. External exposure remains outside this
decision and requires a new founder-approved architecture and data boundary.
