# Windows Native Analyzer Confinement ARD

- Status: Accepted for synthetic feasibility
- Date: 2026-08-30
- Governing PRD: [Windows Native Analyzer Confinement PRD](../product/windows-native-analyzer-confinement-prd.md)
- Decision: [ADR-0088](../decisions/0088-windows-native-analyzer-confinement.md)

## Architecture

```text
Rust supervisor
      |
      v
signed Windows broker
      |
      +-- unique LPAC/AppContainer identity
      +-- suspended synthetic worker
      +-- Job Object + mitigations + closed handles
      +-- read-only ACL input + bounded pipes
      v
resume -> validate result -> kill job -> remove profile/staging
```

The broker is separately packaged from Context and accepts only the closed
Analyzer Runner request/profile. It creates the worker suspended, applies the
complete security token, capability, mitigation, handle, and Job Object state,
then resumes it. No command, environment, path outside private staging, or
repository-supplied rule enters the launch contract.

## Access Boundary

- A per-job AppContainer SID is granted read access only to staged exact input
  and execute access only to the pinned worker/analyzer.
- No network capability is granted.
- No writable host path is granted; result transport uses size-bounded pipes.
- Inherited handles are closed except exact protocol descriptors.
- Win32k/UI and other compatible process mitigations are enabled and recorded.

## Resource And Cleanup Boundary

- Job Objects enforce active process, CPU, memory, wall-time supervision, and
  kill-on-close for the complete descendant tree.
- Breakaway flags are prohibited and tested.
- Cleanup orders: stop input, terminate job, wait for zero processes, close
  handles, remove staging ACL state, delete the per-job AppContainer profile,
  then verify cross-job canaries.

## Verification

- Frozen tests cover every capability denial, malformed request/result,
  descendant/breakaway attempt, handle inheritance, resource exhaustion,
  timeout, crash, cancellation, reboot residue, profile collision, and cleanup.
- Exact Windows build and architecture receipts expire and withdraw on drift.
- Production admission requires signed clean-install, upgrade, rollback, and
  uninstall evidence on every claimed target.
