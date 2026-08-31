# IAR-1B macOS Local-VM Host-Interruption Checkpoint

- Status: Partial synthetic checkpoint passed; real host sleep remains open
- Date: 2026-08-30
- Decision: [ADR-0087](../decisions/0087-macos-local-vm-analyzer-confinement.md)
- Host: macOS `26.5.1` arm64

## Result

The macOS VM controller now installs an `NSWorkspace.willSleepNotification`
observer for every synthetic VM job. Operating-system sleep delivery and an
exact job-private synthetic trigger enter the same fail-closed stop handler.
The Rust supervisor exercised the synthetic path through the existing single
audited child-process launch site, verified the complete controller receipt,
reaped the child, confirmed exact job removal, and completed a fresh recovery
VM.

The run passed with ad hoc controller digest
`sha256:7387d96cb0059de368c43e521a6c51c43a7b7fbe1a6c08b3766c613fe3844492`.
The frozen interruption profile is
`sha256:e5f54da3e1fbce7ea7f839dc723e4b288ff7113fd9c85950df3970ae18737fd1`.

## Exact Claim Boundary

This automated run did **not** sleep the host. Its receipt requires:

- `interruption_source=synthetic-job-private-trigger`;
- `sleep_observer_installed=true`;
- `shared_stop_handler_used=true`;
- `virtual_machine_stopped=true`;
- `controller_reaped=true`;
- `recovery_job_succeeded=true`;
- `all_job_state_removed=true`; and
- `real_host_sleep_observed=false`.

The schema rejects a receipt that changes `real_host_sleep_observed` to true.
Therefore this checkpoint proves the bounded stop-and-recovery implementation,
not macOS delivery of a genuine will-sleep event and not behavior across wake,
reboot, power loss, or an operating-system crash.

Every receipt retains `vm_confined=false`, `production_admitted=false`,
`analyzer_execution=false`, `source_retained=false`, and
`authority_added=false`. No repository source, credential, network endpoint,
or real analyzer entered the VM.

## Reproduction

After the exact guest assets have been prepared:

```sh
./scripts/check-macos-vm-host-interruption.sh
```

The command does not request or initiate host sleep. A later manual rehearsal
must deliberately put an otherwise disposable test host to sleep while the VM
job is active and must capture sleep, wake, cleanup, and recovery evidence. It
must run only with explicit operator coordination because it interrupts the
machine and any active development session.

## Remaining Gates

Genuine host sleep/wake, reboot and abrupt power-loss recovery, authenticated
and vulnerability-reviewed guest supply chain, Developer ID/notarized distribution, one-cask lifecycle,
multi-host evidence, and independent human review remain open. macOS remains
publicly at IAR-1A.
