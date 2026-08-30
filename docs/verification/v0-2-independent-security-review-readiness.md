# v0.2 Independent Security Review Readiness

- Date: 2026-08-30
- Decisions: ADR-0083, ADR-0084, and ADR-0085
- Prepared-scope state: immutable historical planning evidence
- Candidate-scope state: immutable historical candidate evidence
- Scheduling state: roadmap development resumed; future final candidate required
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

This checkpoint evaluates the immutable historical candidate scope at product commit
`1a9923c0e5d671581f6b7da3bc4248b604971d63`. Exact candidate workflow run
`33323269945` passed on all three release targets. The scope identity is
`aa96ad705335d86948ad61810c39f06e5901faf1b0e9ab2e7f437d17f1acd9d3`.
This is still not the review, and ADR-0085 prevents this historical scope from
covering later production changes. Automated and AI-assisted work cannot
satisfy the future release gate.

## Scheduling amendment

ADR-0084 retains the manual release gate but backlogs reviewer engagement until
the final v0.2.0 release candidate is frozen. Run:

```sh
ruby scripts/check-independent-security-review-backlog.rb
```

The backlog check remains historical proof that development was permitted before
the first freeze. ADR-0085 resumes that state because the frozen candidate is no
longer final. Before any release, the project must freeze a new exact candidate
scope and bind a separate attributable review record to that new digest. Until
then, release readiness, tag creation, publication, production support, and
real-analyzer authorization remain false.
