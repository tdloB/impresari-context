# ADR-0120: Bind Structural Runtime to Trusted MCP Startup

- Status: Accepted; implementation gated on hosted completion of ADR-0119
- Date: 2026-09-01
- Decider: Aaron Boldt through the active evaluation roadmap continuation
- Related PRD: [Trusted MCP Structural Lifecycle PRD](../product/trusted-mcp-structural-lifecycle-prd.md)
- Architecture: [Trusted MCP Structural Lifecycle ARD](../architecture/trusted-mcp-structural-lifecycle-ard.md)

## Context

ADR-0118 added product-owned deterministic seed selection and ADR-0119 requires
provider-free proof that its evidence is mechanically useful. The remaining
external gap is lifecycle custody: normal MCP does not prepare a graph, while
the legacy structural request lets the caller provide a graph and start node.
That makes graph work invisible, permits adapter-side evidence selection, and
does not bind cached parser responses to the exact worker executable.

Graft demonstrates the useful pattern of a local deterministic graph refreshed
from content-addressed source state before repository queries. LeanCTX
demonstrates reversible progressive disclosure. The former is the immediate
need; the latter should follow only after static structural delivery is
measurable.

## Decision

Add an opt-in trusted MCP startup tuple for the pinned structural worker,
prepare one snapshot-bound graph before MCP readiness, and automatically route
ordinary profile/query builds through the core seed selector when that runtime
is present. Keep the MCP tool list and admitted request shape equal between
ordinary and structural processes.

Include the worker executable SHA-256 in the internal worker request and parser
cache identity. Return a non-authoritative lifecycle receipt and cumulative
read telemetry with each context result. Structural intent fails closed; it
never silently degrades to the ordinary arm.

## Consequences

- The product, not the evaluator, owns graph construction and seed choice.
- Cold graph reads and launch latency become visible in the treatment result.
- Parser cache entries cannot cross an executable-digest change.
- Existing MCP clients remain unchanged unless they explicitly add the trusted
  startup tuple.
- Static structural packets may still increase bytes or fail to improve an
  agent; no provider-effect claim follows from this decision.
- Progressive disclosure becomes a smaller later delivery problem rather than
  a replacement architecture.

## Rejected alternatives

- Keep caller-supplied graphs for evaluation: creates oracle and accounting
  confounders.
- Add a treatment-only MCP tool: changes the available tool surface between
  arms.
- Build graphs in the independent adapter: moves product work outside product
  telemetry and trust boundaries.
- Reuse parser cache by declared versions only: does not bind results to the
  immutable worker binary.
- Adopt Graft's LLM summary layer now: adds provider cost and a new correctness
  surface before deterministic delivery is proven.
- Adopt LeanCTX proxy compression or durable memory now: changes context and
  session semantics beyond the observed structural problem.

## Revisit triggers

Revisit before changing worker protocol compatibility, graph persistence,
startup budgets, refresh cadence, admitted edge kinds, long-lived MCP source
refresh, progressive delivery, or any evaluator baseline/treatment contract.
