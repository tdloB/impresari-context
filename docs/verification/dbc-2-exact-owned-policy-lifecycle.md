# DBC-2 exact-owned policy lifecycle and runtime composition

- Status: Implemented locally; hosted acceptance pending
- Decision: [ADR-0072](../decisions/0072-local-metadata-dashboard-and-narrowing-budget-policy.md)
- Scope: Local policy state and existing engine admission only

## Implemented boundary

- `PolicyStore` uses a private, explicit state root with an exact owner marker.
  Current and one previous canonical policy are written together as one bounded
  atomic state object, so update, removal, and rollback have no cross-file
  partial-commit state.
- Apply, remove, and rollback are preview-only unless the CLI receives the
  existing explicit `--apply` write flag. Every mutation requires the exact
  expected current policy identity and revision (or an explicit `absent`).
- Canonical state is protected against unknown fields, modified identities,
  permissive roots, symlinks, oversized bytes, concurrent writers, stale
  revisions, and non-monotonic updates.
- `LocalEngine::open_with_budget_policy_store` requires the policy state root to
  be disjoint from source and cache. It reloads and revalidates current policy
  at every capability admission.
- The effective field-wise minimum replaces the caller budget used by snapshot,
  search, structural, context-build, evidence, validation, and handoff paths.
  A limit is recorded as `limited`; a denial is recorded as `denied` before the
  safe policy error is returned.

## Evidence

- Store tests cover preview without writes, install, monotonic update, remove,
  rollback, stale expectations, modified bytes, symlink replacement, canonical
  identity, and exact retained state.
- Engine tests prove a one-file policy actually narrows snapshot discovery and
  its audit limits, then prove a policy applied after engine open is reloaded,
  denies the next search, and records the denial.
- CLI tests cover the preview/apply/inspect/remove/rollback lifecycle without
  touching source or cache state.
- Closed schemas and fixtures cover store-state and mutation receipts, including
  rejection of authority-bearing fields.

## Explicit non-claims

DBC-2 adds no HTTP listener, SSE stream, UI asset, browser token, daemon, remote
access, telemetry, source display, policy synchronization, or dashboard
availability claim. Those remain DBC-3 and DBC-4.
