# ADR-0029: Progressive client integration depth and consent

- Status: Accepted
- Date: 2026-08-24
- Scope: Client setup, guidance, packet delivery, and lifecycle claims

## Context

ADR-0018 establishes truthful client classifications, read-only diagnostics,
and a documentation/template-first connection-kit boundary. The roadmap now
needs the useful depth publicly demonstrated by Graft and LeanCTX: native
setup, agent guidance, and where safe, deliberate context delivery.

These features cross third-party configuration and prompt/lifecycle boundaries.
Implicit convenience behavior would create hidden state and weaken the
evidence, workspace-isolation, and authority guarantees.

## Decision

Adopt the L0–L4 model in the Client Integration Depth Roadmap. L1 is the
first-class managed-connection claim. L2 guidance and L3 planner-backed
delivery are separate opt-in client claims; L4 requires stable client lifecycle
support and ongoing version/OS evidence.

A live conversational tool-use test is valuable evidence that a client can use
an integration, but is not deterministic proof that a model will choose it on
every run.

## Mandatory Controls

- External client configuration/artifact changes require explicit user action,
  dry-run preview, exact target/value disclosure, and narrow owned removal.
- Never silently modify a shell profile, global hook, repository instructions,
  or client configuration.
- Guidance/delivery artifacts are versioned, minimal, and ownership marked;
  repository content cannot change their policy.
- Guided delivery is disabled by default and exposes profile, budget, packet
  identity, provenance, redactions, omission reasons, and a no-delivery
  fallback.
- No level adds source mutation, arbitrary repository execution,
  provider-traffic proxying, durable-memory promotion, agent routing, or
  undisclosed network authority.
- Unsupported lifecycle capability degrades to the lower level or fails closed
  with a source-free explanation.

## Relationship to ADR-0018

ADR-0018 remains the Phase 0 foundation. This ADR supersedes only its blanket
deferral of later configuration-writing, instruction-file, and hook-adjacent
work: those actions are permitted solely for an approved L2–L4 milestone under
the controls above. It does not authorize implementation without the relevant
roadmap milestone, conformance evidence, and explicit user approval.

## Consequences

- The project can meet user expectations for leading context-tool integration
  depth without opaque automation.
- Each client/version/OS surface needs separate release evidence.
- Some clients may remain at L1/L2 when no safe documented lifecycle surface
  exists.

## References

- [Client Integration Depth Roadmap](../product/client-integration-roadmap.md)
- [Revised Product Roadmap](../product/revised-product-roadmap.md)
- [ADR-0018](0018-first-class-client-integration-and-compatibility-contract.md)
