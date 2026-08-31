# macOS Local-VM Feasibility Prototype

This directory contains the synthetic-only ADR-0087 prototype. It is not a
production analyzer backend and is not included in release artifacts.

The prototype uses Apple's public Virtualization framework to boot a pinned
Alpine ARM64 virtual kernel with an Impresari-built initramfs. The initramfs
contains only a fixed synthetic PID 1 and the exact `virtio_blk` module needed
by the pinned kernel. It has no package manager, login, interactive shell,
network configuration, repository parser, or analyzer.

The VM configuration exposes exactly:

- one serial result channel;
- one read-only 4 KiB synthetic input disk; and
- one fresh, fixed-capacity 1 MiB scratch disk.

It exposes no network adapter, host directory share, graphics, audio, input,
USB, clipboard, Rosetta, or persistent disk. Each controller invocation creates
and removes its own private job directory.

Run the explicit preparation step once, then the offline check:

```sh
./scripts/prepare-macos-vm-feasibility.sh
./scripts/check-macos-vm-feasibility.sh
./scripts/check-macos-vm-supervisor-lifecycle.sh
./scripts/check-macos-vm-resource-canary.sh
./scripts/check-macos-vm-host-interruption.sh
ruby ./scripts/check-macos-vm-guest-supply-chain.rb \
  --prepared-assets target/iar-macos-vm-feasibility
./scripts/verify-macos-vm-alpine-archive.sh \
  /absolute/path/alpine-netboot-3.24.1-aarch64.tar.gz
```

Preparation downloads only the two exact official Alpine artifacts listed in
`guest-assets.json` and rejects either unless its size and SHA-256 match. The
guest never receives a network device.

The check now runs two ordinary fresh jobs, deterministic malformed-result,
bounded output-flood, timeout, forked-descendant timeout, early-exit, and
controller-cancellation cases, an exact tampered-guest rejection, and a final
recovery job. It builds the initramfs twice and rejects non-identical output.
The separate supervisor-lifecycle check then runs exact external cancellation
and forced controller termination through the Rust runner crate, proves child
reaping and exact stale-job removal, and completes a fresh recovery VM after
each action. The separate resource/canary check builds a second frozen guest,
proves exact guest cgroup v2 memory/CPU/process enforcement, and exercises six
host-only canary classes through the same Rust launch boundary. The separate
host-interruption check installs the macOS will-sleep observer, drives its
shared fail-closed VM-stop path with an exact job-private synthetic trigger,
reaps the controller, removes both job roots, and completes a fresh recovery
VM. It deliberately records `real_host_sleep_observed=false`: it does not put
the Mac to sleep and cannot replace a later manual operating-system sleep
rehearsal. Real sleep/reboot/power-loss evidence, sealed supply-chain and
distribution, multi-host, and independent-review evidence remain future gates.

The offline guest supply-chain check freezes the candidate release manifest,
component inventory, SPDX SBOM, license record, source/build provenance,
vulnerability policy, expiry, and explicit initial rollback state. With
`--prepared-assets`, it also rehashes and measures every built guest component.
It performs no download and the guest remains unable to update itself. This
checkpoint does not authenticate the upstream publisher, complete a current
vulnerability assessment, verify a Developer ID signature or notarization, or
admit a sealed production distribution.

The separate explicit Alpine-archive check closes only the upstream
publisher-authentication sub-gate. It verifies the exact 3.24.1 archive using
the frozen detached signature and Alpine release-key fingerprint, then extracts
only the two required members and proves their exact correspondence to the
guest manifest. The 411 MB archive stays outside the repository. This check
does not seal Impresari release metadata or alter the remaining production
gates.
