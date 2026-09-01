# macOS Local-VM Ephemeral Product Candidate Architecture

- Status: Implemented; product-only rehearsal passed, release remains gated
- Date: 2026-09-01
- PRD: [macOS Local-VM Ephemeral Product Candidate PRD](../product/macos-local-vm-ephemeral-product-candidate-prd.md)
- Decision: [ADR-0110](../decisions/0110-build-and-delete-ephemeral-macos-product-candidates.md)

## Context

The ADR-0109 release-identity contract closes five material roles, but its full
candidate schema requires a guest payload and compound release identity. This
checkpoint must prove that the four product build units can be produced and
identified without pretending that the guest, app, cask, or release exists.

## Flow

```text
exact aca6567 source + ADR-0109 commands + local locked caches
                              |
                              v
           private build A       private build B
             3 Cargo +              3 Cargo +
             1 Swift                 1 Swift
                    \               /
                     v             v
             exact metadata and byte comparison
                              |
              offline dependency dispositions
                              |
                              v
              delete binaries, caches, and logs
                              |
                              v
            retain closed metadata-only receipt
```

## Identity layers

The product rehearsal binds:

1. the full candidate Git revision and deterministic Git archive digest;
2. macOS, Xcode, SDK, Swift, Rust, Cargo, LLVM, architecture, and target;
3. per-build artifact and raw-log digests;
4. linker ad-hoc code-directory identities and explicit absence of an Apple
   Developer ID team identity;
5. locked dependency, SPDX SBOM, offline license, no-fetch advisory, and
   reproducibility dispositions; and
6. one canonical product identity over source, version, target, and sorted
   artifact identities.

That canonical product identity is not the ADR-0109 compound release identity.
The latter remains impossible until an exact guest candidate is present.

## Custody boundary

Compilers may read the exact repository source and existing local dependency
caches. Cargo is locked and offline. Swift uses only the installed Apple
toolchain. No Apple credential is read. Artifacts and raw logs exist only in
fresh private temporary roots and are deleted before the retained receipt is
accepted.

## Signature semantics

Modern Apple linkers place an ad-hoc code directory in Mach-O output. The
record calls this `linker-adhoc`; it does not call the artifacts unsigned in a
way that hides those bytes, and it does not treat them as Developer ID signed,
notarized, authenticated for distribution, or production-ready.

## Failure behavior

Fail before a receipt if source or toolchain identity drifts, Cargo would fetch,
a build unit is missing or duplicated, architecture or bundle role changes,
artifact bytes differ without an explicit changed disposition, evidence is
missing, cleanup fails, runnable bytes are retained, or any later-gate claim is
true.

## Sequencing

The next reversible checkpoint may design the exact authenticated guest
candidate substitution needed to complete the ADR-0109 unsigned release
record. App assembly with real bytes, Apple signing/notarization, the Option C
cask lifecycle, VM launch, analyzers, production, and macOS IAR-1B remain
separate later gates.
