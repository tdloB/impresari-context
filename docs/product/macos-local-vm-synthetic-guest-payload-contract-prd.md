# macOS Local-VM Synthetic Guest Payload Contract PRD

- Status: Implemented; source-free contract only
- Date: 2026-09-01
- Architecture: [macOS Local-VM Synthetic Guest Payload Contract ARD](../architecture/macos-local-vm-synthetic-guest-payload-contract-ard.md)
- Decision: [ADR-0111](../decisions/0111-freeze-the-macos-synthetic-guest-payload-before-materialization.md)

## Problem

ADR-0110 proves and deletes the four product executables, but the exact guest
payload needed by the ordinary VM path is still represented only by a closed
directory role. The existing guest release manifest includes six components,
four of which are build or resource-test intermediates that must not silently
become release payload members.

## Outcome

Freeze a source-free contract for exactly two mode-`0644` resources beneath
`Contents/Resources/macos-vm/guest`: `Image` and
`impresari-initramfs.gz`. Bind those identities to the current metadata-sealed
synthetic guest, the ordinary controller asset names, and one future
authenticated, private-root, delete-on-completion materialization recipe.

## Requirements

1. Close the payload root to two sorted, unique regular files with no links,
   special files, traversal, executable modes, or additional members.
2. Bind exact bytes and SHA-256 identities from guest release
   `iar-macos-local-vm-guest-2026-08-31.1`, its manifest, component-set digest,
   metadata-set digest, and metadata seal.
3. Exclude the standalone guest init, resource guest init, resource initramfs,
   and extracted module as runtime package members while retaining their build
   provenance roles.
4. Bind the exact ordinary controller names, size limits, and digests without
   running the controller or a VM.
5. Freeze the later build route to the exact authenticated Alpine APK, public
   key, package identity, two extracted inputs, Impresari-owned guest source,
   Zig target/options, and deterministic initramfs builder.
6. Require a new private temporary root, exact output remeasurement, no
   retained cache or executable, and complete cleanup before a later receipt
   may pass.
7. Keep download, build, guest materialization, app assembly, Apple signing,
   notarization, cask creation or install, VM launch, analyzer execution,
   release identity, production, and macOS IAR-1B false.

## Non-goals

- downloading the Alpine package or producing guest bytes;
- packaging resource-canary or build-intermediate components;
- changing the controller or guest implementation;
- assembling or distributing an app or cask;
- launching a VM or admitting a real analyzer.

## Acceptance

The contract, profile, receipt, three schemas, provenance record, invalid
materialization overclaim, and offline checker pass the repository suite. The
checker must read only exact project metadata and source identities and must
not download, compile, launch, or retain a guest artifact.
