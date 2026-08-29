# Impresari Context — Automatic update installation PRD

- Status: Proposed; founder approval required before implementation
- Date: 2026-08-29
- Authority: Future adoption-experience increment
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)
- Architecture: [Automatic update installation ARD](../architecture/automatic-update-installation-ard.md)
- Decision: [ADR-0071](../decisions/0071-opt-in-verified-automatic-updates.md)

## Objective

Allow an operator to enroll a portable Impresari Context installation in
automatic, verified, recoverable updates without granting the updater access to
source workspaces, context caches, client configuration, credentials, or a
system-wide installation boundary.

## User outcome

An operator can explicitly preview and enroll one eligible user-owned portable
installation, choose an update channel and version policy, inspect update
status, pause or remove enrollment, and recover the prior known-good version.
Updates install unattended only after the enrolled policy and cryptographic
release evidence pass. A failed or unverifiable update leaves the current
installation runnable and produces an auditable no-update result.

## Proposed scope

- An optional updater component, packaged separately from the three runtime
  executables and disabled until explicit enrollment.
- User-level macOS ARM64 and Linux x86-64 enrollment for portable installations
  created by the pinned installer; no root or system installation.
- Preview-by-default enrollment recording the canonical install root, target,
  stable channel, allowed version range, check cadence, maintenance window,
  retained rollback count, and exact owned scheduler targets.
- TUF-style signed, versioned, expiring root/targets/snapshot/timestamp metadata
  with consistent snapshots, independent of mutable GitHub API responses.
- Verification of the release archive digest, accepted source tag, builder
  identity, and provenance subject before staging.
- Same-filesystem staging, source-free health checks, atomic activation of all
  three sibling executables, and automatic rollback on activation failure.
- Explicit status, check-now, pause, resume, rollback, and exact-owned removal
  commands with machine-readable receipts.

## Non-goals

- Updating Homebrew, package-manager, root-owned, development, source-built, or
  unrecognized installations; Homebrew retains its own update lifecycle.
- Silent enrollment, enabled-by-default checks, forced major-version changes,
  update-time workspace reads, cache migration, MCP or guidance mutation,
  client restart, sign-in, telemetry, arbitrary commands, or arbitrary URLs.
- Reusing application release credentials in the updater, trusting a checksum
  fetched from the same unsigned location as an archive, or treating GitHub
  Release availability alone as update authorization.
- Guaranteeing immediate security patch installation when the host is offline,
  asleep, outside its maintenance window, or has paused enrollment.

## Authority and privacy requirements

- Enrollment is a distinct explicit write consent. Preview identifies every
  file and scheduler entry before `--apply`; removal deletes only exact owned
  state and never removes the installed runtime.
- The updater can read only its enrollment, trusted metadata, target install
  root, staged artifacts, and update receipts. It receives no workspace, cache,
  client-home, shell-startup, or broad home-directory capability.
- The scheduled process runs as the enrolling user with a fixed executable,
  fixed arguments, scrubbed environment, bounded time/network/disk use, a
  single-instance lock, and no shell interpretation.
- Network access is limited to the configured project update-metadata and
  artifact origins. Redirects, mirrors, proxies, and origin changes fail closed
  unless admitted by signed metadata and policy.
- Receipts contain versions, identities, timestamps, reason codes, byte counts,
  and digests, but no source names, source content, credentials, environment,
  or unrelated paths.

## Update policy

- The initial channel is stable only. Prereleases and downgrades are rejected.
- The operator chooses patch-only, same-major, or exact upper-bound policy;
  major-version movement is never inferred.
- Expired metadata, missing intermediate root rotation, rollback/freeze/
  mix-and-match evidence, unknown signer, unexpected artifact, provenance
  mismatch, insufficient space, active installation lock, or failed health
  check yields a visible no-update or rolled-back state.
- Enrollment never weakens the current release assurance gate. Only a release
  admitted by the existing publication process can enter signed update targets.

## Acceptance criteria

- Preview and apply tests prove exact enrollment effects, idempotence, conflict
  rejection, symlink rejection, permission limits, and exact removal on both
  supported platforms.
- Frozen fixtures cover valid rotation and every TUF-relevant rollback, freeze,
  fast-forward, mix-and-match, expiry, signature-threshold, and target-integrity
  failure without contacting a live service.
- Provenance verification binds archive digest, source repository, source tag,
  workflow identity, builder identity, and expected target tuple.
- Fault-injection tests at every download, verify, stage, activate, health, and
  receipt boundary prove the old version remains or is restored atomically.
- Concurrent application starts and updater runs cannot expose a mixed set of
  the three sibling executable versions.
- Seeded workspace, cache, client configuration, guidance, credentials, and
  unrelated home files remain byte-identical through enrollment, successful
  update, failed update, rollback, pause, and removal.
- Hosted live rehearsal requires an explicitly disposable user-level install
  and demonstrates enroll, one accepted update, rollback, pause, and removal.

## Manual boundary

Implementation requires explicit founder acceptance of ADR-0071 plus separate
authorization for update-signing roots, scheduled background execution, and a
live rehearsal. This proposal grants none of those authorities.
