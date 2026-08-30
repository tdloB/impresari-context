# ADR-0088: Evaluate LPAC/AppContainer Plus Job Objects For Windows IAR-1B

- Status: Accepted for synthetic feasibility
- Date: 2026-08-30
- Decider: Aaron Boldt

## Context

The release matrix includes Windows, and the evidence foundation recognizes
Windows-oriented artifacts, but no Windows OS-confinement backend exists. A
Linux or macOS result cannot establish Windows process, registry, credential,
device, network, or cleanup behavior.

## Decision

Use a separately signed Windows broker combining a least-capability
AppContainer/LPAC identity with Job Object resource and descendant supervision.
Admit only exact Windows versions and architectures that independently pass the
complete original-synthetic corpus. Do not use administrator services or a VM
in the first candidate.

## Consequences

- Windows gains a native, local, no-upload feasibility path.
- AppContainer profile lifecycle, ACL staging, Job Object semantics, mitigations,
  signing, and Windows-version drift become maintained platform work.
- No real analyzer may run until this backend passes independently.

## Alternatives

- Reuse Linux evidence: rejected because Windows has different kernel and
  authority surfaces.
- Windows Sandbox or Hyper-V first: deferred because it adds a VM image and
  optional-feature/edition boundary before native feasibility is measured.
- Restricted token alone: rejected because it does not supply the complete
  capability and resource boundary.

## Revisit Triggers

Review if native confinement cannot deny a Tier A boundary, requires
administrator installation, or cannot provide clean cross-job identity.
