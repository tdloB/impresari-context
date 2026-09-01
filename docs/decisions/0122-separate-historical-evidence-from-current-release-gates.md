# ADR-0122: Separate Historical Evidence From Current Release Gates

- Status: Proposed; founder approval required before implementation
- Date: 2026-09-01
- Decider: Pending Aaron Boldt decision
- Related PRD: [Historical Release Evidence Gate Separation PRD](../product/historical-release-evidence-gate-separation-prd.md)
- Architecture: [Historical Release Evidence Gate Separation ARD](../architecture/historical-release-evidence-gate-separation-ard.md)

## Context

ADR-0085 declares the first `v0.2.0` candidate immutable historical evidence,
allows accepted roadmap development to continue, and says later production
changes prevent that candidate from satisfying release gates. ADR-0109 also
separates a contract baseline from a later candidate source identity.

The ordinary repository checker still hashes today's direct entrypoint files
against the historical release-identity contract. The first legitimate change
to the packaged MCP entrypoint therefore causes the entire development gate to
fail. Rewriting the frozen hashes would destroy historical integrity. Treating
the failure as optional would weaken the current release gate. The verifier is
answering two different questions with one outcome.

## Proposed decision

Separate source-free historical-evidence integrity from current release-
candidate validation.

Ordinary development validates historical records, fixtures, provenance,
canonical identities, and cross-bindings without claiming that current source
matches them. It returns `historical_not_current` and keeps all current,
release, publication, production, independent-review, and authority fields
false.

Only a separately invoked release gate may validate a newly frozen current
candidate against its exact source archive/revision and artifact/evidence
closure. Release workflows must fail when that current result is absent,
historical, stale, or changed.

## Consequences

- Historical candidate evidence remains byte-for-byte immutable.
- Accepted development can change packaged entrypoints without pretending the
  old candidate is current.
- Ordinary CI can pass honestly while release readiness remains false.
- Release validation remains strict and becomes harder to confuse with a
  source-free metadata check.
- New schemas, receipts, fixtures, workflow checks, and migration tests are
  required.
- A fresh candidate lineage is still required before any release; this decision
  does not create or approve one.

## Alternatives

- Rewrite historical source hashes after each feature: rejected because it
  mutates evidence and relabels new source as an old candidate.
- Skip the failing checker ad hoc: rejected because an unlabeled bypass can
  reach release workflows.
- Block all packaged-source development until release: rejected because it
  contradicts ADR-0085 and the accepted roadmap.
- Fetch the historical commit during every CI run: rejected because it adds
  network/history availability, mutable remote, and shallow-checkout failure
  modes to ordinary validation.
- Treat recorded hashes alone as proof the old source was reproduced now:
  rejected; metadata integrity and byte reproduction are distinct claims.

## Approval requirement

This decision changes a validation rule and release-evidence routing. It may not
be implemented from roadmap continuation alone. Founder approval must identify
ADR-0122 explicitly; no approval for a paid test, ordinary PR, or feature
implementation substitutes for that decision.

## Revisit triggers

Revisit before creating the final candidate lineage, changing historical
records, adding remote archive retrieval, altering release workflow protections,
or accepting a historical receipt in signing, publication, production,
platform admission, or independent review.
