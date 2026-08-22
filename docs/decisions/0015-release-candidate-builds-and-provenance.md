# ADR-0015: Release-candidate builds and provenance

- Status: Accepted for implementation; publication remains manual
- Date: 2026-08-22
- Scope: Binary packaging, checksums, SBOM, provenance, and clean-install rehearsal

## Context

The repository needs repeatable release artifacts for macOS ARM64, Linux x64,
and Windows x64. Building and uploading binaries from a maintainer laptop would
make source linkage and platform reproduction difficult. Automatic publication,
however, would make a mistaken tag or compromised workflow immediately public.

## Decision

GitHub Actions is the initial release-candidate build system.

- A manually dispatched workflow builds the exact selected commit. Tag creation
  and GitHub Release publication are not performed by the workflow.
- Every job uses the pinned Rust toolchain and locked dependency graph.
- Each target package contains the CLI, structural worker, local MCP binary,
  license, notices, acknowledgments, security/support documentation, and SBOM.
- Deterministic package manifests and SHA-256 checksums bind filenames, target,
  source commit, toolchain, and contained files.
- Jobs use least-privilege `contents: read`; no release secret, write permission,
  OIDC token, signing key, or package-registry credential is available.
- A clean-install rehearsal unpacks into a temporary directory, invokes packaged
  binaries, verifies stdout discipline, and confirms no global shell, editor,
  Git, or source-workspace mutation.
- CI artifacts are release candidates, not published releases and not a claim of
  reproducible bit-for-bit cross-run builds.

## Signing and provenance

V1 uses source-commit binding, a generated manifest, SHA-256 checksums, the
locked SBOM, and GitHub workflow/run identity as build provenance. Cryptographic
publisher identity and non-repudiation remain a manual pre-publication gate.
Keyless attestations or offline maintainer signing may be added only after a
separate credential, revocation, verification, and recovery review.

## Verification

- Build all Tier A target packages in the hosted matrix.
- Verify package contents and checksums before upload.
- Run clean-install smoke tests from unpacked packages on each native target.
- Keep upload/download actions pinned and workflow permissions read-only.
- Archive successful workflow links in the release evidence record.

## Publication boundary

Creating a version tag, changing repository visibility, publishing a GitHub
Release, or uploading to Cargo/Homebrew/Winget/Scoop remains a manual owner
action after independent security/release review.
