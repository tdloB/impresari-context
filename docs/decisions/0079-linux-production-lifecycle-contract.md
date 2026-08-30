# ADR-0079: Freeze A Shared Linux Production-Lifecycle Contract

- Status: Accepted
- Date: 2026-08-30
- Decider: Aaron Boldt

## Context

ADR-0078 selected rootless user-manager delegation plus an explicit externally
managed profile. Both profiles now have independent synthetic confinement
evidence, but neither has a frozen install-through-uninstall lifecycle. The
production acceptance gate requires clean install, upgrade, rollback,
session/relaunch, cancellation, crash, prerequisite withdrawal, and uninstall
evidence without stale services, policy, cgroups, descendants, or source.

## Decision

Use one closed source-free lifecycle policy and evaluator for both selected
profiles. Share all common package and cleanup rules, while retaining one
profile-specific reentry phase: logout/login for rootless and operator relaunch
for external delegation.

The package boundary is CLI plus first-party worker only. It installs no
service unit, authorization policy, privileged helper, or background repair
component. Every changed prerequisite withdraws the claim; no automatic sudo,
repair, IAR-1A downgrade, or persistent service is permitted.

## Rationale

A shared contract prevents the two profiles from drifting into different
security or release semantics. A profile-specific reentry phase preserves the
real operational difference without duplicating every gate. A pure evaluator
makes failed and incomplete evidence explicit before live package workflows
receive any mutation authority.

## Consequences

- Both selected profiles must pass independently before either gains a
  production-support claim.
- Rootless login evidence cannot admit the external profile, and operator
  relaunch evidence cannot admit rootless desktop behavior.
- Installation remains unprivileged and foreground-oriented.
- The next checkpoint is hosted package-lifecycle rehearsal against exact
  release-candidate artifacts; production and IAR-2 remain closed.

## Alternatives

- Separate lifecycle policies per profile: rejected because common package and
  cleanup semantics could diverge silently.
- Admit after synthetic confinement alone: rejected because it omits release
  replacement, recovery, drift, and removal behavior.
- Add automatic repair or an administrator service: rejected by ADR-0078 and
  outside the accepted A+C authority boundary.

## Acceptance Effect

This decision authorizes schemas, a frozen policy, a source-free evaluator,
synthetic fixtures, and documentation. It does not authorize host mutation,
package publication, service installation, privilege, repository-derived
analyzer input, real analyzers, production admission, or IAR-2.
