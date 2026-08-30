# Linux IAR-1B Production Lifecycle ARD

- Status: Accepted
- Date: 2026-08-30
- Decision: ADR-0079

## Architecture

The released policy is a closed source-free contract:

```text
frozen A+C lifecycle policy + explicit bounded observations
                              |
                              v
                  exact shape and order validation
                              |
                              v
         profile-specific phase and cleanup evaluation
                              |
                              v
        authority-denying lifecycle-candidate receipt
```

The evaluator does not collect observations. Hosted package rehearsals must
produce them through separately reviewed workflows. This keeps health and
claim evaluation free of host discovery, process execution, privilege,
service mutation, repair, and background authority.

## Canonical Profile Sequences

Rootless user manager:

1. clean install;
2. upgrade;
3. rollback;
4. logout/login reentry;
5. cancellation;
6. crash recovery;
7. health withdrawal after a changed prerequisite; and
8. uninstall.

Externally managed:

1. clean install;
2. upgrade;
3. rollback;
4. operator relaunch;
5. cancellation;
6. crash recovery;
7. health withdrawal after a changed prerequisite; and
8. uninstall.

Sequence, phase count, and operation-evidence vocabulary are exact. The
externally managed profile cannot borrow the rootless login result, and the
rootless profile cannot accept an operator-provided capability as hidden
authority.

## Deterministic Evaluation

1. Reject malformed or authority-expanding policy and observation shapes.
2. Require the exact selected-profile phase order and operation evidence.
3. Treat a nonzero `not_observed` phase as an invalid contract.
4. Return `lifecycle_failed` when a normal phase fails identity,
   revalidation, or clean-state evidence.
5. Return `withdrawal_failed` when changed prerequisites do not withdraw the
   claim and leave a clean state.
6. Return `incomplete` when one or more canonical phases remain unobserved.
7. Return `lifecycle_candidate` only when every phase passes.

The lifecycle candidate is never a production or packaging admission. A later
hosted rehearsal must bind real release artifacts and independently reproduce
each profile's matrix.

## Clean-State Invariant

Every passing phase leaves no persistent service, privileged policy, stale
cgroup, worker descendant, or staged source. The health-withdrawal phase is
the one intentional topology mismatch: it must set
`topology_revalidated=false`, `claim_withdrawn=true`, and still prove clean
state. Automatic repair and application-only fallback are prohibited.

## Resource Profile

The evaluator is a short foreground Ruby process over two bounded JSON files.
It spawns no children and writes no persistent state. The existing synthetic
worker resource profile remains separate and is not executed by this contract.
