# ADR-0112: Materialize And Delete The macOS Synthetic Guest Payload

- Status: Implemented; metadata only, runtime and release gated
- Date: 2026-09-01
- Decider: Aaron Boldt through the accepted roadmap continuation
- Related PRD: [macOS Local-VM Synthetic Guest Materialization PRD](../product/macos-local-vm-synthetic-guest-materialization-prd.md)
- Architecture: [macOS Local-VM Synthetic Guest Materialization ARD](../architecture/macos-local-vm-synthetic-guest-materialization-ard.md)

## Context

ADR-0111 freezes a two-file guest payload and an exact authenticated recipe but
does not prove that the files can be reproduced under the required custody and
cleanup boundary.

## Decision

Perform one macOS arm64 rehearsal in a fresh mode-`0700` private root. Download
only the exact Alpine APK, verify its pinned bytes, digest, APKv2 signature,
signed data hash, package identity, and commit, and extract only the two bounded
regular files required by the guest. Use Zig 0.16.0 and the frozen project
sources and builders to reproduce exactly `Image` and
`impresari-initramfs.gz`.

Inspect and hash those files but never execute them. Delete the download,
extracted data, output, caches, and raw logs before emitting a metadata-only
record. The materializer accepts no workspace or destination path.

## Consequences

- The synthetic guest can be proven reproducible without adding runnable
  artifact custody to the repository or release process.
- Network and compiler process launch are explicit and restricted to this
  operator rehearsal.
- App assembly, Apple identity, signing, notarization, cask lifecycle, VM
  launch, analyzer execution, release identity, production, and macOS IAR-1B
  remain separate gates.

## Alternatives

- Retain the guest binaries: rejected because executable custody and release
  distribution have not been admitted.
- Reuse the repository `target` tree: rejected because it mixes build state and
  cannot prove complete cleanup.
- Launch the guest after building it: rejected because runtime confinement is a
  later independent checkpoint.

## Revisit triggers

Revisit before changing any input, toolchain, output identity, endpoint, or
resource limit, retaining runnable bytes, assembling an app, accessing Apple
credentials, signing, notarizing, installing or publishing a cask, launching a
VM, executing an analyzer, or making release, production, or macOS IAR-1B
claims.
