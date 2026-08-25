# ADR-0039: Caller-declared convention and exemplar evidence

- Status: Accepted for implementation
- Date: 2026-08-25
- Scope: Final Phase 4 impact-evidence slice

## Decision

Expose convention and exemplar evidence only through a caller-declared,
current-snapshot-verified manifest. A declaration contains bounded opaque
labels and paths with expected content hashes. The engine verifies and recovers
exact current source, then records the declaration identity separately from
observed evidence.

No capability may infer that those examples are representative, preferred,
widely followed, owned, approved, or semantically correct.

## Constraints

- No repository-wide mining, statistical analysis, embeddings, model calls,
  recommendation, ranking, execution, source mutation, Git/history, network,
  compiler, or language-server authority.
- Labels and declarations are untrusted caller assertions; only exact source
  bytes and snapshot/hash membership become evidence.
- Declarations are canonically ordered, deduplicated, bounded, and bound to
  their current snapshot and deterministic plan identity.

## Consequences

The product can deliver useful explicit examples to agents and reviewers
without crossing into opaque intelligence or governance. Inference-based
convention discovery remains a separate future proposal.

## References

- [Convention and exemplar evidence delivery record](../product/phase-4-convention-exemplar-evidence-prd.md)
- [ADR-0034: Declared change-set packets](0034-declared-change-set-packets.md)
- [ADR-0024: Deterministic context planner](0024-deterministic-context-planner.md)
