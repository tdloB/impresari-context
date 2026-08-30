# ADR-0083: v0.2 Independent Security Review Gate

- Status: Accepted; release gate retained, scheduling amended by ADR-0084
- Date: 2026-08-30
- Deciders: Founder and maintainers
- Related: ADR-0015, ADR-0017, ADR-0074, ADR-0082, ADR-0084

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

ADR-0084 subsequently backlogs reviewer engagement until the final v0.2.0
release-candidate scope is frozen. That scheduling change does not waive this
gate: roadmap development may continue, but no v0.2.0 tag or publication may
occur until a refreshed exact-source review is admitted.

## Candidate freeze

The final product candidate is frozen at
`1a9923c0e5d671581f6b7da3bc4248b604971d63`. Candidate workflow run
`33323269945` passed on all three release targets and produced exact archive,
manifest, and workflow-artifact identities. The refreshed immutable scope is
`release-review/v0.2.0-independent-review-candidate-scope.json`; the original
prepared scope remains unchanged historical evidence.

The human report is recorded separately from the immutable scope. A final tag
may descend from the reviewed product only through the scope's exact allowed
review/release metadata paths, while the release workflow, gate, and schema
remain hash-pinned. This avoids the impossible requirement that a commit contain
a review record which already names that commit as its own reviewed source.
