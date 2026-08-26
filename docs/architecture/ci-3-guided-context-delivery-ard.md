# CI-3 Guided Context Delivery — Architecture Requirements and Design

- Status: Approved for implementation
- Date: 2026-08-25
- Governing product record: [CI-3 PRD](../product/client-integration-l3-guided-context-delivery-prd.md)
- Governing decision: [ADR-0042](../decisions/0042-planner-backed-guided-context-delivery.md)

## Architectural objective

Provide an optional delivery adapter that transports one already-authorized
deterministic context packet into a client lifecycle surface. The adapter is a
thin boundary around the planner; it must never become a second retriever,
prompt proxy, policy engine, source reader, or client controller.

## Required components

```text
explicit user enablement
        |
        v
owned adapter configuration ----> source-free adapter validator
        |                                  |
        v                                  v
explicit delivery intent ----------> capability/policy gateway
                                           |
                                           v
                                  deterministic context planner
                                           |
                                           v
                              immutable packet + plan/coverage record
                                           |
                                           v
                                client-specific delivery serializer
                                           |
                                           v
                              documented client lifecycle surface
```

The client-specific serializer receives only the immutable serialized packet
and its delivery receipt. It does not receive workspace access, credentials,
shell access, an MCP configuration writer, or a free-form prompt.

## Delivery-intent contract

An intent must contain exactly these validated, caller-declared values:

- client, scope, adapter version, and documented lifecycle point;
- supported task profile and query;
- authorized workspace/snapshot identity;
- policy profile and hard resource budget;
- explicit one-delivery consent and request/event identifiers.

The adapter rejects an absent consent, unknown profile, unsupported
client/version/scope, stale snapshot, policy/budget mismatch, malformed input,
or any intent that asks it to infer task state. It may return only a stable
no-delivery receipt in those cases.

## Invariants

1. Delivery is disabled until a user explicitly installs an owned adapter.
2. Planner output is produced before serialization; adapters cannot alter
   evidence selection, packet bytes, redactions, plan ID, reason codes,
   coverage, or omission record.
3. A delivery receipt binds packet ID, plan ID, snapshot ID, policy ID,
   client/scope/version, lifecycle point, and outcome (`delivered`,
   `unavailable`, `rejected`, or `degraded`).
4. No packet is retained beyond the existing packet/session lifetime.
5. Failure has no retry loop, background process, network fallback, source
   mutation, client-state mutation, or hidden alternate delivery path.
6. Disable/removal affects only the exact owned adapter artifact.

## Verification requirements

- Direct planner packet and adapter-delivered bytes are identical for the same
  declared intent and snapshot.
- Fixtures cover success, unavailable lifecycle surface, disabled adapter,
  stale snapshot, redaction preservation, hard-budget exhaustion, malformed
  input, client rejection, and exact removal.
- Every check proves source-workspace immutability and absence of network,
  process, shell, environment, credential, and background authority.
- Real-client evidence is recorded separately per official client/version/OS;
  a conversational tool call is not a deterministic conformance substitute.

## Initial implementation order

1. Define and validate the client-neutral intent and source-free receipt.
2. Prove planner/serializer packet equivalence using a reference adapter.
3. Add one client adapter only after its official lifecycle surface, consent
   boundary, and exact-removal behavior are admitted.
4. Keep every other client at manual MCP/no-delivery until its own evidence is
   complete.
