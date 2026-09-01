# ADR-0121: Use Bounded Progressive Structural Disclosure

- Status: Accepted for implementation after the ADR-0120 provider-free MCP gate
- Date: 2026-09-01
- Decider: Aaron Boldt through the accepted evaluation-integrity roadmap
- Related PRD: [Progressive Structural Disclosure PRD](../product/progressive-structural-disclosure-prd.md)
- Architecture: [Progressive Structural Disclosure ARD](../architecture/progressive-structural-disclosure-ard.md)

## Context

ADR-0120 establishes product-owned graph preparation and static structural
delivery through the real MCP lifecycle. Its exact provider-free comparison
showed two different results: digest-bound warm reuse reduced graph preparation
from 645 milliseconds to 6 milliseconds, while eager structural delivery
increased the initial response from 4,149 bytes to 8,063 bytes on the controlled
fixture. The graph/cache lifecycle should be retained, but delivering every
selected exact excerpt before the consumer requests it is not an efficient
general architecture.

Graft's applicable contribution is deterministic content-addressed structural
state and reuse. LeanCTX's applicable contribution is reversible progressive
disclosure. Impresari already has the stronger primitives needed to implement
that idea without importing a proxy or memory architecture: current snapshots,
typed graph facts, exact evidence identities, source revalidation, bounded
expansion, process-local sessions, and product-owned read telemetry.

## Decision

Add an opt-in `progressive_structural` MCP delivery mode over the existing
trusted graph lifecycle. Keep the `context_build` input and advertised tool
definitions equal across ordinary, eager, and progressive processes.
Progressive build returns a compact deterministic map whose opaque handles are
owned by an already-open process-local session. Add always-advertised bounded
lookup and exact-expansion tools to resolve only those handles.

Every call reauthorizes and revalidates workspace, snapshot, graph, path,
content, span, policy, and session ownership as applicable. One monotonic
session ledger limits operations, items, exact bytes, serialized bytes, reads,
repeated reads, and elapsed time. Exact source is returned only through the
existing evidence-expansion authority.

## Consequences

- Initial structural context can be smaller without discarding recoverability.
- The consumer chooses which product-provided handles to expand; it cannot
  supply arbitrary graph nodes, paths, spans, or source hashes.
- More tool round trips are possible, so tool-call count and cumulative latency
  become first-class evaluation outcomes.
- Process-local state grows modestly and remains bounded, consumer scoped, and
  non-durable.
- Ordinary and eager delivery remain compatibility and evaluation controls.
- Provider-free mechanics must pass before another paid benchmark.
- No claim follows about model behavior, correctness, token use, cost, or
  latency until a separately frozen controlled study is officially graded.

## Rejected alternatives

- Keep eager structural packets only: contradicted by the observed initial-byte
  amplification and offers no way to defer exact source.
- Remove structural evidence: discards verified relationships and the successful
  warm-cache lifecycle rather than fixing delivery.
- Generate summaries: adds a provider/correctness surface and can sever exact
  recoverability.
- Add embeddings or semantic vector retrieval: adds dependencies, nondeterminism,
  storage, and evaluation confounders before the bounded graph path is tested.
- Adopt a prompt proxy or rewrite conversation history: changes client and
  model semantics beyond the repository-evidence product boundary.
- Persist handles or memory: requires encryption, retention, revocation,
  multi-user authorization, crash recovery, and secure deletion decisions.
- Let the evaluator choose graph nodes or expansion paths: creates an oracle and
  moves product policy outside product telemetry.

## Revisit triggers

Revisit if the provider-free gate fails its initial-byte, anchor-preservation,
exact-recovery, read-amplification, tool-parity, or cumulative-budget checks; if
a controlled pilot shows correctness regression or excessive expansion calls;
or before adding durable sessions, generated context, embeddings, automatic
refresh, model-specific policy, remote transport, or server-initiated actions.
