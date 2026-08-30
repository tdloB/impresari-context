# ADR-0084: Defer Independent Review to the v0.2 Release Candidate

- Status: Accepted
- Date: 2026-08-30
- Deciders: Founder and maintainers
- Related: ADR-0017, ADR-0083

## Context

ADR-0083 correctly requires an attributable independent human security review
before v0.2.0 can be tagged or published. Its first scheduling text made the
prepared source commit the immediate review target and placed the report before
all later roadmap work. Continuing product development would invalidate that
review target and require avoidable repeat review.

The founder chose to backlog reviewer engagement so accepted roadmap work can
continue without weakening the release-assurance requirement.

## Decision

Treat the independent review as a mandatory release-candidate gate, not a
development gate.

The prepared ADR-0083 scope and digest remain immutable historical evidence of
the review planning completed at commit
`1ed4500a6d3ac4a0d375c62f1c208ba8ddf98d51`. They cannot admit a descendant
release. Before reviewer engagement, the project must freeze the final v0.2.0
product candidate, refresh every affected review artifact, issue a new exact
scope and reviewer brief, and pin their identities.

Roadmap development, ordinary pull requests, tests, documentation, and
non-release evidence may continue. A tag, release publication, v0.2.0
release-ready claim, ADR-0082 production-support admission, or real-analyzer
authorization remains prohibited until the refreshed review is admitted and
all separate release gates pass.

The tracked backlog schedule is deterministic and fail-closed. It may state
that development is not blocked, but it grants no source, process, network,
credential, risk-acceptance, tag, or publication authority.

The release workflow enforces the distinction. Existing v0.1.0 policy remains
intact, while v0.2.0 requires a `review_recorded` scope whose report and reviewed
commit exactly match the protected release tag. Unknown later versions fail
until their review policy is recorded.

## Consequences

- The project avoids commissioning a review of a knowingly moving target.
- Independent review remains mandatory before v0.2.0 tag or publication.
- The original prepared scope is preserved rather than rewritten as if it had
  reviewed future work.
- Product changes accumulate into one refreshed release-candidate assessment.
- A later review may still require remediation and a bounded delta review.
- Ordinary CI can pass with the review backlogged; the release workflow cannot.
