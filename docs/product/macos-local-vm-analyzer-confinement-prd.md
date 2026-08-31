# Impresari Context — macOS Local-VM Analyzer Confinement PRD

- Status: Synthetic feasibility in progress; storage, fault, lifecycle, resource/canary, simulated-interruption, and offline supply-chain checkpoints passed
- Date: 2026-08-30
- Owner: Aaron Boldt
- Decision: ADR-0087

## Objective

Replace the failed macOS XPC analyzer-execution boundary with a fresh local
Linux virtual machine per analyzer job while preserving local-only operation,
the existing Analyzer Runner protocol, and the one-cask CLI experience.

## User Outcome

A supported macOS user requests a security analysis through the ordinary CLI.
Impresari automatically starts a headless VM on that Mac, exposes only the
explicitly staged read-only job, returns one bounded validated result, and
destroys all per-job guest state. No cloud service or VM administration is
required from the user.

## Feasibility Scope

- Apple Virtualization framework on explicitly supported macOS/architecture
  targets with the public virtualization entitlement.
- One digest-addressed minimal Linux kernel/initramfs/root image with verified
  build provenance and no package manager or interactive login surface.
- Fresh fixed-capacity copy-on-write job storage and no host directory share.
- No virtual network device, host credentials, clipboard, GUI, audio, USB,
  Rosetta, or unrelated host device.
- Bounded read-only input disk and bounded serial or socket result channel.
- Synthetic no-op, crash, timeout, fork, memory, CPU, output, storage,
  cross-job, and cleanup workers only.

## Non-goals

- Cloud execution, repository execution, build/test execution, disposable
  quarantine, real YARA/ClamAV execution, guest package installation, mutable
  shared cache, privileged daemon, kernel extension, or VM persistence.
- Treating a VM boot as proof of IAR-1B or packaging the guest in a public cask
  before the complete synthetic matrix and supply-chain gates pass.

## Acceptance Criteria

- The two failed XPC gates pass first: aggregate job-storage enforcement and
  cross-job state isolation.
- The guest cannot access host home, repository, cache, credentials, devices,
  processes, local network, or Internet.
- Every descendant remains inside guest and host resource supervision.
- Cancellation, crash, timeout, host sleep/interruption, partial result, and
  forced termination leave no mounted disk or per-job state.
- Guest image, runner, resource profile, request, input, result, and receipt
  identities are exact and version-aligned.
- Unsupported virtualization or entitlement state fails closed before input is
  staged.
- Multi-host clean-install evidence, signing/notarization, and independent
  review remain production gates.

## Current Evidence

The first native checkpoint passed on macOS `26.5.1` arm64 with two fresh
local VM jobs. It proved the two requirements the earlier XPC topology failed:
a hard 1 MiB scratch-device capacity and no cross-job marker retention. It also
proved exact read-only synthetic input, an absent guest network device, exact
identity binding, VM stop, and per-job removal. See the
[local-VM feasibility record](../verification/iar-1b-macos-local-vm-feasibility.md).

At that first checkpoint, the complete escape, descendant, CPU, memory,
timeout, crash, interruption,
malformed-result, supply-chain, multi-host, signing/notarization, packaging,
and independent-review gates remain open. The evidence therefore retains
`vm_confined=false` and does not admit a real analyzer.

The next synthetic matrix checkpoint froze the exact guest identity, corrected
the initramfs build to be byte-reproducible, and passed malformed-result,
bounded serial-flood, whole-VM timeout, forked-descendant timeout, early-exit,
controller-cancellation, tampered-guest rejection, cleanup, and post-fault
recovery cases. External-supervisor cancellation, forced host termination,
sleep/interruption, memory/CPU, and the full host-canary corpus remained open
at that checkpoint. See
the [fault-matrix evidence](../verification/iar-1b-macos-local-vm-synthetic-matrix.md).

The Rust-supervisor lifecycle checkpoint then passed exact external
cancellation, forced controller termination, child reaping, stale-job removal,
and a fresh recovery VM after each action without adding another production
process-launch site. A wrong controller digest failed before staging. Host
sleep/interruption, guest memory/CPU, and the full host-canary corpus remained
open at that checkpoint. See the [supervisor lifecycle evidence](../verification/iar-1b-macos-local-vm-supervisor-lifecycle.md).

