# IAR-1B macOS Local-VM Build And Release-Identity Contract

- Status: Source-free contract accepted; executable candidate and IAR-1B pending
- Date: 2026-09-01
- Decision: [ADR-0109](../decisions/0109-freeze-macos-build-and-release-identity-before-candidates.md)

## Result

The offline checker binds the ADR-0107 app roles, ADR-0091 guest metadata seal,
product version `0.2.0`, Rust 1.98.0 arm64 Apple target, exact contract baseline,
15 direct build-control inputs, four product build units, one guest substitution
unit, complete future candidate evidence fields, and whole-bundle rollback.

This is contract evidence only. No candidate was compiled, retained, assembled,
archived, signed, notarized, installed, published, launched, or executed.

## Required future candidate evidence

- exact candidate Git revision and source-archive SHA-256;
- exact macOS, Xcode, SDK, Swift, Rust, Cargo, and target identities;
- exact bytes, SHA-256, format, architecture, unsigned state, and build-log
  digest for each product executable;
- exact guest release and metadata seal;
- SPDX 2.3 product SBOM, license inventory, vulnerability assessment, and
  reproducibility disposition digests; and
- one compound candidate identity plus rollback binding.

## Claims intentionally absent

Executable candidate, release bundle, GitHub attestation, Developer ID signing,
Apple notarization, cask lifecycle, sealed distribution, VM launch, analyzer
execution, production admission, and macOS IAR-1B all remain false.
