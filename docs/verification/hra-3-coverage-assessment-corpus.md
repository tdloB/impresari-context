# HRA-3 Coverage And Assessment Corpus

- Scope: the first ADR-0073 HRA-3 increment.
- Inputs: an identity-valid HRA-1 inventory, its HRA-2 observation bundle, and
  a caller-supplied canonical UTC generation time.
- Capability boundary: pure record planning and assembly only; no analyzer
  discovery, process execution, network access, upload, policy evaluation, or
  repository execution.

## Deterministic coverage planning

The planner groups exact artifact hashes by the closed analyzer capability IDs
already emitted by HRA-1. It emits one mandatory requirement per capability,
sorts and deduplicates artifact hashes, uses one stable reason rule, and derives
the requirement and coverage identities from canonical structured data.

Because this increment cannot discover or run analyzers, every emitted
requirement is exactly `unavailable` with
`analyzer-execution-not-authorized`. An empty requirement ledger means only
that the bounded inventory requested no analyzer capability; it is not a
malware-free or repository-safety claim.

## Immutable assessment assembly

The assembler verifies the inventory identity, recomputes the exact coverage
plan, checks every finding/evidence/artifact snapshot binding, and derives an
immutable assessment identity. Inventory omissions, execution-surface
exclusions or truncation, and every non-completed mandatory requirement remain
prominent as stable unknowns and force `partial` completeness.

`safety_claimed`, `ordinary_host_execution_authorized`, and `authority_added`
are always false. Finding count does not control coverage: zero findings with a
missing mandatory analysis remains partial.

## Adversarial and resource review

Tests cover deterministic grouping, ordering, schema conformance, forged
inventory rejection, coverage-state laundering rejection, mandatory-analysis
visibility, exact finding propagation, the no-requirement complete case, and
the frozen output ceiling. Completed external analyzer results are deliberately
not representable by this increment; their envelope normalization, provenance,
freshness, and mismatch rules remain the next HRA-3 slice.
