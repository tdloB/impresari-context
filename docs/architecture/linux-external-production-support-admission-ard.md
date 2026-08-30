# Linux External Production-Support Admission ARD

## Boundary

The evaluator is a source-free pure decision process. The caller supplies
already-observed target, evidence-availability, release-availability, and date
values. The evaluator neither discovers nor changes the host.

## Identity chain

The tracked manifest SHA-256 is compiled into the evaluator. The manifest pins:

1. the exact C profile and GitHub-hosted surface;
2. runner image, OS, kernel, architecture, and Landlock ABI;
3. hosted run/job, source commit, candidate archive, and composition receipt;
4. an expiry date; and
5. the release-publication gate.

A caller cannot substitute a permissive manifest. Updating the admitted release
requires a reviewed source change that updates both the manifest and its pinned
identity. The schema supports a future published-release shape, but the tracked
pending manifest and evaluator cannot produce a positive receipt.

## Decision order

The evaluator rejects an unrecorded surface, then target unavailability,
missing evidence, stale evidence, and target drift. The current manifest then
returns `release_pending`. A future published manifest must additionally match
the exact version, tag, source, and archive before returning
`compatible_supported`.

## Claim separation

`support_claim_active` and `production_admitted` are true only for
`compatible_supported`. `real_analyzer_authorized` remains false in every
state. A production-support admission therefore proves a maintained OS boundary
for a specific distribution surface; it does not authorize IAR-2 execution.
