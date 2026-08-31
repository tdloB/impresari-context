# IAR-1B Windows Native Synthetic Worker Matrix

- Status: Hosted native launch attempted; exact host unsupported
- Date: 2026-08-31
- Decisions: [ADR-0088](../decisions/0088-windows-native-analyzer-confinement.md), [ADR-0092](../decisions/0092-freeze-windows-native-feasibility-contract.md), [ADR-0093](../decisions/0093-windows-native-synthetic-worker-matrix.md)

## Checkpoint

This checkpoint advances the successful no-worker preflight to one exact
first-party synthetic boundary worker. The worker must be created suspended
inside a fresh zero-capability LPAC/AppContainer, assigned to the fully
configured Job Object before resume, and given only three exact protocol
handles plus exact synthetic staging access.

The frozen matrix profile is
`iar-windows-native-synthetic-worker-matrix-v1`, digest
`sha256:82ab5c5c0cff76079ae19925b92da23b2d86e3a31e7cfc58626e17cb01c14678`.
It remains bound to the ADR-0092 base profile digest
`sha256:6b8f614387fc97321497e6b725213b9ee3c2159f3d1384fb800ffbe8af490a73`.

## Required Evidence

- exact LPAC/AppContainer SID and zero-capability token identity;
- AppContainer profile-storage write denial, exact staging access, and denied
  sibling/user-profile/registry canaries;
- no external network contact and denied broker-owned loopback connection;
- exact handles, child denial, no breakaway, and active-process peak one;
- exact Job Object query plus effective CPU/process-memory, timeout, output,
  crash, cancellation, and malformed-result fault handling;
- complete idempotent cleanup and a second fresh-identity cross-job denial.

## Contract Evidence

Two closed schemas, a digest-bound profile, positive unexecuted contract
fixtures, an invalid admission-overclaim fixture, reviewed synthetic provenance,
and a source-free checker are present. The contract fixture deliberately keeps
every measured field false. Native execution cannot be inferred from schema or
fixture validation.

## Hosted Attempt

PR 183 run `33365999004`, job `99406666156`, evaluated source `d01344c`
on the `windows-2025` GitHub-hosted image: Windows Server 2025 build `26100`,
x86-64, NTFS. The no-worker capability preflight passed. The broker then
created and hardened fresh profiles, built the exact LPAC attributes, Job
Object, handles, mitigations, child policy, and filtered non-caller environment,
but `CreateProcessW` denied the first suspended worker with Win32 error `5`
before worker code ran.

The live receipt records `status=unsupported`,
`reason_code=unsupported_host`, and keeps every measured and authority field
false. The job passed because unsupported is an explicit fail-closed contract
state, not because confinement was demonstrated. Exact staging and both
profiles were removed before that receipt was emitted; uncertain cleanup would
have failed the job.

The observed boundary matches Microsoft's documented AppContainer host
preparation limitation: least-privileged AppContainers may require a
host-administered, non-inheriting system-drive-root access grant before common
process images can start. This rehearsal did not alter the drive-root ACL,
invoke an administrator helper, install a service, or weaken LPAC to ordinary
AppContainer. A later decision must evaluate reversible host preparation,
packaged activation, a newer BaseContainer host, or VM isolation as distinct
authority and compatibility choices.

## Claim Boundary

This hosted attempt does not establish any worker denial or resource result.
Even a later complete native matrix must keep `os_confined=false`,
`production_admitted=false`, and `analyzer_execution=false`. Independent-host
repeatability, compatibility withdrawal, signing, packaging, lifecycle, and a
later explicit admission decision remain required before Windows IAR-1B or a
real analyzer can be claimed.
