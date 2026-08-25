# ADR-0046: Explicit Guided-Delivery Intent Contract

- Status: Accepted
- Date: 2026-08-25
- Deciders: Impresari Context maintainers
- Related: [CI-3a PRD](../product/client-integration-l3-delivery-intent-prd.md), [ADR-0042](0042-planner-backed-guided-context-delivery.md)

## Decision

Implement CI-3 incrementally. The first capability is a client-neutral,
strictly typed delivery-intent adapter that returns the existing planner packet
and a source-free receipt. It has no client I/O surface and accepts no inferred
conversation or repository state. Client/version/scope/lifecycle identity is a
fixed allowlist in the reference adapter until a separate client admission
decision adds a documented lifecycle surface.

## Consequences

- Packet equivalence and consent enforcement are testable before any client
  lifecycle integration exists.
- The reference adapter cannot claim delivery to a client; it reports only a
  deterministic prepared/no-delivery result.
- New client identities or lifecycle delivery require narrow follow-on records,
  not a generic execution or hook framework.
