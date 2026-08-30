# v0.2 Independent Security Review Readiness

- Date: 2026-08-30
- Decisions: ADR-0083 and ADR-0084
- Prepared-scope state: immutable historical planning evidence
- Candidate-scope state: `manual_review_required`
- Scheduling state: final candidate frozen; attributable human review required
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

This checkpoint now evaluates the refreshed candidate scope at product commit
`1a9923c0e5d671581f6b7da3bc4248b604971d63`. Exact candidate workflow run
`33323269945` passed on all three release targets. The scope identity is
`aa96ad705335d86948ad61810c39f06e5901faf1b0e9ab2e7f437d17f1acd9d3`.
This is still not the review: automated and AI-assisted work cannot satisfy the
release gate.

## Scheduling amendment

ADR-0084 retains the manual release gate but backlogs reviewer engagement until
the final v0.2.0 release candidate is frozen. Run:

```sh
ruby scripts/check-independent-security-review-backlog.rb
```

The backlog check remains historical proof that development was permitted before
freeze. That phase is complete. The release workflow now requires both the
immutable candidate scope and a separate attributable review record bound to
its digest. Until that record exists, release readiness, tag creation,
publication, production support, and real-analyzer authorization remain false.
