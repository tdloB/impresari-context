# Historical Release Evidence Gate Separation — Architecture

- Status: Approved for implementation by the founder on 2026-09-01.
- Date: 2026-09-01.
- Governing PRD:
  [Historical Release Evidence Gate Separation PRD](../product/historical-release-evidence-gate-separation-prd.md).
- Proposed decision:
  [ADR-0122](../decisions/0122-separate-historical-evidence-from-current-release-gates.md).

## Architecture outcome

```text
immutable historical records                 future fresh candidate lineage
            │                                             │
            ▼                                             ▼
historical_evidence_integrity                  current_release_candidate
  - schema and canonical IDs                    - exact source archive/revision
  - frozen fixture/provenance                   - current input/artifact hashes
  - cross-record bindings                       - SBOM/risk/review/release gates
  - current/release claims false                - source drift fails
            │                                             │
            ▼                                             ▼
ordinary development evidence                  release-only evidence
historical_not_current                         current_candidate | denied
```

The split preserves evidence rather than changing it. A historical verifier
answers whether tracked metadata remains the same internally consistent record.
A current-candidate verifier answers whether an exact source/artifact set is
eligible to proceed through later release gates. Neither result is silently
promoted into the other.

## Historical integrity path

The existing versioned historical JSON, schemas, checksum sidecars, fixtures,
and provenance inventories remain immutable. A new closed profile identifies
their exact digests and runs only source-free operations:

1. parse with duplicate/unknown-field rejection;
2. verify every tracked record and fixture digest;
3. recompute canonical identities and cross-bindings from recorded values;
4. verify the original candidate/source identifiers remain present and unique;
5. require `historical=true`, `current_source_verified=false`, and all release,
   publication, production, independent-review, and authority claims false;
6. emit `historical_not_current`.

It deliberately does not open current product source paths. This avoids
substituting new bytes for historical source. The receipt must state that the
recorded source bytes were not reproduced during ordinary validation.

## Optional deep historical audit

A separately invoked audit may accept one local regular-file source archive
and its expected prefixed SHA-256. The archive path is trusted operator input,
must be outside the repository/cache, non-symlinked, bounded, and read-only.
The verifier first checks the archive digest and recorded revision, then safely
enumerates the exact historical paths and compares their bytes/hashes. Missing
archives yield `historical_source_unavailable`, not failure of ordinary
development and not proof of source equality.

No network, Git fetch, mutable ref, checkout, extraction into the product tree,
or automatic archive creation is permitted.

## Current-candidate path

The current release gate consumes one newly versioned candidate manifest bound
to an exact full source archive SHA-256 and immutable revision. It runs only in
the release workflow or an explicit local release rehearsal. Its source
verifier reads the admitted archive, reconstructs the closed input inventory,
and checks every current digest before consuming artifact, SBOM, vulnerability,
reproducibility, platform, signing, notarization, distribution, and independent
review evidence.

No current candidate means `release_candidate_absent`, which is a successful
ordinary-development state and a failed release state.

## Separation controls

- Separate schema names, profile IDs, receipt states, commands, and CI jobs.
- Historical profiles cannot reference current candidate schemas or approval
  states.
- Release jobs never accept `historical_not_current` as a successful
  dependency.
- Ordinary jobs never emit `current_candidate`.
- Candidate lineage is append-only and versioned; historical files are not
  edited when new source is frozen.
- Every report carries the exact lineage ID and source-verification state.
- Source-free validation remains safe on shallow checkouts and worktrees.

## Migration

1. Add new closed historical-integrity profile/receipt schemas and fixtures.
2. Add a source-free checker that binds the existing historical artifacts
   without editing them.
3. Move direct current-tree hash comparison out of the ordinary repository gate
   into the current-candidate checker.
4. Keep the old checker available temporarily as a diagnostic and prove its
   only failure is expected post-candidate source drift.
5. Add release-workflow conformance checks proving the current-candidate job is
   mandatory for release paths.
6. Remove the ambiguous ordinary invocation only after hosted macOS/Linux/
   Windows tests pass and the founder-approved decision is recorded.

## Security and quality checks

- historical file mutation, fixture substitution, digest mismatch, duplicate
  lineage, and current/release overclaim;
- absent, symlinked, oversized, wrong-digest, traversing, duplicate-member, and
  decompression-boundary historical archives;
- current-candidate source mutation, dirty/archive mismatch, missing input,
  mixed lineage, stale evidence, and historical-result substitution;
- deterministic source-free receipts across platforms;
- workflow static checks that no release path can skip the current-candidate
  gate.

## Revisit triggers

Revisit before changing historical files, accepting remote archives, fetching
Git history, creating a final candidate, changing release versions, weakening a
current-source drift check, or allowing any historical result into signing,
publication, production, or independent-review admission.
