# IAR-1B macOS Local-VM Unsigned Synthetic Bundle Assembly Evidence

- Date: 2026-09-01
- Decision: [ADR-0108](../decisions/0108-assemble-a-non-runnable-macos-bundle-before-signing.md)
- Scope: source-free, offline, non-runnable temporary assembly only

## Exact identities

- profile: `iar-macos-local-vm-unsigned-bundle-assembly-v1`
- profile SHA-256: `0d661ae58e579d899325b130a0de597e5e82075331bef17ee4430f280a3db3eb`
- assembly: `iar-macos-local-vm-unsigned-bundle-2026-09-01.1`
- assembly-spec SHA-256: `36978dfd1f475d219ed7168d7f00c17fca1dcd5951e771e6dd81a5cfff7058d9`
- package-contract SHA-256: `4f249a15c1cd0b5283c937d49cc1888c3ab56b2a9a22847b8913901c72d5f676`
- metadata-seal SHA-256: `c0294a88c2c7fe1d33bdd8ddfbb55e26e6595f02c12a9645c898f36148aa82e1`
- canonical tree SHA-256: `ace9ff8230be69e0df6a8e7977fde6cf82a8ecb9221be841f49718a4c6f79813`

## Proven

- two private temporary assemblies produced the same 13-entry tree;
- every path, kind, mode, byte count, and digest matched the closed spec;
- the synthetic `Info.plist` and ADR-0091 seal copy were exact;
- all apparent executable payloads were non-executable text markers;
- symlinks and special files were absent; and
- both temporary roots were removed before the receipt was accepted.

The target macOS/POSIX run additionally verifies exact `0700` temporary-root
mode. Windows CI covers structure, determinism, non-symlink roots, and cleanup
only; Windows mode bits are not macOS privacy evidence.

## Not proven

No release app, archive, cask, install, source-revision binding, publication
attestation, Developer ID signature, notarization, Homebrew lifecycle, VM,
analyzer, production, or macOS IAR-1B claim is established.
