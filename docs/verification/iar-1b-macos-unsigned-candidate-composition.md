# macOS Unsigned Candidate Composition Verification

- Decision: ADR-0113
- Date: 2026-09-01
- Result: source-free prospective projection exact; complete candidate absent

## Verified

- Exact release and package contracts.
- Exact ADR-0110 product, ADR-0108 synthetic-tree, ADR-0111 guest-contract,
  ADR-0112 guest-materialization, and guest-seal source records.
- A sorted unique eight-file prospective app projection.
- Exact required paths, modes, bytes, SHA-256 identities, and evidence roles.
- Prospective compound identity
  `sha256:39ae0afbb77eff80ff5308cc4fe811b7cc266b42d02b4457aa5295310908b11e`.

## Not verified or claimed

No complete candidate or app was materialized. Product and guest bytes never
shared a custody root and candidate modes were not verified. No executable is
retained. Release identity, Apple identity, signing, notarization, cask,
installation, VM, analyzer, production, and macOS IAR-1B remain false.
