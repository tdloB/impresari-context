# ADR-0082: Linux External Production-Support Admission

- Status: Accepted; release gate pending
- Date: 2026-08-30
- Deciders: Founder and maintainers
- Related: ADR-0074, ADR-0077, ADR-0078, ADR-0079, ADR-0080, ADR-0081

## Context

The accepted Linux direction is A+C: existing rootless systemd user-manager
delegation where available, plus an externally managed delegated subtree. The C
profile now has exact-host synthetic package, topology, interruption, crash,
cleanup, withdrawal, and uninstall evidence. That evidence was produced from a
release candidate newer than the published v0.1.0 artifact, while the project
version still reports 0.1.0. Candidate evidence must not become a production
claim without an immutable release identity.

## Decision

Freeze a source-free, foreground admission contract for only the
`externally_managed` profile on the exact GitHub-hosted Ubuntu 24.04 x86_64
target recorded by release-candidate run `33300661271`. Pin the tracked manifest
identity, target identity, evidence identity, freshness window, source commit,
candidate archive, and final lifecycle-composition receipt.

The tracked manifest remains `pending_publication` and therefore always returns
`release_pending`, with production support withdrawn. Activation requires a new
reviewed change binding a new immutable version, tag, source commit, and release
archive. The v0.1.0 tag must not be reused.

Every stale, changed, missing, unsupported, or unavailable state withdraws the
claim. The evaluator performs no discovery, execution, network access,
credential access, privilege use, service mutation, repair, or background work.
Even a future compatible support receipt cannot authorize a real analyzer;
IAR-2 remains a separate decision.

The A rootless profile is not generalized from C and remains partial until its
genuine logout/login reentry evidence is recorded. This decision does not add
an administrator-installed service.

## Consequences

- Exact candidate evidence cannot self-promote into production support.
- Published-artifact identity and freshness are mandatory and reviewable.
- Support is narrow to the recorded surface and target, not broad Linux.
- A and C continue to be admitted independently.
- The next manual release decision is visible without blocking additional
  source-free contract work.
