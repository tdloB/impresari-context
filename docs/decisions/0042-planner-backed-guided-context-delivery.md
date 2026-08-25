# ADR-0042: Planner-backed guided context delivery

- Status: Accepted for implementation
- Date: 2026-08-24
- Scope: CI-3 client lifecycle adapters for deterministic packet delivery

## Decision

Build guided delivery only as a disabled-by-default adapter over the existing
deterministic context planner. A delivery adapter must accept an explicit,
authorized task intent and provide a byte-equivalent, snapshot-bound packet or
a visible no-delivery result. It cannot infer intent from conversation or
repository content.

## Rationale

Native lifecycle delivery can make evidence available at the moment it is most
useful, but it is also the point at which an integration could turn into hidden
prompt injection or unbounded context collection. Binding delivery to an
explicit planner contract makes the operation inspectable and reversible.

## Constraints

- Adapters are client/scope/version specific, are enabled only after explicit
  previewed installation, and use documented lifecycle extension points.
- They carry an exact packet, not an opaque summary or model-generated
  substitute. Packet identity, coverage, reasons, omissions, and redactions
  remain visible.
- No lifecycle surface means no delivery integration; degrade to manual MCP.
- No network, provider proxy, shell hook, repository execution, persistent
  memory, background polling, automatic profile choice, or trust/approval
  mutation is permitted.
- L3 evidence requires positive and no-delivery equivalence tests; a live
  conversational smoke cannot replace deterministic adapter validation.

## Consequences

The product can meet the useful guided-delivery tier of adjacent products while
preserving the central evidence and authority model. CI-4 maintenance is a
separate admission because it introduces freshness/lifecycle observations.

## References

- [CI-3 guided delivery PRD](../product/client-integration-l3-guided-context-delivery-prd.md)
- [Deterministic Context Planner PRD](../product/phase-3-deterministic-context-planner-prd.md)
- [Client Integration Depth Roadmap](../product/client-integration-roadmap.md)
