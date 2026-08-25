# Phase 4 structural-impact planner evidence

- Status: Implementation candidate; full local release gate passed, hosted release gates pending
- Governing records: [Phase 4 structural-impact planner delivery record](../product/phase-4-structural-impact-planner-prd.md) and [ADR-0033](../decisions/0033-structural-impact-planner-admission.md)

## Implemented evidence

`build_profiled_structural_context` accepts an explicitly supplied structural
graph only through the existing audited `StructureQuery` gateway. The gateway
rejects a stale graph before packet construction. The resulting deterministic
plan binds the graph traversal result, requested edge kinds, and a canonical
query identity; it marks `structural_relationship` available only for that
invocation.

For each returned edge, the engine recovers its source span with
`evidence_for_span`, which rechecks the current authorized snapshot artifact
and source hash. The packet therefore contains regular exact-source evidence
with extraction method `structural_graph_edge`, not an inferred impact claim.
Traversal unknowns and limits remain explicit. Change-set, associated-test,
configuration-to-code, convention, exemplar, orientation, and incremental
update evidence remain unavailable.

The CLI exposes the same adapter as `context profile-structure-build`; MCP
extends `context_build` with the optional `structural_graph`, `start_node`, and
`edge_kinds` profile form. Both adapters deserialize the canonical graph
contract and delegate all validation and policy decisions to the shared engine.

## Local verification

- Engine tests establish byte-stable plan and packet identities across equal
  declared inputs; assert structural coverage, traversal binding, and exact
  source recovery; and prove a stale graph fails closed before evidence
  recovery.
- The complete `./scripts/check.sh` release gate passes: repository/security
  boundary policy, contracts, schemas, evaluations, cache restart, frozen SBOM,
  formatting, lint, unit/integration/binary-smoke tests, and doc tests.
- The deterministic-plan schema permits the optional, snapshot-bound structural
  traversal binding while preserving the prior profile form.

## Deliberate limits

The adapter does not build graphs, run a worker, invoke Git, inspect revision
diffs, execute tests, resolve compilers or projects, add network/process
authority, or write source. It does not establish reachability, runtime impact,
dispatch, aliases, inheritance, test association, or a change set.
