# ADR-0034: Declared change-set packets

- Status: Accepted and implemented in PR #56
- Date: 2026-08-24
- Scope: Second Phase 4 impact-evidence slice

## Decision

Implement `change_set` planner evidence as a caller-declared manifest verified
only against the current authorized snapshot. Each entry contains a bounded
relative path and expected current content hash. The engine verifies membership
and hash equality before recovering exact source evidence. The manifest may
carry an optional base revision assertion, which is labeled asserted and may be
compared only to the existing non-mutating repository metadata.

The resulting evidence class is named `declared_change_set`; it is not a
computed diff, historical change set, working-tree report, or proof that a
path differs from the base revision. `change_set` becomes available only for
this adapter. Ordinary profiled context retains its explicit unavailable state.

## Constraints

- The engine must not invoke Git, read revision diffs or objects, inspect the
  working tree, spawn a process, or add network/source-write authority.
- All entries must bind to the same current snapshot, be canonically ordered,
  deduplicated, hard-bounded, and included in the declaration identity.
- Current exact-source evidence and caller assertions must remain separate in
  the packet/plan contract.
- Mismatches and unavailable metadata must be explicit unknowns or safe
  failures; no entry may be silently accepted or silently dropped.

## Consequences

The product can produce useful review-scoped context from a trusted client
selection while preserving its evidence-grade boundary. Full diff semantics,
provider integration, merge-base analysis, and automatic association remain
future separately admitted capabilities.

## References

- [Declared change-set packets delivery record](../product/phase-4-declared-change-set-packets-prd.md)
- [ADR-0033: Structural impact-planner admission](0033-structural-impact-planner-admission.md)
- [ADR-0005: Hashing, serialization, and schema](0005-hashing-serialization-and-schema.md)
