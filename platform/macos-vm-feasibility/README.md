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

The separate ADR-0091 metadata-seal check now closes that repository-metadata
sub-gate. It verifies one exact sixteen-member v2 metadata/profile inventory
and canonical set digest without network, credentials, prepared executable
assets, or source input. It does not verify a GitHub publication attestation,
Developer ID signature, Apple notarization, Homebrew cask lifecycle, sealed
distribution, production admission, or analyzer execution.

ADR-0111 additionally closes the future ordinary app guest payload to only
`Image` and `impresari-initramfs.gz`. Its offline checker binds those two
runtime identities, excludes the resource-test and standalone build
intermediates, and freezes a later authenticated private-root build-and-delete
recipe. The contract does not download or build the guest and does not admit a
release, VM, analyzer, production backend, or macOS IAR-1B.

ADR-0112 performs that one bounded operator rehearsal. It downloads only the
exact authenticated Alpine APK, extracts only the two required regular-file
inputs, rebuilds the ordinary synthetic guest with Zig 0.16.0, confirms the
exact two ADR-0111 payload identities, and deletes the complete private root
before retaining metadata. The guest was not executed or retained. This does
not assemble an app, access Apple identity, sign, notarize, install or publish
a cask, launch a VM, execute an analyzer, bind a release, admit production, or
admit macOS IAR-1B.

ADR-0113 composes the retained product, guest, metadata-seal, and deterministic
`Info.plist` identities into one closed eight-file prospective app projection
and compound digest. It deliberately does not satisfy the materialized
unsigned-candidate schema: the product and guest existed in separate deleted
rehearsals, no complete app tree existed, and candidate filesystem modes were
not verified. The next gate is one complete ephemeral build, assembly,
verification, and deletion rehearsal without Apple identity or execution.

ADR-0114 completes that one-root rehearsal. The exact product source archive
was rebuilt offline, the exact public Alpine APK was publisher-authenticated,
the four product and two guest identities were reproduced, and the closed
eight-file app tree was assembled with exact modes. Both the ADR-0109 compound
identity and ADR-0113 material identity reproduced. No produced artifact was
executed, and the source, download, app, build products, caches, and raw logs
were deleted with the complete private root. The retained record is an unsigned
candidate, not a signed, notarized, installed, sandboxed, or production app.

ADR-0115 freezes the next manual Developer ID and Apple notarization rehearsal
without crossing that boundary. Its source-free contract fixes the exact
synthetic candidate, inside-out signing order, hardened runtime, secure
timestamp, controller entitlement, strict verification, Keychain-reference
only notarization, accepted-log review, stapling, Gatekeeper assessment, final
archive recreation, and whole-root cleanup. The checker accesses no Keychain,
launches no process, contacts no service, and creates or retains no executable
or archive. Live signing and notarization still require founder action; cask,
installation, publication, VM, analyzer, production, and IAR-1B remain false.
