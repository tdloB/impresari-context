# macOS Local-VM Synthetic Guest Materialization PRD

- Status: Implemented; metadata-only rehearsal evidence
- Architecture: [macOS Local-VM Synthetic Guest Materialization ARD](../architecture/macos-local-vm-synthetic-guest-materialization-ard.md)
- Decision: [ADR-0112](../decisions/0112-materialize-and-delete-the-macos-synthetic-guest-payload.md)

## Outcome

Prove that the exact ADR-0111 two-member synthetic guest payload can be
recreated from its authenticated public input and project-owned source without
retaining runnable guest bytes.

## Requirements

- Use one fresh mode-`0700` private root.
- Fetch only the exact publisher-authenticated Alpine APK over HTTPS with zero
  redirects and enforce its bytes and SHA-256 before parsing it.
- Verify the APKv2 publisher signature, signed data hash, package identity, and
  commit before extracting only the kernel and `virtio_blk` module.
- Use exact project source, Zig 0.16.0, the frozen target and flags, and the
  canonical kernel and initramfs builders.
- Require the exact two mode-`0644` payload identities from ADR-0111.
- Inspect but never execute the guest artifacts.
- Delete the download, extracted inputs, outputs, caches, and raw logs before a
  metadata-only record is emitted.

## Non-goals

This increment does not assemble an app, access Apple identity, sign, notarize,
install or publish a cask, launch a VM, execute an analyzer, bind a release, or
admit production or macOS IAR-1B.

## Acceptance

One local macOS arm64 rehearsal passes the frozen resource profile, produces
the exact two expected identities, proves complete cleanup, and retains only a
schema-valid metadata record and receipt. Offline CI revalidates the record,
contract, source digests, fixture provenance, and non-claim boundary.
