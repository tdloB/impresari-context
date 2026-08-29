# HRA-3 Coverage And Assessment Corpus

- Scope: the complete ADR-0073 HRA-3 increment.
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

## Synthetic normalized analyzer results

Completed coverage can enter only through an ADR-0013
`normalized-extension-output` record with `untrusted_derived_data` trust and an
exact analyzer artifact digest and envelope digest. Its payload must validate
against `analyzer-result-envelope.schema.json` and match one unavailable
requirement's snapshot, identifier, capability, and complete sorted artifact
set. Completion and freshness timestamps are canonical UTC; the application
time must be at or after completion and strictly before `fresh_until`.

Only categorical artifact hash, category, severity, confidence, and bounded
method identifiers are admitted. Raw detection descriptions, source bytes,
commands, paths, provider metadata, and analyzer-controlled limitations are not
retained. Accepted findings are marked `derived` and
`untrusted_derived_data`, with exact analyzer and ruleset digests. The coverage
ledger is re-identified after completion and the assessment independently
checks every derived finding against completed coverage provenance.

## Adversarial and resource review

Tests cover deterministic grouping, ordering, schema conformance, forged
inventory rejection, coverage-state laundering rejection, mandatory-analysis
visibility, exact finding propagation, the no-requirement complete case, and
the frozen output ceiling. Additional tests cover the closed envelope schema,
exclusive freshness boundary, snapshot/requirement/capability/artifact
mismatch, authority claims, duplicate derived identities, exact analyzer and
ruleset provenance, and completed assessment integration.
