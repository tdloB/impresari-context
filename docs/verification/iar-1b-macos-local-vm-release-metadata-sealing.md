# IAR-1B macOS Local-VM Release-Metadata Sealing Checkpoint

- Status: Content-addressed metadata checkpoint passed; sealed distribution remains open
- Date: 2026-08-31
- Decision: [ADR-0091](../decisions/0091-content-address-macos-local-vm-release-metadata.md)
- Guest release: `iar-macos-local-vm-guest-2026-08-31.1`

## Result

The active v2 macOS local-VM guest metadata is now one closed,
content-addressed set. Sixteen exact metadata and public-verification members
are bound by repository-relative path, byte length, and SHA-256. Their canonical
metadata-set digest is:

`sha256:ea29c43f36493f7e61935f33a64822805c8275d804c5384c3e8becea849fc54b`

The exact seal digest is:

`sha256:c0294a88c2c7fe1d33bdd8ddfbb55e26e6595f02c12a9645c898f36148aa82e1`

The exact profile digest is:

`sha256:4f3b504dd20682de005d074f57fcb3807d721939a5e03fe801274aa6efbe47a5`

## Bound evidence

The set includes the active guest manifest and assets, Alpine public
verification key and authentication record, SPDX SBOM, license and provenance
records, vulnerability policy and current incomplete assessment, plus every
active v2 matrix, supervisor, resource, interruption, supply-chain,
authentication, and vulnerability-review profile.

The checker independently verifies every member and recomputes the canonical
set digest. It cross-binds the active guest release, component-set digest,
upstream-authentication record, vulnerability assessment, and rollback
predecessor. Missing, extra, reordered, symlinked, resized, modified, escaped,
or expired evidence fails closed.

## Boundary

`release_metadata_sealed=true` means only that the reviewed repository metadata
set is exact and immutable. It is not a publisher signature and does not mean
the distributable macOS bundle is sealed. Routine validation is source-free,
offline, read-only, and credential-free.

The deterministic receipt therefore requires complete advisory coverage,
vulnerability completion, publication attestation, Developer ID signing,
notarization, cask lifecycle, sealed distribution, production, analyzer
execution, and added authority all to remain false.

## Remaining gates

Complete advisory coverage, genuine sleep/wake plus reboot and power-loss
evidence, multi-host evidence, Developer ID signing/notarization, the one-cask
lifecycle, final GitHub publication attestation, and the deferred independent
human security review remain required before macOS IAR-1B or any real-analyzer
admission.
