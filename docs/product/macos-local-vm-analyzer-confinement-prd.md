# Impresari Context — macOS Local-VM Analyzer Confinement PRD

- Status: Synthetic feasibility in progress; storage/cross-job and partial fault matrix passed
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
sleep/interruption, memory/CPU, and the full host-canary corpus remain open. See
the [fault-matrix evidence](../verification/iar-1b-macos-local-vm-synthetic-matrix.md).

## Packaging Direction

ADR-0076's one CLI-compatible Homebrew cask remains the desired user topology.
Its XPC execution payload is replaced only after the VM backend is admitted.
The guest image may be bundled or explicitly downloaded later; that distribution
choice requires exact signing, expiry, rollback, and offline behavior evidence.
