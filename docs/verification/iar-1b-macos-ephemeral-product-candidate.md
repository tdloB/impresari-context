# macOS Ephemeral Product Candidate Verification

- Date: 2026-09-01
- Decision: [ADR-0110](../decisions/0110-build-and-delete-ephemeral-macos-product-candidates.md)
- Candidate source: `aca656771f9286b13fbcc046b133ade62b58da2a`
- Product identity: `sha256:7bd280339e2a8cf30c26fc2ad96225f52cad5593c63ea621e7e44ba62b9bd5ca`

## Result

On macOS 26.5.1 build 25F80, Xcode 26.6 build 17F113, SDK 26.5 build
25F70, Swift 6.3.3, and Rust/Cargo 1.98.0, two independent private-root
builds produced byte-identical copies of all four product executables.

| Unit | Bytes | SHA-256 |
| --- | ---: | --- |
| CLI supervisor | 8,261,920 | `fa1992cd02678c03888a4a5f5a42849880dba42ef9e2b59153c5e66749499bd9` |
| MCP server | 4,496,400 | `4324a95f4a6ceeb506f659bda8d8a6cb54cb00cbfa0248e81f6b98bb815e086c` |
| Structural worker | 35,820,544 | `ab2efcae9c89c2a3cf8543c5be5cf6a63650e0ef689ec2be95df5b48aad103a7` |
| VM controller | 274,704 | `48689796ad27aa4413a95d23ebb318d14c64a786cf0c5ab1b12553d5d656b7a5` |

Each file was a thin arm64 Mach-O. Each contained a linker ad-hoc code
directory, no Team Identifier, and no Developer ID signature. The candidates
were inspected but never executed.

## Dependency evidence

- source archive SHA-256: `f26fcf7ccdc6cb499e3eacc1f479a93083c58d397c8730b72a56d43d8c0adb8b`;
- locked Cargo graph stdout SHA-256: `5a63c27b8e0eba2cbcfc842adca388118e725a0aea8883d11881e6c2f08ba44c`;
- frozen SPDX 2.3 SBOM SHA-256: `bb249501b6d693edaff188edc2344d1d1a62a94bd13ace8488f4a03e5273a3bb`;
- no-fetch Cargo Audit passed against advisory database revision
  `ba9db2a77a6a0fe93bc63a3d9b730e08b145aff5` with zero matching advisories;
- offline Cargo Deny passed advisories, bans, licenses, and sources.

Zero matching advisories is scoped to the recorded database and dependency
graph. It is not a claim that the product is vulnerability-free.

## Cleanup and claim boundary

The first pair of roots used a stable but non-source-derived epoch and was
superseded. The accepted pair used the exact candidate commit timestamp. Both
pairs were deleted. A `/private/tmp` search confirmed no matching build root
remained. No binaries, caches, raw logs, guest, app, archive, or cask were
retained.

This establishes a product-only same-host build observation. It does not
establish a complete release candidate, Apple signing, notarization,
installation, publication, VM operation, analyzer safety, production support,
or macOS IAR-1B.
