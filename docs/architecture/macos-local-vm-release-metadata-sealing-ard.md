# macOS Local-VM Release-Metadata Sealing Architecture

- Status: Accepted for synthetic implementation; distribution remains gated
- Date: 2026-08-31
- Governing PRD: [macOS local-VM release-metadata sealing PRD](../product/macos-local-vm-release-metadata-sealing-prd.md)
- Governing decision: [ADR-0091](../decisions/0091-content-address-macos-local-vm-release-metadata.md)

## Boundary

The seal covers metadata and public verification material only. It never reads
workspace content, prepared guest executables, credentials, or network data.
Its validator is a source-free repository check with no mutation or process
launch authority.

## Identity graph

```text
active v2 metadata and profiles
        │ exact path + bytes + SHA-256
        ▼
sorted closed member inventory
        │ SHA-256(path<TAB>bytes<TAB>sha256:digest<LF>...)
        ▼
metadata_set_digest
        │ embedded in content-addressed seal
        ▼
profile binds exact seal bytes
        │
        ▼
deterministic source-free receipt
```

The seal does not inventory its own profile, avoiding a circular digest. The
profile binds the exact seal digest, and its adjacent sidecar binds the exact
profile bytes. The receipt binds both identities.

## Canonicalization

- Members are strictly increasing by bytewise repository-relative path.
- Paths are limited to the frozen `platform/macos-vm-feasibility/` and
  `profiles/v1/` active-v2 allowlist.
- A member line is UTF-8 ASCII:
  `path<TAB>decimal-bytes<TAB>sha256:lowercase-hex<LF>`.
- The metadata-set digest is SHA-256 over the concatenated member lines.
- JSON formatting is itself identity-bearing through the seal and profile file
  digests; semantic reserialization is not accepted silently.

## Authentication layers

The content-addressed seal proves internal consistency and immutability of the
reviewed set. It does not authenticate an Impresari publisher by itself. A
published bundle must later receive:

1. GitHub keyless build-provenance attestation under ADR-0016;
2. Developer ID nested code signatures and Apple notarization;
3. an exact cask checksum and lifecycle evidence.

No long-lived Impresari metadata-signing key is introduced. The receipt keeps
publication attestation, Developer ID, notarization, cask lifecycle, sealed
distribution, production, and analyzer claims false.

## Failure behavior

Missing, extra, duplicate, reordered, escaped, symlinked, resized, or modified
members fail before a receipt is emitted. Expired guest metadata and any
attempt to set complete vulnerability coverage or production authority also
fail. Validation is offline and deterministic.

## Verification

- JSON Schema valid and invalid fixtures.
- Exact fixture/repository byte equivalence for the profile and seal.
- Independent recomputation of every member digest and the metadata-set digest.
- Negative tests for member drift and distribution/production overclaim.
- Inclusion in the complete repository gate.
