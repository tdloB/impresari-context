# ADR-0085: Continue Roadmap Development Before Independent Review

- Status: Accepted
- Date: 2026-08-30
- Decider: Aaron Boldt
- Supersedes: ADR-0084's final-candidate scheduling state, not its review requirement

## Context

ADR-0084 deferred reviewer engagement until the proposed `v0.2.0` candidate
was frozen. That candidate was prepared, but the broader analyzer-confinement
roadmap remains incomplete and no attributable independent reviewer is
currently available. Reviewing the intermediate evidence-only candidate now
would not admit or cover the later macOS VM, Windows native confinement, or
real-analyzer code and could require a second substantial review.

## Decision

Continue accepted roadmap development before engaging the independent human
reviewer. Treat the candidate frozen at
`1a9923c0e5d671581f6b7da3bc4248b604971d63` and scope digest
`aa96ad705335d86948ad61810c39f06e5901faf1b0e9ab2e7f437d17f1acd9d3` as
immutable historical evidence, not the final release candidate.

The independent-review requirement is deferred, not waived. No protected
release tag, publication, production analyzer support, real analyzer, or risk
acceptance may occur until a later final candidate is frozen, its exact review
scope is regenerated, and an attributable independent human report passes the
existing finding policy.

## Consequences

- Roadmap implementation may continue without a reviewer engaged now.
- Any later production-code change prevents the historical candidate scope
  from satisfying a release gate.
- The next final candidate may retain version `0.2.0`; no version is reserved
  merely because an earlier candidate used it.
- Existing release automation remains fail-closed until a fresh admitted
  review record is bound to the final candidate.
- Review effort can cover the feature-complete candidate once instead of
  certifying an intermediate security foundation and then repeating most of
  the work.

## Alternatives

- Publish the intermediate candidate after review: rejected because no reviewer
  is currently available and the principal analyzer boundary remains pending.
- Remove the review requirement: rejected because the security-sensitive
  release trigger remains valid.
- Treat automated or AI review as independent review: rejected by ADR-0083.

## Revisit Trigger

Freeze a new review scope when the intended release contents are complete and
no accepted production-code increment remains before publication.
