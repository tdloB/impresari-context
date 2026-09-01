# macOS Local-VM Unsigned Synthetic Bundle Assembly PRD

- Status: Accepted for source-free implementation; distribution remains gated
- Date: 2026-09-01
- Architecture: [macOS Local-VM Unsigned Bundle Assembly ARD](../architecture/macos-local-vm-unsigned-bundle-assembly-ard.md)
- Decision: [ADR-0108](../decisions/0108-assemble-a-non-runnable-macos-bundle-before-signing.md)

## Problem

ADR-0107 freezes what a future one-cask package owns, but no evidence yet proves
that those roles can be placed into one deterministic `.app` tree. Moving
directly to release binaries, signing, installation, or Homebrew would combine
several security and credential boundaries in one step.

## Outcome

Assemble `Impresari Context.app` twice under separate private temporary roots
using only generated non-runnable markers and the exact ADR-0091 metadata seal.
Verify the complete tree, file modes, byte lengths, digests, determinism, and
cleanup. Retain only schemas, metadata, and receipts; retain no app bundle.

## Requirements

1. Every file and directory is enumerated in one digest-bound assembly spec.
2. CLI, MCP, structural-worker, VM-controller, and guest-payload destinations
   contain unmistakable synthetic text with no executable permission.
3. `Info.plist` uses a synthetic identifier and version and is digest-bound.
4. The metadata-seal copy is byte-identical to ADR-0091.
5. Symlinks, special files, unexpected paths, files above 8 KiB, and path
   traversal fail closed.
6. Two independent assemblies produce the same canonical tree digest. Windows
   structural CI compares paths, kinds, bytes, and digests against that target
   tree without treating Windows mode bits as target evidence.
7. Each macOS/POSIX temporary root has mode `0700` and is removed before
   success. Windows CI verifies only fresh non-symlinked temporary roots,
   deterministic structure, and cleanup; Windows mode bits are not accepted as
   evidence of the target macOS privacy or executable-bit policy.
8. No archive, cask, install, signing, notarization, network, credential,
   child-process, VM, analyzer, production, or IAR-1B authority is added.

## Non-goals

- compiling or packaging product executables;
- staging a runnable guest;
- producing a downloadable or installable artifact;
- validating macOS launch behavior;
- accessing Apple credentials or services;
- running Homebrew or changing a user's machine.

## Acceptance

The offline checker must validate the registered schemas and fixtures, assemble
and remove two exact trees, reject the overclaim fixture, and emit the frozen
receipt. This checkpoint must remain valid on non-macOS CI because it evaluates
portable filesystem structure rather than OS launch behavior. Only macOS/POSIX
runs provide `0700` mode evidence; Windows remains structural-only.
