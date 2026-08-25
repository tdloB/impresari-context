# ADR-0038: Explicit incremental structural updates

- Status: Accepted for implementation
- Date: 2026-08-25
- Scope: Fifth Phase 4 impact-evidence slice

## Decision

Admit incremental structural updates only as a caller-declared replacement
manifest bound to a prior validated graph and a newly authorized current
workspace snapshot. The engine verifies every supplied artifact identity and
content hash before replacing structural-file material and producing a new
canonical graph.

The adapter is an explicit one-shot operation. It is not a watcher, polling
agent, Git-diff engine, background indexer, or synchronization service.

## Constraints

- The prior graph and replacement manifest must be snapshot- and
  content-verified; stale, duplicate, malformed, or out-of-budget requests
  fail closed.
- No source is modified and no process, compiler, language server, package
  manager, Git command, or network capability is introduced.
- Removed artifacts are caller-declared and are accepted only when absent from
  the new current snapshot; undeclared changes are never inferred.
- Output preserves deterministic ordering, graph identity, explicit limits,
  and unknowns.

## Consequences

The product can reduce repeated structural work for a caller that already
knows a bounded current change set, while preserving the existing snapshot and
authority boundaries. Automatic change discovery remains out of scope.

## References

- [Incremental structural updates delivery record](../product/phase-4-incremental-structural-updates-prd.md)
- [ADR-0033: Structural impact planner admission](0033-structural-impact-planner-admission.md)
- [ADR-0010: Structural worker protocol and isolation](0010-structural-worker-protocol-and-isolation.md)
