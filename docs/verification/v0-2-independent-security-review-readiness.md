# v0.2 Independent Security Review Readiness

- Date: 2026-08-30
- Decision: ADR-0083
- Current state: `manual_review_required`
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

This checkpoint prepares the review; it is not the review. Automated and
AI-assisted work cannot change the current state. An attributable independent
human report bound to the exact product commit is the next manual artifact.
