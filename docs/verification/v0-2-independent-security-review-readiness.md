# v0.2 Independent Security Review Readiness

- Date: 2026-08-30
- Decisions: ADR-0083 and ADR-0084
- Prepared-scope state: `manual_review_required`
- Scheduling state: development continues; review deferred to candidate freeze
- Review gate satisfied: No
- Release ready: No
- Publication authorized: No

Run:

```sh
ruby scripts/check-independent-security-review-readiness.rb
```

The check reproduces the manual-review state and deterministic scope-change,
missing-scope, non-independent-review, blocking-finding, and unsupported-version
failures. It verifies that the scope identity is pinned and that every result
denies source, process, network, credential, tag, publication, and risk-
acceptance authority.

This checkpoint prepared the initial review scope; it is not the review.
Automated and AI-assisted work cannot satisfy the release gate. ADR-0084 now
controls when the exact candidate scope is refreshed and sent to a reviewer.

## Scheduling amendment

ADR-0084 retains the manual release gate but backlogs reviewer engagement until
the final v0.2.0 release candidate is frozen. Run:

```sh
ruby scripts/check-independent-security-review-backlog.rb
```

The backlog check proves that roadmap development may continue while review
admission, release readiness, tag creation, publication, production support,
and real-analyzer authorization remain false. The prepared scope is historical
planning evidence; any descendant candidate requires a refreshed exact scope.
The release workflow independently enforces that boundary with
`enforce-v0-2-independent-review-release-gate.rb`; the current prepared scope is
an explicit failing test case for v0.2.0 publication.
