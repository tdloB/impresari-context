# IAR-1B macOS Local-VM Current Guest Replacement

- Status: Current authenticated synthetic candidate passed; full IAR-1B remains pending
- Date: 2026-08-31
- Decision: [ADR-0087](../decisions/0087-macos-local-vm-analyzer-confinement.md)

## Result

The denied Alpine `6.18.35-0-virt` guest was replaced between jobs by the
current Alpine 3.24 aarch64 `linux-virt` package `6.18.48-r0`. The active v2
profiles, controller, Rust supervisor, release manifest, SBOM, license record,
build provenance, rollback predecessor, publisher-authentication record, and
repeated vulnerability review all bind the same replacement identity. The v1
profiles and receipts remain unchanged as historical evidence.

The replacement is still synthetic-only. It does not enable a real analyzer,
claim production admission, or establish complete vulnerability coverage.

## Authentication chain

The exact Alpine netboot archive previously authenticated by OpenPGP contains
the exact `alpine-devel@lists.alpinelinux.org-616ae350.rsa.pub` key. That key
verified both the exact v3.24 aarch64 `APKINDEX.tar.gz` and the exact
`linux-virt-6.18.48-r0.apk`. The signed APK control metadata binds the exact
package data gzip stream through its SHA-256 `datahash`.

| Object | Bytes | SHA-256 |
| --- | ---: | --- |
| APK signing key | 800 | `d11f6b21c61b4274e182eb888883a8ba8acdbf820dcc7a6d82a7d9fc2fd2836d` |
| APKINDEX snapshot | 529,311 | `db44420861bbe4b2ae28756f8fceee9ced313a8585c56fed85dc7667b722d0fc` |
| `linux-virt-6.18.48-r0.apk` | 41,557,960 | `c9ec62df20409d06f201cea7355140d5f99d421629ad35e9a023621a3c881616` |
| APK signed data stream | — | `e2ec28de6d80fa2b3535fc29475a7657ed8375dec99d4da96871ffd5b1077263` |

APKv2 uses the provider's legacy RSA PKCS#1 v1.5/SHA-1 signature format. The
record treats that as Alpine publisher authentication, explicitly discloses
the legacy algorithm, and adds HTTPS plus complete SHA-256 pins. It does not
misrepresent APKv2 as Impresari distribution signing or Apple notarization.

## Active guest identity

| Component | Bytes | SHA-256 |
| --- | ---: | --- |
| Extracted ARM64 `Image` | 36,175,872 | `4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5` |
| Extracted `virtio_blk.ko` | 49,687 | `c8eb0f6b98a18a5cc237bc3019637551f46f964a5efd215253a0946889e3f31d` |
| Synthetic initramfs | 38,207 | `89c50636f21054dfcfd1761a1bfcf613df302960317876b3e137e1267b45397b` |
| Resource/canary initramfs | 50,047 | `1a4029b781020260e4cb8c18271e3a01e1920f1448d87a71678e12cc617a1ec3` |
| Signed local controller | — | `b5d19e052844b6b7a47bbd39ce277438161b36f646818230ccd0c0fb1cf67441` |

The exact package/index verification command is:

```sh
./scripts/verify-macos-vm-alpine-package.sh \
  /absolute/path/alpine-netboot-3.24.1-aarch64.tar.gz \
  /absolute/path/APKINDEX.tar.gz \
  /absolute/path/linux-virt-6.18.48-r0.apk
```

The verifier performs no retrieval. Callers must supply the three exact local
files explicitly.

## Native evidence

On macOS arm64, the v2 candidate passed:

- three successful clean VM jobs and seven fail-closed fault jobs;
- tampered-identity rejection before staging;
- exact cgroup v2 memory, CPU, and process accounting in the resource guest;
- all six host-canary absence checks;
- external cancellation, forced controller termination, exact reap, cleanup,
  and recovery;
- the synthetic host-interruption stop/cleanup/recovery path;
- repeated deterministic initramfs construction; and
- the complete repository validation suite.

## Vulnerability disposition

The candidate now exactly equals the current authenticated Alpine package, so
`candidate_current=true` and `replacement_required=false`. Alpine's published
v3.24 `linux-lts` secdb snapshot still does not enumerate a complete current
`6.18` advisory surface. Therefore:

- `advisory_coverage_complete=false`;
- `vulnerability_assessment_complete=false`;
- `production_admitted=false`; and
- `analyzer_execution=false`.

The replacement closes the stale-guest defect. It does not make a
vulnerability-free claim.

## Remaining gates

The subsequent ADR-0091 checkpoint now content-addresses the exact active
release metadata. Complete advisory coverage and disposition, genuine
sleep/wake plus reboot and power-loss evidence, multi-host evidence, Developer
ID signing/notarization, the one-cask lifecycle, final publication attestation,
and the deferred independent human security review remain required before
macOS IAR-1B or any real-analyzer admission.
