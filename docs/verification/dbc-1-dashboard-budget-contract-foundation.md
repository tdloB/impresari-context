# DBC-1 dashboard and budget contract foundation

- Status: Merged; hosted acceptance passed in PR 152
- Decision: [ADR-0072](../decisions/0072-local-metadata-dashboard-and-narrowing-budget-policy.md)
- Scope: Pure contracts and read-only metadata projection only

## Implemented boundary

- `context-dashboard` owns canonical local-policy compilation, exact policy and
  decision identities, deterministic selector precedence, expiry withdrawal,
  and field-wise minimum evaluation across engine, authorized, local, and caller
  limits.
- The policy accepts only enumerated planner purposes and engine capabilities.
  It has no path, query, regex, executable, client, user, or authorization
  selector.
- `DashboardRecord` is projected only from a revalidated `AuditEvent`. It keeps
  metadata, replaces the opaque workspace identity with a domain-separated
  one-way label, and has no source/query/path/prompt/response field.
- `AuditReader` opens an existing audit database read-only without taking the
  writer lock or creating state. Malformed and future rows are counted as
  unavailable and their raw bytes are withheld.
- Bounded snapshots aggregate only capability, outcome, count, and duration.

## Evidence

- Closed Draft 2020-12 schemas and positive/negative fixtures cover local
  policies, effective decisions, snapshots, path-selector rejection, denied
  decisions carrying no effective budget, and source-field rejection.
- Unit tests cover canonical rule ordering, duplicate-selector rejection,
  resource-profile bounds, ambiguous-rule rejection, deny precedence, expiry,
  deterministic aggregation, metadata-only projection, concurrent read-only
  access, malformed-row withholding, and field-wise monotonic narrowing.

## Explicit non-claims

DBC-1 does not add a CLI command, policy write, engine integration, HTTP
listener, SSE stream, browser UI, daemon, remote access, telemetry, or dashboard
availability claim. Those remain DBC-2 through DBC-4 and cannot inherit
acceptance from this foundation.
