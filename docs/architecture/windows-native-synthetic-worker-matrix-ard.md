# Windows Native Synthetic Worker Matrix ARD

- Status: Accepted for implementation under ADR-0088
- Date: 2026-08-31
- Governing PRD: [Windows Native Synthetic Worker Matrix PRD](../product/windows-native-synthetic-worker-matrix-prd.md)
- Decision: [ADR-0093](../decisions/0093-windows-native-synthetic-worker-matrix.md)

## Architecture

```text
fixed synthetic matrix profile
             |
             v
Windows broker -> fresh AppContainer SID -> exact ACL/canary staging
             |             |
             |             +-> profile storage write removed
             v
pre-limited Job Object + STARTUPINFOEX
  LPAC + zero capabilities + exact handles + mitigations + child deny
             |
      CreateProcess suspended
             |
       assign job, then resume
             v
first-party boundary worker -> bounded pipes -> closed receipt
             |
     terminate/reap/remove -> second fresh-identity cross-job probe
```

The broker and worker are fixed reviewed sources. They accept no repository
path, repository bytes, analyzer, rule set, endpoint, arbitrary command,
caller-supplied environment, or credential. Native execution occurs only in a
dedicated hosted Windows job; non-Windows validation is source-free.

The child environment is rebuilt rather than inherited. It retains only five
default current-user path keys obtained with `CreateEnvironmentBlock` using
`bInherit=false`, derives Windows loader paths from `GetWindowsDirectoryW`, and
binds `TEMP`/`TMP` to the exact staged job. Process-level CI, repository,
credential, proxy, and arbitrary user variables cannot cross the boundary.

## Launch Order

1. Verify the exact target, base profile, matrix profile, and worker identity.
2. Create one unique zero-capability AppContainer profile and derive its SID.
3. Locate the profile directory and remove path-backed write access for the
   AppContainer SID; failure is unsupported.
4. Stage one worker per fresh profile plus original-synthetic input and
   host-only canaries with exact read/execute DACLs compatible with LPAC.
5. Create an unnamed Job Object and set/query kill-on-close, active-process,
   process/job memory, and CPU controls with breakaway absent.
6. Build one `STARTUPINFOEX` attribute list containing exact
   `SECURITY_CAPABILITIES`, LPAC all-application-packages opt-out, handle list,
   child-process restriction, and compatible frozen mitigations.
7. Create the worker suspended, assign it to the Job Object, verify assignment,
   and only then resume its primary thread.
8. Collect only bounded pipe output, validate the closed receipt, terminate the
   job on every fault, and converge on one idempotent cleanup path.
9. Verify zero active processes and remove exact synthetic state before the
   second fresh-identity cross-job probe.

## Boundary Corpus

The worker receives a canonical source-free control frame over its inherited
input pipe and returns one bounded canonical result frame. Scenarios cover:

- positive exact-input read;
- input mutation, worker mutation, sibling read, user-profile canary read,
  profile-storage write, and synthetic HKCU canary read denial;
- broker-owned loopback connection denial;
- unrelated inheritable-handle and unrelated-process access denial;
- child/breakaway denial and active-process accounting;
- CPU, effective process-memory, timeout, output-flood, crash, cancellation,
  and malformed-result handling;
- exact cleanup and two-identity cross-job isolation.

No external network destination or existing credential store is touched.

## Resource Semantics

The base profile allows one process, limits that process to 64 MiB, and limits
the job to 128 MiB. With one active process, the 128 MiB aggregate threshold
cannot be independently exhausted. The broker must query both configured
limits and exercise the lower 64 MiB effective boundary. Evidence must not
claim an independently exercised job-memory threshold.

## Failure And Cleanup

Every error path stops control input, terminates the Job Object, waits for zero
active processes, closes process/thread/pipe/job handles, removes exact staging
and canaries, deletes the exact AppContainer profile, and verifies absence.
Uncertain cleanup is a failed result, never a successful or application-only
fallback.

## Claim Boundary

The first worker-matrix receipt fixes `os_confined=false` even when every
scenario passes. A later decision must evaluate independent-host repeatability,
compatibility withdrawal, signing, lifecycle, and production admission before
Windows can claim IAR-1B or run a real analyzer.
