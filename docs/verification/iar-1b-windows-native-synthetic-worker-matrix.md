# IAR-1B Windows Native Synthetic Worker Matrix

- Status: Contract frozen; native worker evidence pending
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

## Claim Boundary

Even a complete first native matrix must keep `os_confined=false`,
`production_admitted=false`, and `analyzer_execution=false`. Independent-host
repeatability, compatibility withdrawal, signing, packaging, lifecycle, and a
later explicit admission decision remain required before Windows IAR-1B or a
real analyzer can be claimed.
