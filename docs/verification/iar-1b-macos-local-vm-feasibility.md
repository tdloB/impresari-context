# IAR-1B macOS Local-VM Feasibility Evidence

- Status: Storage and cross-job checkpoint passed; full IAR-1B remains pending
- Date: 2026-08-30
- Decision: [ADR-0087](../decisions/0087-macos-local-vm-analyzer-confinement.md)
- Profile: `iar-macos-local-vm-feasibility-v1`
- Profile SHA-256: `a082df092d5180058f732d47ae99164316f3bfd3b12f4079de43575834314757`

## Scope

This record covers two consecutive synthetic-only local Linux VM jobs through
Apple's public Virtualization framework. No analyzer, repository content,
hostile artifact, provider, credential, network device, host directory share,
package manager, login, interactive shell, or production signing identity was
used.

The checkpoint was deliberately ordered around the two Tier A requirements the
earlier App Sandbox/private-XPC topology failed:

1. hard aggregate temporary-storage capacity; and
2. fresh cross-job state isolation.

Passing this checkpoint does not set `vm_confined=true`, admit a production
backend, open IAR-2, or authorize YARA.

## Observed Host

| Property | Exact value |
| --- | --- |
| macOS | `26.5.1` (`25F80`) |
| Kernel/architecture | Darwin `25.5.0`, arm64 |
| Xcode | `26.6` (`17F113`) |
| Swift | `6.3.3` |
| Controller signing | ad hoc, virtualization entitlement only |
| Virtualization API | Apple `VZLinuxBootLoader` and `VZVirtualMachine` |

## Exact Guest Supply-Chain Inputs

The explicit preparation command downloaded only two pinned files from the
official Alpine `v3.24/releases/aarch64/netboot` directory and rejected size or
digest drift:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| Alpine `vmlinuz-virt` zboot source | 10,351,104 | `47970e0ee0478fe5c60824a89f162d5a353fa29466e5d3bddb0f9c506f1ed756` |
| Extracted raw ARM64 `Image` | 36,110,336 | `8b216f74e7f89def4604adf69e2345437363aff4819101bb1551c9e83cd35cdd` |
| Alpine `initramfs-virt` module source | 9,385,851 | `e47d38bc88509a3db11affc09f9762f9643b026bd29441724a4729ad8e97add6` |
| Extracted `virtio_blk.ko` | 49,687 | `80341fdb0869f5df4813b7bfb4a1cd77d2f6cd7c26c04fc15706cbc44d680ef6` |
| Impresari-built static synthetic PID 1 | 31,648 | `ad41fec15562f9d30a8888c468c599d7e33925b8aadcb58f41e082072ddfb249` |
| Impresari-built initramfs | 34,669 | `592073c3ee8e7d3d497102f57da16e0e6b31524fdff482b754fe92656bccaa60` |
| Ad hoc signed Swift controller | local build | `8ba1902a91c6302d33066693898c8dd0070d332781d3f3308cc110146552dfd4` |

The source zboot wrapper initially produced Apple's generic fail-closed
`VZErrorDomain` code `1` before guest start. The preparation step was corrected
to validate the ARM64 zboot header, extract its bounded gzip payload, verify the
raw `ARM\x64` image marker, and pin the extracted image digest. No weaker start
or fallback path was added.

## Effective Device And Resource Topology

Each job used:

- one virtual CPU and 256 MiB memory;
- one serial result channel from the fixed synthetic guest (a hard host-side
  capture bound was not yet proved at this checkpoint);
- one exact 4 KiB read-only raw input disk;
- one fresh 1 MiB writable raw scratch disk; and
- zero network, host-directory, graphics, audio, keyboard, pointing, USB, or
  other configured devices.

The controller binary carried only the public
`com.apple.security.virtualization` entitlement. It carried no network
entitlement.

## Two-Job Result

Both `job-one` and `job-two` returned complete receipts bound to the exact
profile, raw kernel, initramfs, and synthetic input identities. For each job:

- Virtualization support and configuration validation passed.
- The guest read the exact synthetic input and an `O_RDWR` open against the
  read-only attachment was denied.
- The fresh scratch disk began with no prior marker.
- The guest filled the complete 1 MiB device and a write beyond the device end
  did not succeed.
- The guest observed no network interface other than loopback.
- The host rehashed the unchanged input and verified the scratch file remained
  exactly 1 MiB.
- The VM stopped and the exact per-job directory was removed.

The second fresh VM's `scratch_initially_clean=true` result is the cross-job
canary: the marker written by the first guest was not observable in the second
guest. Both source-free receipts retained `vm_confined=false`,
`production_admitted=false`, `source_retained=false`, and
`authority_added=false`.

## Repeat

On a supported Apple-silicon Mac with Xcode, Zig, Ruby, `curl`, `cpio`, `jq`,
and the code-signing tools available:

```sh
./scripts/prepare-macos-vm-feasibility.sh
./scripts/check-macos-vm-feasibility.sh
```

Preparation is the only networked step and writes ignored build artifacts
under `target/`. The check itself runs from the already verified local assets.
On other platforms it reports the native check as not applicable.

## Remaining IAR-1B Gates

This first checkpoint does not yet prove:

- home, repository, cache, credential, unrelated-process, and host-device
  canary denial through the complete Tier A corpus;
- guest descendant containment and guest/host CPU, memory, timeout, output,
  crash, cancellation, sleep/interruption, and forced-stop behavior;
- malformed disk, malformed or partial result, guest panic, and controller
  crash handling;
- reproducible guest-image build, SBOM, full license/vulnerability policy,
  expiry refresh, rollback, or offline distribution behavior;
- Developer ID signing, notarization, clean installation, upgrade, uninstall,
  Gatekeeper, one-cask packaging, or automatic updates; or
- evidence on more than this one macOS/architecture/toolchain target and the
  deferred attributable independent human review.

macOS therefore remains publicly at IAR-1A. The subsequent
[partial synthetic fault matrix](iar-1b-macos-local-vm-synthetic-matrix.md)
closes the exact-guest, reproducibility, bounded-serial, malformed-result,
timeout/descendant, controller-cancellation, and recovery cases while retaining
the remaining gates—not real analyzer execution or packaging.
