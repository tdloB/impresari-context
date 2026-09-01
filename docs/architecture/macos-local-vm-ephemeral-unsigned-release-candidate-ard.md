# macOS Local-VM Ephemeral Unsigned Release Candidate Architecture

- Status: Accepted for implementation under ADR-0114
- Date: 2026-09-01
- Governing PRD: [macOS Local-VM Ephemeral Unsigned Release Candidate PRD](../product/macos-local-vm-ephemeral-unsigned-release-candidate-prd.md)
- Decision: [ADR-0114](../decisions/0114-build-assemble-verify-and-delete-the-macos-unsigned-candidate.md)

## Architecture

```text
exact Git object + offline Cargo/Swift toolchains
                         |
exact authenticated Alpine APK + frozen guest builders
                         |
                         v
              fresh private root (0700)
              | source | build | app |
                         |
                         v
              closed eight-file app tree
               |                 |
       ADR-0109 identity   ADR-0113 identity
               |                 |
                         v
                  delete whole root
                         |
                         v
                  metadata-only record
```

The operator script accepts no path or credential argument. It obtains the
exact candidate source from the local Git object database, verifies the archive
digest, builds the three locked Cargo units offline and the Swift controller in
private caches, and reconstructs the guest using the already frozen APKv2
authentication chain. Network access is limited to the exact public APK.

The app tree contains exactly four product executables, `Info.plist`, the guest
metadata seal, `Image`, and `impresari-initramfs.gz`, plus their fixed parent
directories. The executable outputs are inspected but never launched.

## Identity separation

- The frozen ADR-0109 candidate compound identity uses its documented
  source/version plus sorted path/bytes/SHA-256 canonicalization.
- The ADR-0113 material identity additionally binds target, file kind, and
  filesystem mode.

Both must reproduce. Neither is a Developer ID signature, notarization ticket,
published archive identity, or runtime-isolation result.

## Failure behavior

Wrong platform, toolchain, source, archive, APK authentication, build output,
tree member, mode, identity, or cleanup fails closed. The complete temporary
root is removed from an `ensure` path on success and failure. No partial
candidate record is accepted.
