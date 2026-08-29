# ADR-0036: Declared associated-test evidence

- Status: Accepted and implemented in PR #57
- Date: 2026-08-24
- Scope: Third Phase 4 impact-evidence slice

## Decision

Implement `associated_test` planner evidence as caller-declared source/test
pairs verified only against the current authorized snapshot. Each endpoint has
a bounded native-relative path and expected current content hash. The engine
verifies both endpoints before recovering exact current-source evidence.

The resulting association is named `declared_associated_test`; it is neither a
discovered association, a test execution record, coverage proof, behavioral
claim, or test-selection recommendation. `associated_test` becomes available
only for this adapter. Ordinary test-selection profiles retain their explicit
unavailable state.

## Constraints

- The engine must not execute tests or other processes, infer naming or
  framework conventions, access network/package/build state, or write source.
- All pairs must bind to one current snapshot, be canonically ordered and
  deduplicated, reject self-association, and participate in the association
  identity.
- Current exact-source evidence and the caller's association assertion remain
  distinct in the plan and packet contract.
- No association is silently accepted, dropped, or upgraded into coverage.

## Consequences

The product gains a useful, reviewable test-context handoff primitive while
preserving the evidence-grade boundary. Test discovery, execution, coverage,
and inference remain future separately admitted capabilities.

## References

- [Declared associated-test evidence delivery record](../product/phase-4-declared-associated-test-evidence-prd.md)
- [ADR-0034: Declared change-set packets](0034-declared-change-set-packets.md)
- [ADR-0024: Deterministic context planner](0024-deterministic-context-planner.md)
