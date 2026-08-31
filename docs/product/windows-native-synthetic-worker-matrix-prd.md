# Impresari Context — Windows Native Synthetic Worker Matrix PRD

- Status: Accepted for implementation under ADR-0088
- Date: 2026-08-31
- Owner: Aaron Boldt
- Governing decision: [ADR-0088](../decisions/0088-windows-native-analyzer-confinement.md)

## Objective

Advance the Windows native confinement candidate beyond API preflight by
launching one exact first-party synthetic worker suspended inside the frozen
zero-capability LPAC/AppContainer and pre-limited Job Object boundary.

## Scope

- A checkpoint-specific closed profile and receipt contract bound to the exact
  ADR-0092 base profile.
- A reviewed Windows-only broker and first-party synthetic boundary worker.
- Exact ACL staging, hardened AppContainer profile storage, bounded inherited
  pipes, creation-time mitigations, child-process restriction, and Job Object
  assignment before resume.
- Positive exact-input processing plus path, network, registry, handle,
  process, resource, fault, cleanup, and cross-job synthetic observations.
- One fresh GitHub-hosted Windows 2025 x86-64 VM per native evidence run.

## Non-goals

Repository input, repository executables, real analyzers, YARA, ClamAV,
external network destinations, existing credential inspection, arbitrary
commands/arguments/environment, administrator services, installers, signing,
production support, Windows arm64, broad Windows compatibility, and an IAR-1B
admission claim.

## Acceptance Criteria

- The worker cannot run before the complete AppContainer, handle, mitigation,
  child-policy, ACL, and Job Object preparation succeeds.
- The worker observes the exact fresh AppContainer SID, zero capabilities, and
  LPAC posture.
- Exact current-job input is readable but immutable; worker, sibling,
  profile-storage, user-profile canary, and synthetic registry boundaries are
  denied as specified.
- A broker-owned loopback connection attempt is denied without contacting any
  external destination.
- Only the fixed control/result/error handles cross the launch boundary, child
  and breakaway attempts fail, and active-process peak remains one.
- The frozen CPU and memory controls are queried exactly; the effective
  single-process memory ceiling plus CPU, timeout, output, crash, cancellation,
  and malformed-result faults fail closed.
- All descendants stop and exact staging, canaries, ACL state, handles, and
  AppContainer profile are removed before a second unique identity proves
  cross-job cleanliness.
- Any unavailable control, successful denied action, or ambiguous cleanup
  returns unsupported/failed and keeps every admission claim false.

## Evidence Boundary

A complete result is exact-host synthetic feasibility only. It may record that
a synthetic LPAC worker ran and individual controls passed, but
`os_confined`, `production_admitted`, and `analyzer_execution` remain false.
