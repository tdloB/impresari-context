# macOS Local-VM Analyzer Confinement ARD

- Status: Synthetic feasibility in progress; partial simulated-interruption checkpoint passed
- Date: 2026-08-30
- Governing PRD: [macOS Local-VM Analyzer Confinement PRD](../product/macos-local-vm-analyzer-confinement-prd.md)
- Decision: [ADR-0087](../decisions/0087-macos-local-vm-analyzer-confinement.md)

## Architecture

```text
Context exact manifest
        |
        v
Rust supervisor -> signed VM controller -> fresh local Linux guest
        |                                      |
        |                              read-only input disk
        |                              no network device
        |                              bounded result channel
        +<----------- validated result --------+
                         |
                         v
                 destroy job overlay
```

The Context core never invokes Virtualization APIs. A separately packaged
macOS controller receives the existing source-free runner request and fixed
profile. The Rust supervisor validates controller, guest-image, and profile
identities before creating any job artifact.

## Guest Boundary

- Direct Linux boot with a minimal immutable root image.
- One fixed read-only input image containing only manifest-selected exact bytes.
- One fixed-capacity ephemeral overlay or scratch block device.
- No host directory sharing and no network interface.
- Fixed CPU and memory at VM construction; guest cgroup/seccomp/Landlock
  defense-in-depth where supported by the pinned guest kernel.
- One framed result channel with byte and time limits; no general shell or RPC.

## Lifecycle

1. Verify controller signature, entitlement, guest image, and profile.
2. Create private job directory and fixed-capacity disks.
3. Rehash staged input before VM start.
4. Boot and require an exact guest handshake before the deadline.
5. Run only the synthetic worker during feasibility.
6. Validate complete result, terminate VM, detach devices, and remove job state.
7. Verify no source canary survives into the next fresh job.

## Supply Chain

The guest kernel, initramfs, root image, and every included executable have a
reproducible manifest, SBOM, license record, source provenance, vulnerability
policy, expiry, and rollback identity. The VM cannot update itself. Image
updates occur only between jobs through separately reviewed release metadata.

## Verification

- Original IAR synthetic corpus plus VM boot, malformed disk, malformed result,
  guest panic, controller crash, host cancellation, storage exhaustion, and
  cross-job canary cases.
- Packet capture or absent-interface evidence for network denial.
- Host filesystem and process canaries for every prohibited boundary.
- Clean install, upgrade, rollback, uninstall, Gatekeeper, and notarization on
  every claimed macOS target before production admission.

The first implementation checkpoint now boots an exact raw ARM64 Linux kernel
and Impresari-built synthetic initramfs through an ad hoc signed Swift
controller. Two consecutive fresh guests passed read-only input, fixed scratch
capacity, absent-network-device, cross-job canary, stop, and exact job-removal
checks. The controller remains a feasibility adapter; it is not wired into the
Rust production runner and cannot claim `vm_confined` or production admission.

The second checkpoint replaces disk-backed serial capture with a bounded
memory-only drain, freezes the guest initramfs identity inside the controller,
and adds deterministic malformed-result, output-flood, whole-VM timeout,
forked-descendant, early-exit, controller-cancellation, identity-rejection, and
post-fault recovery paths. The external Rust-supervisor, forced-termination,
sleep, memory/CPU, and complete host-canary gates remained unimplemented at
that checkpoint.

The third checkpoint routes the controller through the Rust runner crate's one
existing audited child-process launch site. The supervisor verifies the ad hoc
controller digest immediately before launch, waits for an exact job-private readiness marker,
delivers cancellation through an exact job-private marker, or kills and reaps
the controller for the forced-termination case. It owns exact stale-job removal
and requires a fresh recovery VM after either action. It does not yet connect
the Analyzer Runner request/result protocol or admit a real analyzer. This
pre-launch digest check does not close executable substitution; production
requires sealed signature and bundle identity verification.

The fourth checkpoint freezes a separate resource-test initramfs instead of
rewriting the earlier matrix identity. The guest PID 1 creates one cgroup v2
leaf with exact memory, swap, CPU, and process ceilings, runs only synthetic
pressure children, validates kernel accounting, kills the empty leaf, and
removes it. Six host-only canary classes are created beside but never attached
to the guest disks. Guest device, raw-disk marker, prohibited-path, and process-
identity observations are combined with host byte-integrity and exact cleanup.
The Rust supervisor validates the complete source-free receipt. This closes
only the guest-resource and host-canary checkpoint.

The fifth checkpoint adds a macOS will-sleep observer to the controller. The
observer and an exact job-private synthetic trigger invoke one locked,
first-event-wins stop handler. The controller stops the VM and removes its job;
the Rust supervisor validates the synthetic source, reaps the controller,
requires both exact roots absent, and boots a fresh recovery VM. The profile
and schema freeze `real_host_sleep_observed=false`, so this automated path
cannot satisfy the genuine sleep/wake gate. No automatic sleep command is
included in routine checks.
