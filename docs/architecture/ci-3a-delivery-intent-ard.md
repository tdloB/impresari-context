# CI-3a Explicit Delivery-Intent Contract — Architecture Requirements and Design

- Status: Approved for implementation
- Date: 2026-08-25
- Governing product record: [CI-3a PRD](../product/client-integration-l3-delivery-intent-prd.md)
- Governing decision: [ADR-0046](../decisions/0046-explicit-guided-delivery-intent-contract.md)

## Flow

```text
strict declared intent -> fixed identity/consent/static validation
                                      |
                                      v
                     workspace/snapshot identity verification
                                      |
                                      v
                            shared deterministic planner
                                      |
                                      v
                    immutable canonical bytes + source-free receipt
```

The adapter receives no filesystem path, client configuration, credentials,
environment, network handle, hook callback, or raw client prompt. The planner
remains the only component selecting evidence.

## Contract invariants

1. An intent is `deny_unknown_fields`, names a workspace/snapshot identity,
   and contains an explicit `consent=true`.
2. Client/scope/version/lifecycle are exact reference values; unknown values
   return an unavailable/rejected result before packet construction.
3. The adapter first verifies the caller-declared snapshot against the
   current in-session snapshot using a derived audit event. A stale, missing,
   or workspace-mismatched identity produces no delivery and never invokes the
   planner with the caller's event identifier.
4. The result holds the exact `ContextPacket` returned by the shared planner
   and its canonical bytes; serialization does not transform, summarize,
   redact, or augment it.
5. The receipt binds client/scope/version/lifecycle and request/event identity,
   names its prepared/no-delivery outcome and packet identity fields, and never
   exposes source text or adds authority.
6. No result performs client delivery; a client-specific adapter is a separate
   later boundary and may consume only this immutable result.

## Verification

- Compare canonical packet serialization from direct planner and reference
  adapter calls byte-for-byte.
- Exercise consent, identity, version, query, budget, snapshot, and
  unknown-field rejection; prove no engine call on static pre-validation
  failures and no planner call for a stale snapshot.
- Assert every response/receipt declares no client I/O and no added authority.
