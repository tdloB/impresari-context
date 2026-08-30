# Linux IAR-1B Production Lifecycle PRD

- Status: Accepted for implementation
- Date: 2026-08-30
- Owner: Aaron Boldt
- Decision: ADR-0079
- Parent: ADR-0078 Linux production delegation

## Problem

The rootless user-manager and externally managed profiles now have independent
exact-host synthetic confinement candidates. Those passes do not establish that
an installed package remains safe through replacement, rollback, session
reentry, interruption, prerequisite drift, or removal. A production claim made
without those lifecycle gates could leave stale services, policy, cgroups,
descendants, or staged source behind.

## Outcome

Freeze one source-free lifecycle policy and deterministic evaluator shared by
both selected profiles. The evaluator accepts only an explicit bounded
observation set and returns `lifecycle_candidate`, `incomplete`,
`lifecycle_failed`, `withdrawal_failed`, or `invalid_contract`.

`lifecycle_candidate` is a contract-level state only. It does not admit a
package, release, production platform, or analyzer.

## Profile Matrices

Both profiles must independently prove clean install, upgrade, rollback,
cancellation, crash recovery, changed-prerequisite withdrawal, and uninstall.
The rootless profile additionally proves logout/login reentry through the
existing user manager. The externally managed profile instead proves an
operator-controlled relaunch and receives no authority to create or repair its
parent delegation.

Every passing phase requires:

- exact package-artifact identity;
- fresh topology revalidation, except that the withdrawal phase must observe
  failed revalidation and withdraw the claim;
- no persistent Impresari service or privileged authorization policy;
- no stale delegated cgroup or descendants; and
- no staged source bytes.

## Package Boundary

The initial Linux package contains only the three existing release binaries:
CLI, local MCP server, and first-party structural worker. It installs no service
unit, authorization policy, privileged helper, updater, or background monitor.
The external operator contract is documentation, not a package-installed
control plane.

## Acceptance Gates

- Closed JSON Schema for policy, observations, and receipts.
- Exact policy identity and source-free observation identity in every receipt.
- Canonical phase order and profile-specific reentry semantics.
- Deterministic candidate, incomplete, lifecycle-failure,
  withdrawal-failure, and invalid-contract states.
- Changed prerequisites withdraw rather than repair or downgrade the claim.
- Production, release packaging, real analyzers, privileged installation, and
  persistent services remain false in every state.
- Original-synthetic fixture provenance and full repository checks pass.

## Explicit Non-Goals

This increment performs no installation, upgrade, rollback, login-session
mutation, service management, analyzer execution, repository-source staging,
network access, credential access, publication, or production admission. Live
package rehearsals and production release evidence are later gates.

## Package-Rehearsal Increment

ADR-0080 implements the next package-only gate against the public v0.1.0 Linux
archive and an exact-source release candidate. Both profiles must prove install,
replacement, rollback, and removal in a disposable prefix. The external profile
also proves operator relaunch. The rootless profile must remain partial until a
genuine logout/login session boundary is observed; a process restart is not an
acceptable substitute. Cancellation, crash, health withdrawal, topology
revalidation, production, and analyzers remain separate gates.

## External Composition Increment

ADR-0081 composes the exact C package receipt with a fresh external live
receipt, its identity-linked original-synthetic interruption/crash receipt, and
a post-collection missing-capability health receipt. Only a same-run,
exact-source, fully linked set can become a C lifecycle candidate. This does not
complete A, admit production, or authorize an analyzer.
