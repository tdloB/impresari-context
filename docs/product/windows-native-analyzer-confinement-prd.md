# Impresari Context — Windows Native Analyzer Confinement PRD

- Status: Accepted for synthetic feasibility
- Date: 2026-08-30
- Owner: Aaron Boldt
- Decision: ADR-0088

## Objective

Provide an independently admitted Windows IAR-1B backend using documented
Windows isolation primitives before any real analyzer runs on Windows.

## Scope

- Less-Privileged AppContainer or the strongest supported AppContainer profile
  with no network, device, registry, COM, credential, or UI capabilities.
- Job Object supervision with kill-on-close, active-process, CPU, memory, time,
  and descendant accounting limits.
- A unique per-job identity and access-controlled staging directory containing
  only exact read-only inputs, plus bounded inherited result pipes.
- Fixed process mitigation policies appropriate to a headless analyzer.
- Synthetic no-op/fault workers and explicit Windows version/architecture
  compatibility receipts.

## Non-goals

- Windows Sandbox, Hyper-V VM, administrator service, kernel driver, real
  analyzer, executable repository artifact, installer execution, PowerShell
  execution, repository-/policy-directed or persistent registry mutation,
  network reputation, or automatic fallback. The documented transient
  per-user storage created and deleted with one AppContainer profile is part of
  the accepted profile-lifecycle feasibility boundary.

## Acceptance Criteria

- No access to user profile, credentials, browser state, registry, devices,
  unrelated files/processes, local network, or Internet.
- Child and breakaway attempts remain contained or fail closed.
- CPU, memory, process count, wall time, output, and zero writable path-backed
  storage are enforced for the complete job.
- Each job receives a fresh identity; profile and staged state are removed only
  after every descendant is dead, and a cross-job canary never survives.
- Unsupported Windows builds, unavailable mitigations, or cleanup uncertainty
  return `unsupported` or `failed`, never application-only fallback.
- Independent Windows hosts reproduce the complete synthetic matrix before any
  production or real-analyzer claim.
