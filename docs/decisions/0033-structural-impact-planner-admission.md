# ADR-0033: Structural impact-planner admission

- Status: Accepted and implemented in PR #49
- Date: 2026-08-24
- Scope: First Phase 4 impact-evidence slice

## Decision

Connect the deterministic planner to an explicitly supplied, already validated
structural graph only when that graph is bound to the current authorized
workspace snapshot. The adapter may perform bounded graph traversal using
declared start-node, edge-kind, depth, node, edge, and packet-byte limits. It
must report graph identity, query identity, provenance, reason codes,
confidence/resolution, truncation, and unknowns.

`structural_relationship` changes from unavailable to available only for this
validated adapter invocation. All other profiles and invocations retain the
existing explicit unavailable status. Change-set, associated-test,
configuration-to-code, convention, exemplar, and incremental-update evidence
remain unavailable.

## Constraints

- The adapter uses the existing audited `StructureQuery` gateway and canonical
  graph contract; it does not add a second graph or hidden relationship model.
- It accepts no graph unless integrity validation and workspace-snapshot
  equality both succeed.
- It adds no Git, compiler, language-server, build, package, process,
  environment, network, test-runner, or source-write authority.
- It preserves confirmed, heuristic, unresolved, unsupported, and truncated
  states rather than turning them into impact claims.

## Consequences

The product gains useful bounded structural impact evidence while keeping the
planner deterministic, reviewable, and evidence-grade. Revision/diff, test,
and convention capabilities remain separate admission decisions because their
evidence source and failure modes differ materially.

## References

- [Structural Impact Planner delivery record](../product/phase-4-structural-impact-planner-prd.md)
- [ADR-0024: Deterministic Context Planner](0024-deterministic-context-planner.md)
- [ADR-0010: Structural worker protocol and isolation](0010-structural-worker-protocol-and-isolation.md)
- [ADR-0012: Context plans, consumer adapters, and fallback](0012-context-plans-consumer-adapters-and-fallback.md)
