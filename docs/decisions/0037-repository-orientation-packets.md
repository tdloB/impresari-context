# ADR-0037: Repository-orientation packets

- Status: Accepted for implementation
- Date: 2026-08-24
- Scope: Fourth Phase 4 impact-evidence slice

## Decision

Use the existing validated, snapshot-bound structural repository map as the
sole structural input to a bounded repository-orientation packet adapter. The
adapter may recover exact current-source evidence for selected map entries and
report map limits and unknowns. It must not generate an architectural summary
or infer semantic, runtime, convention, ownership, or importance claims.

## Constraints

- Accept only a current, integrity-validated complete graph through the
  existing audited structure gateway.
- Respect graph/map/item/packet budgets and preserve unknown and truncation
  signals.
- Add no execution, Git, history, network, build, compiler, package,
  language-server, environment, source-write, or client-account authority.

## Consequences

The product gains a useful orientation primitive with recoverable evidence.
Natural-language summaries and inferred architecture remain client work, not
engine evidence.

## References

- [Repository-orientation packets delivery record](../product/phase-4-repository-orientation-packets-prd.md)
- [ADR-0033: Structural impact-planner admission](0033-structural-impact-planner-admission.md)
- [ADR-0024: Deterministic context planner](0024-deterministic-context-planner.md)
