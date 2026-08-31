# ADR-0091: Content-Address The macOS Local-VM Release Metadata Before Distribution Signing

- Status: Accepted for implementation; distribution remains gated
- Date: 2026-08-31
- Decider: Aaron Boldt through the accepted ADR-0087 roadmap sequence

## Context

ADR-0087 now has a current authenticated guest, exact v2 runtime profiles,
SBOM, licenses, provenance, rollback predecessor, and a fail-closed
vulnerability assessment. Apple signing or a cask checksum would be ambiguous
unless those records are first bound as one immutable reviewed set.

Checksums alone do not authenticate a publisher, and introducing a second
long-lived signing key would duplicate ADR-0016's keyless release-attestation
design. Apple code signing also authenticates executable bundle bytes rather
than defining the repository metadata set that was reviewed.

## Decision

Create one closed, path-sorted release-metadata inventory. Bind each member by
exact repository-relative path, byte length, and SHA-256, then compute one
canonical SHA-256 metadata-set digest over those entries. Content-address the
seal through an exact profile and produce a deterministic source-free receipt.

The receipt may state `release_metadata_sealed=true` only when the complete
inventory and all cross-bindings pass. It must keep
`impresari_publication_attestation_verified`, `developer_id_signature_verified`,
`apple_notarization_verified`, `cask_lifecycle_verified`,
`sealed_distribution`, `production_admitted`, and `analyzer_execution` false.

At publication, use ADR-0016's GitHub keyless artifact attestation and the
separate Apple/cask gates. Do not introduce a repository, runtime, or CI secret
for this metadata checkpoint.

## Consequences

- Any reviewed-metadata drift withdraws the seal deterministically.
- Release packaging gains one unambiguous metadata subject to attest and sign.
- Routine validation remains offline and credential-free.
- The seal cannot be described as publisher authentication or sealed
  distribution until the independent publication layers pass.
- Any guest/profile update requires a new versioned seal rather than mutation
  of historical evidence.

## Alternatives

- Rely on the guest manifest alone: rejected because it omits active runtime
  profiles and later vulnerability/authentication records.
- Treat a committed checksum as publisher authentication: rejected as an
  overclaim.
- Add a standalone maintainer signing key now: rejected because it adds key
  custody, rotation, revocation, and recovery burden already avoided by
  ADR-0016.
- Wait until notarization: rejected because Apple signing does not define the
  reviewed repository metadata graph.

## Revisit triggers

Revisit before non-GitHub distribution, delegated release authority, offline
publisher signatures, guest self-update, runtime network retrieval, or a
different metadata-signing trust root.
