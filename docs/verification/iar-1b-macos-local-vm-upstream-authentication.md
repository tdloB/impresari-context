# IAR-1B macOS Local-VM Upstream Authentication Checkpoint

- Status: Upstream publisher-authentication checkpoint passed; Impresari distribution remains unsealed
- Date: 2026-08-30
- Decision: [ADR-0087](../decisions/0087-macos-local-vm-analyzer-confinement.md)
- Verification host: macOS arm64 with GnuPG 2.5.20

## Result

Alpine's [official downloads page](https://www.alpinelinux.org/downloads/)
publishes release-key fingerprint
`0482D84022F52DF1C4E7CD43293ACD0907D9495A` and links the aarch64 netboot
archive, checksum, and detached signature. The exact 3.24.1 archive from
Alpine's [versioned release directory](https://dl-cdn.alpinelinux.org/alpine/v3.24/releases/aarch64/)
was downloaded into temporary storage and verified as follows:

- archive bytes: `431008592`;
- archive SHA-256: `54fe38fa41cce740ba379458ed63cfcd89ab06ae5e6a47a06dafe0a34e8427e8`;
- detached-signature result: valid;
- signing fingerprint: `0482D84022F52DF1C4E7CD43293ACD0907D9495A`;
- signed `boot/vmlinuz-virt`: `10351104` bytes and
  `sha256:47970e0ee0478fe5c60824a89f162d5a353fa29466e5d3bddb0f9c506f1ed756`;
- signed `boot/initramfs-virt`: `9385851` bytes and
  `sha256:e47d38bc88509a3db11affc09f9762f9643b026bd29441724a4729ad8e97add6`.

Those two embedded identities exactly match the upstream inputs already bound
by the guest release manifest. The public key and detached signature are
committed with exact third-party provenance. The 411 MB archive is not
committed.

## Exact Claim Boundary

This checkpoint authenticates the candidate's two upstream guest inputs to the
release key identified by Alpine's official site. It does not claim that
Impresari's own release metadata is signed or sealed, does not complete the
vulnerability assessment, and does not create an Apple-signed or notarized
distribution.

The receipt therefore requires:

- `upstream_publisher_authentication_verified=true`;
- `archive_committed=false`;
- `runtime_network_required=false`;
- `release_metadata_sealed=false`;
- `vulnerability_assessment_complete=false`;
- `production_admitted=false`; and
- `analyzer_execution=false`.

No repository source, analyzer, credential, Apple signing key, or guest network
entered this check.

## Reproduction

Download the exact versioned archive outside the repository, then run:

```sh
./scripts/verify-macos-vm-alpine-archive.sh \
  /absolute/path/alpine-netboot-3.24.1-aarch64.tar.gz
```

The verifier itself performs no download. It rejects any other archive size or
digest before signature verification, verifies the pinned fingerprint, and
extracts only the two exact members after the signature succeeds.

Routine source-only validation is network-free:

```sh
ruby ./scripts/check-macos-vm-upstream-auth-contract.rb
```

## Remaining Gates

Complete and disposition the vulnerability review, seal Impresari's release
metadata and rollback chain, sign/notarize the complete bundle, rehearse the
one-cask lifecycle, and collect multi-host evidence. Genuine sleep/wake,
reboot, abrupt power loss, and independent human review also remain open.
macOS remains publicly at IAR-1A.
