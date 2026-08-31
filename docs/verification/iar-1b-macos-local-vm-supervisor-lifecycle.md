# IAR-1B macOS Local-VM Rust-Supervisor Lifecycle Evidence

- Status: Partial synthetic lifecycle passed; full IAR-1B remains pending
- Date: 2026-08-30
- Decision: [ADR-0087](../decisions/0087-macos-local-vm-analyzer-confinement.md)
- Profile: `iar-macos-local-vm-supervisor-v1`
- Profile SHA-256: `f82de32acc12d6cad53c9a8c4a225ea4a352bd91d39222969e9e1efa40035e85`
- Prior checkpoint: [synthetic fault matrix](iar-1b-macos-local-vm-synthetic-matrix.md)

## Scope

This checkpoint connects the existing Rust analyzer-supervisor crate to the
synthetic macOS VM controller without adding a second production child-process
launch site. It accepts only the canonical controller path, its runtime digest,
the canonical asset root, a bounded job identifier, and one of two closed
synthetic lifecycle actions. It accepts no repository path, source bytes,
command, environment, credential, endpoint, parser, or analyzer selection.

The observed ad hoc controller digest for this native run was
`sha256:877fd6c1611fcb4587ee525ddfbf0f7f0c4f8827a3b18fc334bd76a499e1dbe8`.
That value is run-scoped pre-launch evidence, not a durable or atomic release
identity: ad hoc macOS signing can change the controller bytes between builds,
and a path can theoretically be substituted after hashing. Production requires
sealed Developer ID signature and bundle-identity verification.

## Passed Cases

- The Rust supervisor rehashed the exact controller before launching it and
  rejected a mismatched expected digest before job staging.
- External cancellation used one exact job-private `cancel.request` marker.
  The controller stopped the whole VM, returned its closed `cancelled` receipt,
  removed the job, and was reaped by the supervisor.
- Forced termination killed and reaped the exact controller while its
  synthetic non-terminating guest was running. The supervisor removed the
  exact stale job rather than relying on controller cleanup.
- A new fresh VM recovery job completed after each lifecycle action.
- Action, recovery, and identity-rejection job roots were absent before the
  source-free receipts were returned.
- The repository security guard continued to find exactly seven production
  child-process launch sites, including one shared fixed-argv site in the
  analyzer-supervisor crate.

## Claim Boundary

The receipts retain `vm_confined=false`, `production_admitted=false`,
`analyzer_execution=false`, `source_retained=false`, and
`authority_added=false`. This run does not establish recovery across host
sleep, host reboot, power loss, or an operating-system crash. It also does not
establish guest memory pressure, CPU accounting, the complete host-canary
denial corpus, multi-host support, signed distribution, or independent review.

The next checkpoint is guest resource evidence and the host-canary denial
corpus. macOS remains publicly at IAR-1A.
