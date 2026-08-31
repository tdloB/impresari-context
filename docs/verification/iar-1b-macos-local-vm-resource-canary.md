# IAR-1B macOS Local-VM Resource And Host-Canary Evidence

- Status: Partial synthetic resource/canary checkpoint passed; full IAR-1B remains pending
- Date: 2026-08-30
- Decision: [ADR-0087](../decisions/0087-macos-local-vm-analyzer-confinement.md)
- Profile: `iar-macos-local-vm-resource-canary-v1`
- Profile SHA-256: `b711c69b7a46ad26bb7181622edc69366557886cfe43ef3ca2ef05283d861e7e`
- Prior checkpoint: [Rust-supervisor lifecycle](iar-1b-macos-local-vm-supervisor-lifecycle.md)

## Scope

This checkpoint uses a separately frozen synthetic ARM64 guest so the prior
matrix image identity is not rewritten. The Rust supervisor accepts only the
canonical controller, its pre-launch digest, the canonical asset root, and one
bounded job ID. It launches one fixed `resource-canary` scenario through the
runner crate's existing single audited process-launch site.

The host creates six original-synthetic canary files for home, repository,
cache, credential, device, and process classes beside—but never inside—the two
attached raw disks. The VM still receives exactly one read-only 4 KiB input
disk, one 1 MiB scratch disk, no directory share, and no network device.

## Passed Cases

- The resource initramfs reproduced exact digest
  `sha256:f75a3bc10d569622f84c557e88bbc9ce65a157e7bb410f412c8ab39dedc5c80c`.
- The guest observed exactly `vda` and `vdb`; all six host-canary markers were
  absent from both devices, prohibited host path families were absent, and no
  guest `/proc` command line exposed the host controller identity.
- The host canary corpus remained byte-exact after VM stop and the complete
  host job directory was removed.
- A guest cgroup v2 leaf enforced `memory.max=33554432`,
  `memory.swap.max=0`, `cpu.max=10000 100000`, and `pids.max=8`.
- A 128 MiB child triggered exactly one cgroup OOM kill while PID 1 and the VM
  remained responsive.
- A CPU-bound child was throttled for 16 periods and charged 154189
  microseconds during the bounded native run; the Rust validator accepts only
  50000–400000 canonical microseconds and at least one throttled period.
- The job cgroup observed a peak of one child, was killed empty, and was
  removed before the receipt returned.
- Wrong controller identity failed before staging. No job state remained.

The run-scoped ad hoc controller digest was
`sha256:fe53b022b7a3aa04effed1ea56035d4464f7297b4a4b8d976c4b2a8ef2b632fb`.
As in the prior checkpoint, it is a pre-launch measurement rather than atomic
sealed release identity.

## Claim Boundary

The receipt retains `vm_confined=false`, `production_admitted=false`,
`analyzer_execution=false`, `source_retained=false`, and
`authority_added=false`. The canaries are synthetic and no repository,
credential, device, or user source was read or staged. This checkpoint does not
prove host sleep/reboot/power-loss recovery, sealed Developer ID identity,
guest supply-chain update policy, multi-host compatibility, signed cask
lifecycle, real-analyzer behavior, or independent review.

The next dependency was the bounded host-interruption implementation. Its
synthetic checkpoint is recorded separately and does not replace genuine
sleep/wake evidence. Genuine host sleep/wake and restart recovery are followed
by sealed supply-chain/distribution and multi-host evidence. macOS remains
publicly at IAR-1A.
