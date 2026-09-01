# macOS Synthetic Guest Materialization Verification

- Decision: ADR-0112
- Date: 2026-09-01
- Host: macOS arm64
- Toolchain: Zig 0.16.0
- Result: exact synthetic guest materialized, verified, never executed, and deleted

## Verified

- One exact public Alpine APK was fetched with redirects disabled.
- The pinned size, SHA-256, APKv2 publisher signature, signed data hash,
  package name, version, architecture, and source commit matched.
- Only `boot/vmlinuz-virt` and the exact compressed `virtio_blk` module were
  selected as bounded regular files.
- The project-owned ordinary guest init was compiled for
  `aarch64-linux-musl` with the frozen static Zig arguments.
- `Image` matched 36,175,872 bytes and SHA-256
  `4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5`.
- `impresari-initramfs.gz` matched 38,207 bytes and SHA-256
  `89c50636f21054dfcfd1761a1bfcf613df302960317876b3e137e1267b45397b`.
- Both outputs were mode `0644`, format-inspected, and never executed.
- The download, selected inputs, build output, caches, raw child output, and
  private root were deleted before the metadata record was emitted.

## Not verified or claimed

No app was assembled. No Apple credential or identity was accessed. No guest
artifact was retained, signed, notarized, installed, or published. No cask was
created. No VM or analyzer ran. No release identity, production admission, or
macOS IAR-1B admission follows from this evidence.

## Repeatability

The materializer is intentionally not part of ordinary CI because it performs
one authenticated public download and creates temporary runnable bytes. CI
runs only the offline digest-bound checker and schema conformance fixtures.
Repeating the materializer is a new operator rehearsal and must retain only a
new reviewed metadata record after complete cleanup.
