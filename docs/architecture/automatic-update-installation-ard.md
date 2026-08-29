# Automatic update installation architecture

- Status: Proposed; not approved for implementation
- Date: 2026-08-29
- Governing PRD: [Automatic update installation PRD](../product/automatic-update-installation-prd.md)
- Governing decision: [ADR-0071](../decisions/0071-opt-in-verified-automatic-updates.md)

## Context

Automatic installation combines persistent scheduling, network retrieval,
software-supply-chain trust, executable replacement, rollback, and credential
operations. It is therefore a materially different authority boundary from the
pinned installer and Homebrew proposal. The core runtime must remain usable and
network-free without the updater.

## Proposed component boundary

Add an optional `impresari-context-updater` package containing:

1. a source-free enrollment/status CLI;
2. a fixed scheduled-run entry point with no arbitrary argument or shell
   surface;
3. a TUF metadata verifier seeded with the project's trusted root;
4. a release-provenance verifier with fixed repository, workflow, builder, and
   target expectations;
5. a same-filesystem transactional installer and rollback journal; and
6. canonical machine-readable enrollment and update receipts.

The updater is not linked into, launched by, or required by the CLI, MCP server,
structural worker, client adapters, or evidence engine. Those binaries remain
free of updater network, scheduler, and signing authority.

## Capability layout

Enrollment resolves and pins capabilities before scheduling:

- one canonical user-owned portable install root containing the exact three
  recognized executables;
- one updater-owned state root outside every workspace and context cache;
- one platform-specific user scheduler target;
- two fixed HTTPS origins: signed metadata and release artifacts;
- bounded temporary/staging space on the installation filesystem; and
- an exclusive updater lock plus a read-only runtime-version coordination lock.

No recursive home access is granted. The process opens enrolled paths through
capability-relative, no-symlink operations and rejects identity or ownership
changes rather than rediscovering them.

## Trust and metadata design

Use a TUF-compatible repository with root, targets, snapshot, and timestamp
roles, threshold policy, expirations, monotonic versions, and consistent target
names. Root rotation requires the complete intermediate chain and signatures
from both the old and new thresholds. Offline root keys are excluded from CI;
online release automation receives only the minimum delegated targets role
needed for admitted stable releases.

Each target binds:

- SemVer and stable channel;
- platform tuple and immutable archive name;
- archive length and SHA-256;
- source repository and exact release tag;
- accepted build workflow and builder identities;
- provenance subject digest; and
- the minimum updater metadata/schema versions.

Update selection happens only after timestamp, snapshot, delegated target,
version-policy, and provenance verification. Local cached metadata never
becomes trusted merely because the host is offline.

## Transaction design

1. Acquire the updater lock and snapshot the current executable identities.
2. Refresh and verify metadata within strict byte/time/redirect bounds.
3. Select the highest admitted version allowed by enrollment policy.
4. Download into a newly created staging directory on the same filesystem.
5. Verify length, digest, provenance, archive paths, executable set, and modes.
6. Run source-free `--version` and diagnostic checks with no workspace/cache or
   inherited credential environment.
7. Atomically activate one versioned directory and switch a single installation
   pointer; never replace the three visible files independently.
8. Re-run health checks through the active pointer. On failure, restore the
   prior pointer and record rollback before releasing the lock.
9. Retain only the enrolled number of prior admitted versions after success.

If the existing portable layout cannot support a single atomic activation
pointer without breaking sibling discovery, implementation must first add and
accept a compatible versioned-directory layout; it may not approximate
atomicity with sequential overwrites.

## Scheduling design

The enrollment CLI renders but does not apply a user-level `launchd` target on
macOS or `systemd --user` timer/service on Linux until explicit `--apply`.
Targets invoke the canonical updater directly with a fixed enrollment identity,
use randomized delay and bounded timeout, and never run as root or at boot
before the user's session. Scheduler output is redirected only to bounded
updater-owned receipts.

Pause disables the exact owned timer without deleting trust state. Removal
unloads and deletes only exact recognized scheduler/state artifacts. Unknown or
modified artifacts cause removal to fail closed and emit manual recovery data.

## Failure and withdrawal rules

- Loss or suspected compromise of any online signing role freezes new target
  publication; clients retain the current version and await signed recovery.
- Root compromise invokes an offline, dual-threshold rotation and published
  incident procedure; clients missing a valid chain do not update.
- Metadata expiry, clock invalidity, or prolonged offline state is reported as
  stale, never silently accepted.
- A faulty release is withdrawn with new signed metadata; automatic downgrade
  is not allowed. A separately signed rollback authorization and operator
  action are required to move backward.
- The automatic-update claim is withdrawn for a platform whenever its hosted
  scheduler, atomic activation, or rollback rehearsal expires or changes.

## Alternatives rejected

- **Check GitHub Releases and compare SemVer:** lacks independent rollback,
  freeze, mix-and-match, expiry, and root-rotation defenses.
- **Trust an adjacent checksum:** an attacker controlling the artifact location
  can replace both files.
- **Run as root or use a system scheduler:** unnecessarily broadens the initial
  product and installation boundary.
- **Overwrite executables in sequence:** can expose mixed-version runtime sets.
- **Automatically update Homebrew installs:** conflicts with package-manager
  ownership and receipts.
- **Embed updater logic in the MCP server:** gives a source-facing long-lived
  process network and executable-replacement authority.

## Implementation gate

This architecture is non-operative until ADR-0071, signing-root custody,
scheduler authority, and live-rehearsal authorization are separately approved.
