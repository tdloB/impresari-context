# macOS Local-VM Ephemeral Unsigned Release Candidate PRD

- Status: Accepted for implementation under ADR-0114
- Date: 2026-09-01

## Problem

ADR-0110 proved the four product binaries, ADR-0112 proved the two synthetic
guest files, and ADR-0113 proved that their metadata forms a closed app
projection. No checkpoint has built and assembled all eight files at the same
time under one custody and cleanup boundary, so the frozen ADR-0109 unsigned
candidate schema is not yet satisfied.

## Requirements

1. Use the exact ADR-0110 source revision, source archive, product identities,
   host/toolchain class, and dependency evidence.
2. Authenticate and build the exact ADR-0112 synthetic guest from its single
   pinned public APK.
3. Hold every component only in one fresh mode-`0700` private temporary root.
4. Assemble the exact eight-file ADR-0113 app projection with exact modes,
   bytes, SHA-256 identities, and a closed no-symlink/no-special-file tree.
5. Bind both the ADR-0109 compound identity and the stronger ADR-0113 material
   projection identity.
6. Execute no produced artifact and delete the app, source archive, extracted
   source, download, build outputs, caches, and raw logs before acceptance.
7. Retain metadata only. Keep Apple identity, signing, notarization, archive,
   cask, install, publication, VM, analyzer, production, and IAR-1B false.

## Success

One authenticated rehearsal produces a schema-valid unsigned-candidate record,
verifies complete simultaneous custody and exact cleanup, and leaves no
runnable candidate behind.

## Non-goals

Developer ID signing, notarization, Homebrew packaging, installation, VM
launch, analyzer execution, publication, or production admission.
