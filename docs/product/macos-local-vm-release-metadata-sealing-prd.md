# Impresari Context — macOS Local-VM Release-Metadata Sealing PRD

- Status: Accepted for implementation; distribution remains gated
- Date: 2026-08-31
- Authority: ADR-0087 accepted roadmap sequence
- Architecture: [macOS local-VM release-metadata sealing ARD](../architecture/macos-local-vm-release-metadata-sealing-ard.md)
- Decision: [ADR-0091](../decisions/0091-content-address-macos-local-vm-release-metadata.md)

## Objective

Freeze one complete, deterministic, offline-verifiable identity for every
repository record that defines the active macOS local-VM guest candidate. A
consumer must be able to detect an added, removed, reordered, resized, or
changed member before any later Apple signing, notarization, cask packaging,
production admission, or analyzer execution step.

## User outcome

A later release can prove exactly which guest, upstream authentication,
vulnerability disposition, resource profiles, provenance, licenses, and SBOM
were reviewed. A checksum or signature for a package cannot silently refer to
a different metadata set.

## Scope

- One closed release-metadata seal listing every active v2 guest record and
  runtime profile by repository-relative path, byte length, and SHA-256.
- One canonical metadata-set digest derived from the sorted member inventory.
- One content-addressed profile and deterministic source-free receipt.
- Offline validation of every member, the current release identity, upstream
  authentication record, incomplete vulnerability state, and rollback
  predecessor.
- A future publication binding that uses the existing GitHub keyless artifact
  attestation policy and Apple distribution controls without adding a new
  maintainer signing key.

## Non-goals

- Developer ID signing, notarization, Homebrew publication, release tagging,
  GitHub Release publication, or automatic update installation.
- Claiming complete advisory coverage, vulnerability freedom, sealed
  distribution, production admission, macOS IAR-1B, or analyzer execution.
- Accessing the network, Apple credentials, GitHub credentials, repository
  source content, or prepared guest executables during routine validation.
- Replacing GitHub artifact attestations or Apple code signing with a checksum.

## Acceptance criteria

- The seal inventory is path-sorted, duplicate-free, closed, and exact.
- Every listed path is a regular non-symlink file inside the repository and
  matches both the recorded byte length and SHA-256.
- The canonical metadata-set digest is independently recomputed and exact.
- The active manifest, component-set digest, upstream-authentication record,
  current vulnerability assessment, and rollback predecessor are cross-bound.
- The receipt distinguishes `release_metadata_sealed=true` from
  `sealed_distribution=false`.
- Any missing member, unlisted member, path escape, symlink, digest drift,
  expiry, or claim escalation fails closed.
- Conformance includes valid profile, seal, and receipt fixtures plus a
  negative overclaim fixture with reviewed provenance.

## Remaining manual and external gates

This checkpoint does not require an Apple credential or publication action.
Developer ID signing/notarization, the one-cask lifecycle, multi-host and
genuine interruption evidence, complete vulnerability coverage, final release
attestation, and independent human review remain separate gates.
