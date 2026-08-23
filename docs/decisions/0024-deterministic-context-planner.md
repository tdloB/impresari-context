# ADR-0024: Deterministic Context Planner

- Status: Accepted; implementation pending
- Date: 2026-08-23
- Scope: Phase 3 deterministic evidence-selection layer

## Decision

Add a small deterministic context planner after the Phase 1 and Phase 2
coverage and client-admission work reaches its defined depth. The planner takes
a declared task profile, query, exact snapshot, policy, budget, and supported
structural evidence; it produces an explicit retrieval plan, per-item reason
codes, coverage and omission reporting, budget-exclusion reasons, and exact
packet identity.

It begins with orientation, implementation, bug-investigation, change-review,
security-review, test-selection, and configuration-change profiles. It can use
only admitted evidence classes and must make unavailable inputs visible.

## Constraints

- No model call, prompt interpretation, agent routing, execution, approval,
  durable memory, hidden ranking, or new authority is introduced.
- Equivalent declared inputs must yield equivalent plans, reasons, omissions,
  and identities.
- A selection or omission is valid only when recoverable to exact evidence or a
  stable declared rule and budget reason.

## Consequences

The planner becomes an auditable intelligence layer rather than a LeanCTX-style
agent-governance system. It improves naturally as language and configuration
evidence becomes available without overstating unsupported semantics.

## References

- [Phase 3 deterministic context planner PRD](../product/phase-3-deterministic-context-planner-prd.md)
- [Revised Product Roadmap](../product/revised-product-roadmap.md)
- [ADR-0001: Independent core and thin adapters](0001-independent-core-and-thin-adapters.md)
- [ADR-0012: Context plans, consumer adapters, and fallback](0012-context-plans-consumer-adapters-and-fallback.md)
