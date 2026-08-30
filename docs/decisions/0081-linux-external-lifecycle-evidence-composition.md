# ADR-0081: Compose Exact C Lifecycle Evidence After Capability Withdrawal

- Status: Accepted
- Date: 2026-08-30
- Decider: Aaron Boldt

## Context

ADR-0080 produced an exact package-lifecycle candidate for the selected C
externally managed profile. The independent external rehearsal already proves
fresh inherited-capability revalidation, synthetic confinement, exact cgroup
termination, timeout, crash/relaunch, descendant cleanup, and provisioner
collection. Those records must not be combined by prose or across unrelated
runs, and collection alone does not prove that a later missing capability
withdraws the claim.

## Decision

In one exact-source GitHub-hosted Linux release-candidate job, compose four
immutable receipts:

1. the exact C package-lifecycle receipt;
2. a fresh externally managed live-rehearsal receipt;
3. its identity-linked original-synthetic composite receipt; and
4. a post-collection source-free health receipt proving that fixed descriptor
   3 is unavailable, topology does not revalidate, and the claim is withdrawn.

The operator creates and collects exactly one temporary delegated systemd
service under the already accepted external-C CI authority. After collection,
the health collector receives no capability, path, unit name, privilege,
service-manager access, or repair authority. The composer verifies exact source,
receipt, package, host, interruption, crash, cleanup, and withdrawal identities.

## Consequences

- C may advance from a package candidate to an exact-host lifecycle candidate.
- A remains separate and partial until a genuine logout/login boundary is
  observed.
- A failed identity, package, external, interruption, or withdrawal input keeps
  the lifecycle candidate inactive.
- A later C invocation requires a newly supplied and revalidated external
  capability; no prior capability survives collection.
- Production support, release publication, real analyzers, privileged
  installation, persistent services, automatic sudo, and automatic repair stay
  closed.

## Alternatives

- Combine independent historical run identifiers in documentation: rejected
  because temporal and identity linkage would be inferential.
- Treat service collection as health withdrawal without a subsequent probe:
  rejected because the claim-withdrawal behavior would be unobserved.
- Keep or recreate a privileged service for health monitoring: rejected because
  it violates the selected A+C authority boundary.

## Acceptance Effect

This decision authorizes the source-free health collector, closed composition
schema, deterministic evaluator tests, and one exact hosted composition inside
the existing release-candidate job. It does not authorize repository content as
analyzer input, a real analyzer, production admission, publication, networking
inside the collectors, credentials, a persistent service, or administrator-
provisioned installation.
