# Linux External Lifecycle Composition PRD

- Status: Accepted for implementation
- Date: 2026-08-30
- Owner: Aaron Boldt
- Decision: ADR-0081

## Problem

The C package receipt and external synthetic confinement receipt prove useful
but independent facts. Without exact same-run composition, package identity can
drift from the tested topology, interruption checks can be omitted, or a
collected operator service can be misrepresented as successful fail-closed
health withdrawal.

## Outcome

Produce one closed `linux-external-lifecycle-composition` receipt that becomes
`lifecycle_candidate` only when exact package, topology, composite interruption,
crash, cleanup, and post-collection withdrawal records link by SHA-256 identity
inside one exact-source hosted run.

## Requirements

- Bind the candidate source commit, archive, manifest, three-binary package, and
  package receipt identity.
- Bind the external live receipt to its exact composite receipt and observed
  Linux host.
- Require exact cgroup kill, empty-state verification, timeout, crash/relaunch,
  and cleanup.
- After the one transient operator service is collected, invoke the health
  collector with descriptor 3 closed.
- Require capability unavailable, topology not revalidated, claim withdrawn,
  and all clean-state fields true.
- Preserve canonical C phase order from install through uninstall.
- Return deterministic failure states for identity, package, external,
  interruption, and withdrawal failures.
- Keep every production, analyzer, packaging, privileged-installation, and
  persistent-service authority field false.

## Explicit Non-Goals

No repository content is staged or analyzed. The increment installs no service,
uses no background monitor, repairs no topology, receives no credential, grants
no Impresari privilege, publishes no package, and does not change the A profile.

## Acceptance Gates

- Closed schema with valid withdrawal/composition fixtures and an overclaim
  rejection.
- One deterministic candidate plus all five fail-closed states.
- Exactly one existing external-C transient `sudo systemd-run` launch site.
- Exact-source release-candidate workflow execution and artifact retention.
- Full repository gate and all hosted PR checks pass.
