# CI-4 Client Lifecycle Maintenance — Architecture Requirements and Design

- Status: GitHub Copilot CLI and Claude Code scopes implemented
- Date: 2026-08-25
- Governing product record: [CI-4 PRD](../product/client-integration-l4-lifecycle-maintenance-prd.md)
- Governing decision: [ADR-0043](../decisions/0043-source-free-client-lifecycle-maintenance.md)

## Architectural objective

Provide a one-shot, source-free compatibility assessment for a caller-named,
owned client artifact. The maintenance boundary reports observable contract
state; it never watches, changes, repairs, or controls a third-party client.

## Required data flow

```text
explicit health invocation + named owned target
                    |
                    v
source-free target/ownership validator
                    |
                    v
versioned compatibility manifest ----> evidence-record verifier
                    |                          |
                    +------------+-------------+
                                 v
                        deterministic status receipt
```

The verifier reads only released manifest metadata and the named owned artifact
needed to establish its local fixed contract. It never reads workspace source,
client accounts, credentials, shell state, process output, or network data.

## Status contract

The result is exactly one of:

- `compatible`: owned artifact and recorded client/scope/version/OS contract
  match a current evidence record;
- `degraded`: a known contract mismatch, disabled owned artifact, or expired
  evidence record has a safe manual-MCP/removal path;
- `stale_evidence`: the artifact is valid but its compatibility evidence is
  outside its declared freshness window;
- `unknown`: insufficient local evidence to assess a client/version/scope;
- `unsupported`: no released CI-4 adapter exists for the client surface.

No status is an instruction to change a client. The receipt includes the
observed scope, manifest/evidence identity, checked invariants, and one
source-free safe next step.

## Invariants

1. A health check is invoked by a user against a named owned target; it never
   discovers default targets or runs in the background.
2. Unknown, stale, malformed, missing, unowned, or unsupported states fail
   closed. They cannot retain or create a higher integration classification.
3. The checker has no source read, cache write, process, shell, environment,
   credential, client-account, network, trust, or approval authority.
4. A compatibility manifest binds client, surface/scope, supported version/OS,
   artifact identity, lifecycle capability, evidence identity/date, and
   degradation/removal guidance.
5. A client release that falls outside the manifest range is immediately
   reported as `unknown` or `stale_evidence`, never implicitly accepted.

## Verification requirements

- Deterministic fixtures cover each status, malformed manifest/artifact,
  unowned target, missing target, stale evidence, unsupported version/OS, and
  exact owned-artifact removal.
- Tests prove no source workspace reads or writes and no network/process/shell
  activity.
- Published compatibility claims must be generated from the same manifest and
  evidence records used by the checker.

## Initial implementation order

1. Define the source-free compatibility manifest and receipt schema.
2. Add a read-only checker for an already-owned CI-1/CI-2 artifact.
3. Admit one client/scope only after its L1/L2 evidence record proves a stable
   official lifecycle surface.
4. Defer all other clients rather than simulating health from conversational
   behavior or unverified version detection.

## Implemented boundary

`scripts/client-lifecycle-health.rb` accepts an explicit manifest, absolute
owned-target path, client availability, exact version, OS, architecture, and
assessment date. It reads only the supplied manifest and supplied target. It
does not inspect a workspace, discover a client, read environment variables,
spawn a client or shell, access a network, mutate an artifact, or retain a
background process. The admitted manifests are
`client-lifecycle/copilot-cli-native-guidance-v1.json` and
`client-lifecycle/claude-code-native-guidance-v1.json`; independent repository
gates bind each exact artifact and evidence record by SHA-256 while sharing the
same closed health-receipt contract.
