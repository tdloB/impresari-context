# ADR-0083: v0.2 Independent Security Review Gate

- Status: Accepted for review preparation; manual review required
- Date: 2026-08-30
- Deciders: Founder and maintainers
- Related: ADR-0015, ADR-0017, ADR-0074, ADR-0082

## Context

ADR-0017 permits v0.1.0 without an independent human security review but makes
that review mandatory earlier than v1.0 when the project adds materially
higher-risk capability or expands its trust boundary. Current source now has
explicit, consent-gated delivery through external model clients using existing
authentication configuration in place, separate child-process adapters,
application-enforced analyzer supervision, and exact-host OS-isolation
candidates. These boundaries satisfy ADR-0017's earlier-review trigger.

Automated checks, AI-assisted review, hosted rehearsals, and founder approval
are valuable evidence but are not an independent human security review. The
next feature release must not be represented as review-ready merely because
those controls pass.

## Decision

Use v0.2.0 as the recommended next feature-release version and freeze an exact,
attributable independent-review scope before preparing its release tag.

The review target begins at product source commit
`1ed4500a6d3ac4a0d375c62f1c208ba8ddf98d51`. Any later production-code change
invalidates that scope until it is refreshed. Review-only documentation,
evidence intake, and release metadata do not retroactively change the reviewed
product source, but the final release record must disclose any descendant
changes and prove that no product code escaped review.

The reviewer must be a human with relevant security experience, independent of
the implementation, attributable in the returned report, and explicit about
conflicts and limitations. AI may assist the reviewer but cannot be the
reviewer. The report must bind the exact commit and cover every area in the
tracked scope.

Open critical or high findings prevent admission. Medium findings require an
explicit founder disposition; low findings require documentation. Unknown
severity is not accepted. A report may satisfy only the independent-review
gate. It cannot create a tag, publish a release, accept risk, admit Linux
production support, or authorize real analyzers.

## Consequences

- The next feature release remains manual-review-gated.
- The repository provides one exact reviewer brief instead of an informal ask.
- A deterministic evaluator rejects scope drift, missing evidence,
  non-independent review, and blocking findings.
- Publication remains a separate owner and GitHub release-environment action.
- Existing v0.1.0 artifacts and claims are unchanged.
