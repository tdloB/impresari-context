# Historical Release Evidence Gate Separation PRD

- PRD ID/version: IC-HREG-122 / 1.0.
- Status: Approved for implementation by the founder on 2026-09-01.
- Date: 2026-09-01.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Historical Release Evidence Gate Separation ARD](../architecture/historical-release-evidence-gate-separation-ard.md).
- Proposed decision:
  [ADR-0122](../decisions/0122-separate-historical-evidence-from-current-release-gates.md).

## Problem

ADR-0085 classifies the first `v0.2.0` candidate and its related macOS records
as immutable historical evidence while explicitly allowing accepted roadmap
development to continue. The general repository gate nevertheless re-hashes
today's packaged entrypoint files against an earlier source-free release-
identity contract. Any legitimate later entrypoint change therefore fails the
ordinary development gate even though the roadmap says the old candidate can
no longer satisfy release readiness.

Updating the historical hashes would rewrite evidence. Ignoring the failure
would weaken release controls. Historical metadata validation and current
release-candidate validation need separate, explicit outcomes.

## Outcome

The ordinary repository gate validates that every historical record is
immutable, internally bound, schema valid, provenance complete, and incapable
of claiming current release readiness. It does not claim that today's source
matches an old candidate.

A separate current-candidate release gate remains strict: it binds one exact
current source revision/archive, direct build inputs, artifacts, SBOM,
vulnerability/reproducibility evidence, signatures/distribution evidence when
applicable, and all required human gates. Any source drift fails that release
gate.

## Requirements

1. Preserve every existing historical contract, receipt, profile, fixture,
   source digest, artifact digest, and candidate identity byte-for-byte.
2. Replace no historical identifier and never label a later source tree as the
   old candidate.
3. Split validation into two named modes:
   `historical_evidence_integrity` and `current_release_candidate`.
4. The ordinary development gate runs historical integrity. It validates
   schemas, canonical identities, cross-record bindings, frozen fixture
   digests, provenance, and explicit historical/non-current state without
   comparing old source-input hashes to the working tree.
5. Historical integrity must report that current-source equality,
   release-identity binding, publication, production admission, and independent
   review are false or unverified.
6. An optional deep historical audit may accept an exact local source archive
   whose predeclared digest and revision match the record. It must never fetch,
   mutate the checkout, or silently use current files as substitutes.
7. The current-candidate gate requires a newly frozen candidate lineage and
   exact current source archive/revision. It re-hashes every admitted current
   input and rejects drift, dirty/untracked source, missing artifacts, stale
   evidence, or mixed candidate identities.
8. Release workflows, tags, publication, signing/notarization, production
   admission, and independent review may consume only the current-candidate
   result, never historical-integrity success.
9. Use distinct schemas, receipt states, command names, workflow job names, and
   evidence labels so the two outcomes cannot be confused.
10. Add negative fixtures proving that historical integrity cannot satisfy a
    current release gate, current source cannot be relabeled as historical, and
    a current candidate fails after one source byte changes.
11. Keep ordinary development and CI provider-free and artifact-free. Do not
    rebuild, sign, notarize, install, publish, or retain binaries in this
    increment.

## Acceptance

- All historical artifacts and their hashes remain unchanged.
- Ordinary repository validation passes after an authorized post-candidate
  source change while returning `historical_not_current` for the old lineage.
- A forged historical receipt claiming current/release-ready fails schema and
  semantic validation.
- A current-candidate fixture passes only against its exact frozen source
  archive/revision and fails after a one-byte mutation or lineage mismatch.
- General development checks cannot emit release, publication, production,
  signing, notarization, independent-review, or platform-admission success.
- Release workflows fail closed when no fresh current candidate exists.
- Documentation clearly distinguishes historical evidence preservation from
  current release readiness.

## Non-Goals

- Creating a new release candidate, version, tag, archive, or package.
- Rebuilding or modifying the historical unsigned macOS candidate.
- Weakening source-drift checks for a current candidate.
- Using network retrieval, mutable remote refs, or a current working tree to
  reconstruct missing historical proof.
- Authorizing signing, notarization, publication, production, analyzer
  execution, or independent-review substitution.

## Approval gate

This PRD changes the meaning and routing of a repository validation rule. The
founder explicitly approved ADR-0122 on 2026-09-01. That approval authorizes the
scoped checker, schema, fixture, documentation, and workflow-routing changes;
it does not authorize rewriting historical evidence, creating a final
candidate, signing, publication, production, provider spend, or benchmark
submission.