The resource/canary checkpoint then passed exact guest cgroup v2 memory, CPU,
and process controls plus the frozen six-class host-canary corpus through the
same Rust launch boundary. It retained no job state and added no production or
confinement claim. Host sleep/interruption, sealed supply chain and
distribution, multi-host evidence, and independent review remain open. See the
[resource/canary evidence](../verification/iar-1b-macos-local-vm-resource-canary.md).

The host-interruption checkpoint then installed the macOS will-sleep observer
and proved its shared fail-closed VM-stop, cleanup, and recovery path with a
source-free job-private synthetic trigger. The receipt cannot claim actual
host sleep. A coordinated manual sleep/wake rehearsal, reboot and abrupt
power-loss recovery, sealed supply chain and distribution, multi-host evidence,
and independent review remain open. See the
[host-interruption evidence](../verification/iar-1b-macos-local-vm-host-interruption.md).

The guest supply-chain checkpoint then froze one expiring synthetic-candidate
release manifest, the complete six-component guest inventory, SPDX SBOM,
license record, exact source/build provenance, vulnerability policy, and an
explicit initial rollback state. Its offline checker passed against both the
committed metadata and every already-prepared component on the native host.
The receipt fixes publisher authentication, vulnerability completion,
cryptographic signature, notarization, sealed distribution, production, and
analyzer execution to false. See the
[guest supply-chain evidence](../verification/iar-1b-macos-local-vm-guest-supply-chain.md).

The upstream-authentication checkpoint then verified the complete versioned
Alpine 3.24.1 aarch64 netboot archive under the exact release-key fingerprint
published by Alpine. The signed archive's two embedded guest inputs were
byte-identical to the manifest. The large archive was not committed, routine
checks remain offline, and the guest gains no network. Impresari release-
metadata sealing, vulnerability disposition, Developer ID signing,
notarization, distribution lifecycle, and production admission remain open.
See the [upstream-authentication evidence](../verification/iar-1b-macos-local-vm-upstream-authentication.md).

The bounded vulnerability-review checkpoint then dispositioned that exact
candidate as denied: its `6.18.35-0-virt` kernel is thirteen stable patch
releases behind Alpine 3.24 aarch64 `linux-virt` `6.18.48-r0`, and the
published Alpine secdb record does not establish complete current 6.18
advisory coverage. Replacement is mandatory. The review does not assert CVE
applicability, vulnerability freedom, assessment completion, production
admission, or analyzer authority. See the
[vulnerability disposition](../verification/iar-1b-macos-local-vm-vulnerability-disposition.md).

The current-guest replacement checkpoint then authenticated Alpine's exact
v3.24 aarch64 `APKINDEX` and `linux-virt-6.18.48-r0.apk` through the APK key
extracted from the OpenPGP-authenticated netboot archive. Versioned v2 profiles
atomically bind the replacement kernel, module, initramfses, controller,
supervisor, release records, and rollback predecessor while preserving v1 as
historical evidence. Every native synthetic matrix passed again. The candidate
is current and no further package replacement is presently required, but the
provider snapshot still does not establish complete `6.18` advisory coverage.
Production, analyzer execution, and a vulnerability-free claim remain denied.
See the [current guest replacement](../verification/iar-1b-macos-local-vm-current-guest-replacement.md).

## Packaging Direction

ADR-0076's one CLI-compatible Homebrew cask remains the desired user topology.
Its XPC execution payload is replaced only after the VM backend is admitted.
The guest image may be bundled or explicitly downloaded later; that distribution
choice requires exact signing, expiry, rollback, and offline behavior evidence.
The candidate now has exact expiry, rollback, offline behavior, upstream
publisher-authentication evidence, and a fail-closed vulnerability
disposition. The stale guest has been replaced by the current authenticated
package; complete advisory coverage, Impresari release-metadata sealing,
signing, notarization, and the final bundle/update lifecycle remain open.
