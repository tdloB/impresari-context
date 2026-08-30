# Linux External Production-Support Admission PRD

## Outcome

Turn exact C-profile lifecycle evidence into a narrowly scoped, expiring support
claim only after the same implementation is published as a new immutable
release. Until then, report `release_pending` and make no production claim.

## Scope

- Profile: `externally_managed` only.
- Surface: GitHub-hosted Actions only.
- Target: Ubuntu 24.04 x86_64 with the exact runner image, kernel, and Landlock
  ABI in the tracked manifest.
- Evidence: exact run, job, source, archive, composition, and freshness IDs.
- Release: new version and tag; v0.1.0 cannot be reused.

## Required states

`release_pending`, `compatible_supported`, `stale_evidence`, `changed`,
`missing_evidence`, `unsupported`, and `unavailable` are deterministic. Only
`compatible_supported` may activate the exact production-support claim. All
other states withdraw it.

## Acceptance

1. The evaluator accepts only the repository-pinned manifest identity.
2. Candidate bytes without an admitted published release remain pending.
3. Target, release, evidence, or freshness drift fails closed.
4. Conformance rejects a pending-release production overclaim.
5. Every authority field remains denied and real analyzers remain unauthorized.
6. The complete repository gate reproduces every currently reachable state.

## Exclusions

No host discovery, source access, process launch, network, credentials,
privilege, service mutation, automatic repair, background monitoring, analyzer
execution, broad-Linux claim, rootless-profile claim, or persistent service.
