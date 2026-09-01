# macOS Local-VM Ephemeral Product Candidate PRD

- Status: Implemented; product-only rehearsal passed, release remains gated
- Date: 2026-09-01
- Architecture: [macOS Local-VM Ephemeral Product Candidate ARD](../architecture/macos-local-vm-ephemeral-product-candidate-ard.md)
- Decision: [ADR-0110](../decisions/0110-build-and-delete-ephemeral-macos-product-candidates.md)

## Problem

ADR-0109 freezes the evidence required for a complete unsigned release
candidate, but no real product executable has been built under that contract.
Building the complete guest, app, and cask at once would cross too many custody
and release boundaries in one step.

## Outcome

Build the three Rust product executables and Swift VM controller twice on one
recorded macOS arm64 host, using separate private roots, locked offline Cargo,
private Swift module caches, the candidate commit timestamp, and no Apple
credentials. Record exact source, host, toolchain, artifact, dependency,
license, vulnerability, reproducibility, and cleanup metadata, then delete all
runnable bytes and raw logs.

## Requirements

1. Bind candidate revision `aca656771f9286b13fbcc046b133ade62b58da2a`,
   its source-archive SHA-256, product version `0.2.0`, and target
   `aarch64-apple-darwin`.
2. Build exactly the ADR-0109 CLI, MCP server, structural worker, and VM
   controller units in two independent private roots.
3. Keep Cargo locked, offline, and non-incremental. Keep Swift module caches in
   the corresponding private root. Use the candidate commit time as
   `SOURCE_DATE_EPOCH`.
4. Inspect but do not execute each candidate. Record exact bytes, SHA-256,
   Mach-O architecture, linker ad-hoc code identity, absence of Developer ID
   identity, dynamic-library inventory, and build-log digest.
5. Bind the frozen SPDX SBOM and locked dependency graph. Run Cargo Audit with
   no fetch and Cargo Deny offline; record bounded dispositions rather than a
   vulnerability-free or production-safety claim.
6. Record byte equality honestly. Same-host equality cannot establish
   cross-run, cross-host, or production reproducibility.
7. Delete accepted and superseded build roots before retaining the receipt.
   Retain metadata only.
8. Keep guest completion, app assembly, archive, Apple signing, notarization,
   cask creation or installation, publication, VM launch, analyzer execution,
   release identity, production, and macOS IAR-1B false.

## Non-goals

- creating or retaining a complete unsigned release candidate;
- materializing the guest payload;
- signing with the operator's Apple identity;
- assembling, installing, or publishing the Option C cask;
- launching the local VM or any analyzer;
- admitting production or macOS IAR-1B support.

## Acceptance

The four artifacts from both accepted builds are byte-identical. Dependency,
license, and advisory checks pass under the recorded no-fetch/offline boundary.
All four disposable roots are absent after cleanup. The metadata-only record,
profile, receipt, schemas, provenance, negative overclaim fixture, and offline
checker pass the repository quality suite.
