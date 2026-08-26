# CI-3a Explicit Delivery-Intent Contract — Architecture Requirements and Design

- Status: Approved for implementation
- Date: 2026-08-25
- Governing product record: [CI-3a PRD](../product/client-integration-l3-delivery-intent-prd.md)
- Governing decision: [ADR-0046](../decisions/0046-explicit-guided-delivery-intent-contract.md)

## Flow

```text
strict declared intent -> fixed identity/consent validation -> shared planner
                                                             |
                                                             v
                                                 immutable packet + receipt
```

The adapter receives no filesystem path, client configuration, credentials,
environment, network handle, hook callback, or raw client prompt. The planner
remains the only component selecting evidence.

## Contract invariants

1. An intent is `deny_unknown_fields` and contains an explicit `consent=true`.
2. Client/scope/version/lifecycle are exact reference values; unknown values
   return an unavailable/rejected result before packet construction.
3. The result holds the exact `ContextPacket` returned by the shared planner;
   serialization does not transform, summarize, redact, or augment it.
4. The receipt names its prepared/no-delivery outcome and all packet identity
   fields without exposing source text or adding authority.
5. No result performs client delivery; a client-specific adapter is a separate
   later boundary and may consume only this immutable result.

## Verification

- Compare canonical packet serialization from direct planner and reference
  adapter calls byte-for-byte.
- Exercise consent, identity, version, query, budget, and unknown-field
  rejection; prove no planner call on pre-validation failures.
- Assert every response/receipt declares no client I/O and no added authority.
