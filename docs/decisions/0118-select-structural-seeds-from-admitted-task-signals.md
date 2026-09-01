# ADR-0118: Select Structural Seeds from Admitted Task Signals

- Status: Implemented in the core engine; provider-free utility comparison and
  external protocol integration remain gated
- Date: 2026-09-01
- Decider: Aaron Boldt through the active evaluation roadmap continuation
- Related PRD: [Deterministic Structural-Seed Selection PRD](../product/deterministic-structural-seed-selection-prd.md)
- Architecture: [Deterministic Structural-Seed Selection ARD](../architecture/deterministic-structural-seed-selection-ard.md)

## Context

Graft demonstrates the value of querying a precomputed repository structure,
while LeanCTX demonstrates the value of delivering small reversible views on
demand. Impresari already owns a snapshot-bound typed graph, exact-source
verification, bounded packets, opaque evidence handles, and process-local
sessions. Its immediate gap is narrower: the product cannot choose a graph
start node from an ordinary task without caller assistance.

Allowing the evaluator to supply a start node would contaminate the comparison.
Adding progressive delivery first would make it difficult to tell whether any
gain came from better evidence selection or from a different interaction
protocol.

## Decision

First add a pure deterministic selector that maps only admitted exact path and
code-identifier task signals to at most one node in the existing validated
graph. Reuse the existing structural query and packet path. Ambiguity or no
match produces explicit fallback, not guessed traversal.

Keep the first increment inside the existing engine API with a caller-supplied
validated graph. Do not yet change MCP startup, structural-worker custody, the
independent evaluator protocol, or provider execution.

## Consequences

- Structural context becomes product-selected rather than evaluator-selected.
- Exact anchors stay ahead of structural neighbors and remain recoverable.
- Ambiguous common symbol names do not cause broad traversal.
- The first increment can prove utility without adding a new process or public
  protocol identity.
- Real external use still requires a later graph-lifecycle and product-bundle
  identity decision.
- Progressive pull/hybrid remains later and can reuse existing handles and
  session budgets if static structural selection passes its utility gate.

## Rejected alternatives

- Copy Graft's complete architecture: Impresari already has a verified graph,
  capability-relative reads, and stricter packet provenance.
- Add LeanCTX-style progressive delivery first: it confounds selection quality
  with delivery strategy.
- Treat lexical terms as graph seeds: generic prose creates ambiguous broad
  traversal.
- Let the evaluator choose the node: that leaks an evaluation oracle into the
  product arm.
- Build a graph automatically in MCP now: worker identity, custody, cold-start
  cost, and protocol accounting require separate evidence.

## Revisit triggers

Revisit before multi-seed traversal, reverse edges, depth greater than one,
fuzzy names, Unicode normalization, learned ranking, external graph startup,
durable sessions, or progressive context replacement.
