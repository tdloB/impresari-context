# macOS Local-VM Synthetic Guest Materialization ARD

- Status: Implemented; runtime and release remain gated
- PRD: [macOS Local-VM Synthetic Guest Materialization PRD](../product/macos-local-vm-synthetic-guest-materialization-prd.md)
- Decision: [ADR-0112](../decisions/0112-materialize-and-delete-the-macos-synthetic-guest-payload.md)

## Boundary

The materializer is an operator-only build rehearsal. It reads exact public
and project-owned inputs, launches only the pinned fetch, verification,
inspection, and compilation tools, and writes exclusively beneath a newly
created private temporary root. It does not accept a workspace or output path.

## Data flow

1. Validate the frozen ADR-0111 contract, resource profile, key, source, and
   helper digests.
2. Fetch one exact HTTPS APK with redirects disabled.
3. Verify size, SHA-256, APKv2 signature, signed data hash, package identity,
   architecture, version, and source commit.
4. Parse only the two bounded regular-file members needed by the guest.
5. Extract the raw ARM64 kernel, inflate the bounded module, compile the
   project-owned PID 1, and build the deterministic initramfs.
6. Measure the exact two mode-`0644` outputs and inspect their formats without
   executing either output.
7. Delete the complete temporary root and only then emit metadata.

## Failure behavior

Every changed digest, unexpected mode, member, tool version, platform,
signature, package field, output identity, or cleanup result fails closed. The
temporary root is removed on both success and failure. No partial receipt is
valid.

## Trust and authority

The rehearsal has narrowly bounded public network and compiler authority. It
has no credential, Apple identity, application, installation, distribution,
VM, analyzer, repository-input, production, or IAR-1B authority. The retained
record is evidence of reproducible materialization and cleanup only.
