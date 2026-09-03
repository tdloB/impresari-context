# ADR-0119: Require Provider-Free Utility Before External Structural Delivery

- Status: Implemented; local provider-free gate passed, hosted CI required for merge
- Date: 2026-09-01
- Decider: Aaron Boldt through the active evaluation roadmap continuation
- Related PRD: [Provider-Free Structural Utility Gate PRD](../product/provider-free-structural-utility-gate-prd.md)
- Architecture: [Provider-Free Structural Utility Gate ARD](../architecture/provider-free-structural-utility-gate-ard.md)

## Context

ADR-0118 makes structural seed choice deterministic and product-owned. It does
not show that adding graph relationships improves the context packet enough to
justify external graph lifecycle, worker identity, startup, and accounting
work. The earlier paid smoke failures showed the cost of changing provider
mechanics before proving the local product boundary.

## Decision

Freeze and run a provider-free, model-neutral comparison before any external
structural delivery. Compare ordinary and seeded packets on identical tasks and
sources using fresh engines. Require anchor retention, new verified structural
evidence, bounded packet growth, complete product read accounting,
determinism, and source immutability.

A pass permits architecture work on the external graph lifecycle. It does not
authorize model calls, correctness grading, publication, or a performance
claim. LeanCTX-style progressive delivery remains a later independent
decision.

## Consequences

- Structural selection quality is separated from provider and protocol noise.
- Snapshot and exact recovery reads remain visible rather than amortized away.
- A mechanically useful result can still fail to improve an agent; later
  controlled evaluation remains necessary.
- A failed gate sends work back to seed selection, graph resolution, or packet
  ordering instead of expanding the public protocol.

## Rejected alternatives

- Add MCP graph lifecycle immediately: introduces new confounders before local
  utility is known.
- Use token reduction as the local gate: tokenization and agent behavior require
  a provider/model and cannot be inferred from packet bytes alone.
- Use the independent evaluator to choose structural nodes: contaminates the
  treatment with oracle knowledge.
- Start progressive delivery now: confounds evidence selection with delivery
  strategy and session behavior.

## Revisit triggers

Revisit thresholds or fixture composition before changing structural edge
kinds, depth, seed classes, graph versions, language coverage, packet budgets,
telemetry semantics, external delivery, or progressive context behavior.
