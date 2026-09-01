# macOS Local-VM Build And Release-Identity Contract Architecture

- Status: Accepted for source-free implementation; candidate production remains gated
- Date: 2026-09-01
- PRD: [macOS Local-VM Build And Release-Identity Contract PRD](../product/macos-local-vm-release-identity-contract-prd.md)
- Decision: [ADR-0109](../decisions/0109-freeze-macos-build-and-release-identity-before-candidates.md)

## Context

The Option C cask roles and deterministic synthetic app tree are closed. The
next safe boundary is to define what evidence a real unsigned candidate must
carry before any build output can replace a marker.

## Flow

```text
ADR-0107 role contract + ADR-0091 guest seal + ADR-0108 tree
                              |
                              v
             ADR-0109 source/build identity contract
                              |
                 offline validation only
                              |
                              v
              contract-frozen receipt (no artifact)
                              |
                    later separate gate
                              v
       exact unsigned candidate + complete evidence record
```

## Identity layers

The contract keeps three identities separate:

1. **Contract baseline:** the merged revision on which this design began. It
   proves which known project state informed the contract but is not a release.
2. **Candidate source identity:** a later exact Git revision and source-archive
   SHA-256 recorded only when candidates are built.
3. **Compound candidate identity:** a digest over candidate revision, product
   version, and the sorted exact artifact inventory.

This avoids a self-referential contract that tries to predict its own merge
commit and prevents a documentation-only change from masquerading as a built
release.

## Build units

| Role | Builder | Future output |
| --- | --- | --- |
| CLI supervisor | locked Cargo / Rust 1.98.0 / arm64 Apple target | `Contents/MacOS/impresari-context` |
| MCP server | locked Cargo / Rust 1.98.0 / arm64 Apple target | `Contents/Helpers/impresari-context-mcp` |
| Structural worker | locked Cargo / Rust 1.98.0 / arm64 Apple target | `Contents/Helpers/impresari-context-structural-worker` |
| VM controller | recorded Xcode/SDK/Swift identity and exact unsigned command | `Contents/Helpers/impresari-context-vm-controller` |
| Guest payload | exact ADR-0091 metadata-sealed guest candidate | `Contents/Resources/macos-vm/guest` |

The existing metadata seal remains a sixth non-candidate bundle role copied
exactly. It is not rebuilt as a product artifact.

## Evidence boundary

The checker reads and hashes only the declared Impresari project metadata and
entrypoint files in its own process. The candidate record must later bind the
full source revision/archive, exact Apple and Rust build environment, artifact
bytes and digests, build-log digests, SPDX SBOM, license inventory,
vulnerability assessment, reproducibility disposition, and guest seal.

## Failure behavior

Fail before a receipt when an input changes, paths are unsafe or duplicated,
the source-set digest differs, package/guest identities drift, a bundle role is
missing or duplicated, a build command or output changes, future evidence is
optional, rollback weakens, or any current later-gate claim becomes true.

## Sequencing

A later decision may perform one bounded unsigned candidate build and retain
only explicitly approved evidence. That decision must resolve exact build-host
identity, artifact custody, source revision, product SBOM, vulnerability and
reproducibility disposition, and cleanup. Apple signing, notarization, cask
installation/publication, VM launch, analyzers, production, and macOS IAR-1B
remain separate gates.
