# CI-3a explicit delivery-intent contract verification

- Status: passed locally; hosted release verification pending
- Observed: 2026-08-26
- Governing records: [CI-3a PRD](../product/client-integration-l3-delivery-intent-prd.md),
  [CI-3a architecture record](../architecture/ci-3a-delivery-intent-ard.md), and
  [ADR-0046](../decisions/0046-explicit-guided-delivery-intent-contract.md)

## Evidence

The in-process reference adapter accepts only the versioned `reference` /
`process_local` / `prepare` identity with explicit one-delivery consent. The
intent must include bounded identifiers, a canonical conservative budget, a
UTC timestamp, and caller-declared workspace and snapshot identities.

Before planner construction, the adapter checks the current engine snapshot
using a deterministically derived audit event. A malformed static intent never
reaches the engine. A stale snapshot returns a visible `no_delivery` receipt
and does not invoke the planner with the caller-declared event identifier.

For an accepted intent, `packet_bytes` are the exact canonical bytes returned
by the shared `context_core::packet_bytes` function for the same direct
planner packet. The receipt binds client, scope, client version, lifecycle,
request/event, workspace, packet, plan, snapshot, and policy identities. It
always reports `client_io_performed: false` and `authority_added: false`.

The versioned JSON contract has strict unknown-field rejection for the intent,
receipt, and result shapes. It requires a paired prepared result and byte
field, and rejects any receipt that asserts added authority.

## Local commands

```text
ruby scripts/check-contracts.rb
cargo test -p context-conformance --test schema_conformance
cargo test -p context-adapters
```

These commands passed on 2026-08-26. CI-3a does not contact, configure, or
deliver data to any client. It does not promote any client integration level.
