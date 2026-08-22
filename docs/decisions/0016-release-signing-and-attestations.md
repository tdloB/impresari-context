# ADR-0016: GitHub attestations and SHA-256 checksums for v0.1.0

- Status: Accepted
- Date: 2026-08-22
- Scope: Published binary integrity and build provenance

## Context

Users need a practical way to determine whether a downloaded release artifact
is unchanged and whether it was built from the intended Impresari Context source
and workflow. Requiring the initial maintainer to manage a long-lived signing
key would add storage, rotation, revocation, and recovery risks before the
project has that operational capacity.

## Decision

The first public release will publish both SHA-256 checksums and GitHub artifact
attestations for every binary archive.

- A checksum is the portable, offline-verifiable identity of the exact archive.
- A GitHub artifact attestation binds the archive digest to the public
  repository, source commit, workflow, and GitHub Actions identity.
- Attestations will use GitHub's keyless Sigstore-backed service. No maintainer
  signing key or release secret will be introduced.
- The release workflow will receive only the minimum `id-token: write` and
  `attestations: write` permissions required for the attestation job, plus
  explicitly justified release-publication permissions if publication is later
  automated.
- Tag creation and GitHub Release publication remain explicit owner actions for
  v0.1.0.

## Why not checksums alone

Checksums detect a changed file, but an attacker controlling a download page
could replace both an archive and its displayed checksum. The attestation adds
an independently verifiable statement about where and how the archive was
built.

## Why not standalone Cosign initially

GitHub attestations already use Sigstore technology while providing repository
and workflow integration with no long-lived signing key. Standalone Cosign may
be added later for non-GitHub distribution or ecosystem requirements, but it
would duplicate tooling and verification instructions for the initial release.

## Verification

- Verify the published `.sha256` file locally.
- Verify each archive with `gh attestation verify` against the public
  `tdloB/impresari-context` repository.
- Confirm the attestation subject digest equals the released archive digest.
- Archive the workflow run, source commit, checksums, and verification results
  in the release evidence record.

## Reconsider when

Reconsider this decision before publishing through a non-GitHub package
registry, delegating release authority, requiring offline signer identity, or
introducing organizational signing and revocation requirements.
