# Local real-time dashboard and budget control architecture

- Status: Accepted; staged implementation
- Date: 2026-08-29
- Governing PRD: [Local dashboard and budget control PRD](../product/local-dashboard-budget-control-prd.md)
- Governing decision: [ADR-0072](../decisions/0072-local-metadata-dashboard-and-narrowing-budget-policy.md)

## Context

The engine already writes schema-validated metadata-first audit events and
enforces caller-provided hard resource budgets. A useful dashboard can therefore
observe existing metadata rather than inspect source or packet content. Budget
control is more sensitive: a new policy layer must be unable to increase any
existing limit or become a second authorization system.

## Process boundary

Add an optional `dashboard` CLI surface that launches one foreground local
process with four internal components:

1. a read-only bounded audit projection;
2. a deterministic aggregate and stream coordinator;
3. a static-asset and JSON/SSE loopback server; and
4. a narrow budget-policy validator/store using preview/apply/remove semantics.

The process receives explicit audit and policy store capabilities only. It has
no workspace, source, packet, cache, client-home, credential, shell, child
process, or outbound-network capability. UI assets are compiled into the
release and their digests are included in release evidence.

## Local HTTP boundary

- Resolve and bind only an IPv4 or IPv6 loopback address with port `0`; verify
  the actual socket address before announcing readiness.
- Generate 256 bits of process-local randomness. The printed bootstrap URL uses
  a fragment so the token is not sent in the initial HTTP request; bundled
  JavaScript exchanges it once for an HttpOnly, SameSite=Strict, Secure-when-
  available session cookie bound to the process and origin.
- Require exact Host and Origin allowlists, custom anti-CSRF headers for writes,
  JSON content types, small body ceilings, short timeouts, and no CORS.
- Set `default-src 'self'`, deny objects/frames/base URIs/forms, prohibit inline
  script/style, use no-store, and serve no source maps in release builds.
- Keep the server foreground and single-user. It does not auto-open a browser,
  register a service, advertise on the network, or survive its parent session.

## Audit projection and live stream

Read only committed events through the existing audit-store API. Project each
event into a dashboard record containing schema version, opaque IDs, timestamp,
capability, outcome, policy decision, numeric limits, duration, and engine
version. Do not add optional source-bearing fields to the audit schema for UI
convenience.

Maintain bounded in-memory windows and aggregates keyed only by coarse time,
capability, outcome, and operator-owned pseudonymous label. Use a monotonic
stream sequence distinct from audit identity. SSE clients acknowledge the last
sequence; gaps produce a `reset_required` event and a new bounded aggregate
snapshot rather than replaying unbounded history.

Refresh by bounded read-only polling of the audit store. Do not add workspace
watchers or extend audit retention. Polling stops when the foreground server
stops.

## Budget policy layer

Add one canonical `local-budget-policy` schema and exact-owned atomic store.
The engine loads a validated current policy by identity at request admission.
It derives an `EffectiveBudgetDecision` containing:

- every governing policy identity;
- the caller's requested budget;
- the effective field-wise minimum;
- matched deny/ceiling selectors; and
- stable reason codes for every reduction or denial.

The dashboard layer is not authoritative by itself. The engine recomputes and
enforces the same decision from canonical policy bytes; UI previews call the
shared pure evaluator and are rejected if their resulting identity differs.

Policy selectors are enumerated purposes/capabilities already known to the
engine, never free-form paths, consumers, commands, or regexes. Engine hard
maxima remain compiled/release policy. The new layer can only deny or reduce.

## Storage and rollback

- Keep dashboard labels and budget policy in a distinct explicit state root,
  never in source, cache, MCP configuration, or audit rows.
- Write canonical JSON through create-new staging, file sync, no-replace/identity
  comparison, and same-volume atomic rename. Sync the parent directory where
  the safe platform API exposes that durability barrier; on Windows, revalidate
  the directory after the synced-file rename rather than using an unsafe native
  handle.
- Retain one exact previous policy for explicit rollback, with both identities
  in the receipt. Never silently roll back after a valid operator change.
- Unknown or modified ownership markers fail exact removal; return bounded
  manual recovery information without deleting the file.

## Security and evaluation gates

- Threat-model loopback DNS rebinding, local hostile pages, CSRF, XSS, token
  leakage, browser extensions, symlink/path races, audit poisoning, resource
  exhaustion, stale writes, policy ambiguity, and limit-expansion bugs.
- Fuzz schemas, HTTP parsing, filters, aggregate windows, stream resumption, and
  policy evaluation independently.
- Prove there is no outbound socket path and no source-bearing value flow from
  audit/store inputs to any dashboard output.
- Compare dashboard-on and dashboard-off engine results under identical inputs;
  observation must not change packet selection, while an applied budget policy
  may only produce the declared denial or field-wise reduction.

## Alternatives rejected

- **Hosted dashboard:** crosses the local trust boundary and creates retention,
  identity, tenancy, and external-data decisions.
- **Expose packets or queries:** turns observability into a second source-access
  surface and expands breach impact.
- **Let the browser set arbitrary request budgets:** bypasses client contracts
  and makes UI state authoritative.
- **Let policy raise limits:** conflicts with hard resource and authorization
  policy.
- **Persistent dashboard daemon:** adds background authority and a durable HTTP
  attack surface.
- **Copy LeanCTX's UI or receipt model:** violates the original-artifact and
  evidence-bound design requirement; public vision is an influence, not a
  compatibility contract.

## Staged implementation boundary

ADR-0072 was accepted on 2026-08-30. DBC-1 freezes the shared pure evaluator,
closed public contracts, safe metadata projection, deterministic bounded
aggregates, and concurrent read-only audit view before any HTTP listener or
policy write exists. DBC-2 implements the exact-owned atomic policy lifecycle,
admission-time reload, effective-budget enforcement, and audit composition.
DBC-3 adds the foreground loopback server only after those
boundaries pass independently. DBC-4 records the complete synthetic local
rehearsal. A future remote or hosted mode cannot be added by extending this
local server; it requires an independent architecture and decision record.
