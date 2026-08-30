# v0.2 Independent Security Review PRD

## Outcome

Provide an attributable independent human reviewer with a complete, exact, and
bounded assessment package for the next feature release, then admit the review
gate only from a report bound to the reviewed product commit.

ADR-0084 schedules reviewer engagement after the final v0.2.0 product candidate
is frozen. Ongoing roadmap development is not blocked; tagging and publication
remain blocked until the refreshed candidate review is admitted.

## Requirements

1. Pin the intended version, product commit, boundary triggers, review areas,
   governing artifacts, reviewer qualifications, and finding policy.
2. Require examination of workspace/cache isolation, untrusted input, parser
   workers, external-client delivery and consent, credential references,
   managed configuration lifecycle, analyzer supervision/Linux isolation, and
   release supply-chain claim accuracy.
3. Require an attributable report, independence statement, conflict disclosure,
   methodology, limitations, exact commit, findings, and dispositions.
4. Reject non-human or implementation-affiliated review, missing identity,
   scope drift, unknown severity, open critical/high findings, or incomplete
   medium/low dispositions.
5. Keep release readiness, publication, production support, and real-analyzer
   authorization false even after the review gate is satisfied.
6. Re-run normal release-candidate, packaging, provenance, and owner gates after
   review; do not infer them from the report.
7. Preserve the prepared scope as historical planning evidence, require a new
   exact scope after intervening product changes, and never treat the prepared
   commit as review coverage for a descendant release.

## Non-goals

- Selecting or contracting a reviewer.
- Claiming certification, formal verification, penetration-test completeness,
  or absence of vulnerabilities.
- Giving a reviewer credentials, source from private repositories, or authority
  over release state.
- Publishing v0.2.0, creating a tag, changing runtime authority, activating
  Linux production support, or executing a real analyzer.

## Acceptance

- The tracked scope validates against the public schema and has a pinned digest.
- The source-free evaluator returns deterministic manual, changed, missing,
  invalid, and unsupported states.
- Every current state leaves all release and runtime claims false.
- Conformance rejects a manual-review receipt that claims admission or
  publication authority.
- The complete repository gate runs the new evaluator check.
- A separate backlog receipt proves that roadmap development may continue while
  review admission, tagging, publication, production support, and real-analyzer
  authorization remain false.
