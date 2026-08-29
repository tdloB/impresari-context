# Impresari Context — CI-4: Deep Lifecycle Maintenance PRD

- Status: GitHub Copilot CLI and Claude Code scopes implemented
- Date: 2026-08-25
- Authority: Founder-approved client-integration roadmap and autonomous delivery directive
- Governing roadmap: [Client Integration Depth Roadmap](client-integration-roadmap.md)
- Dependency: a client-specific L1/L2/L3 capability with a stable, documented lifecycle surface.
- Architecture requirements: [CI-4 lifecycle maintenance ARD](../architecture/ci-4-client-lifecycle-maintenance-ard.md)

## Objective

For a client whose official lifecycle surface is stable and already admitted,
provide source-free health, compatibility, and freshness signals that help the
user understand whether an owned Impresari integration remains valid. CI-4 must
observe, not control, the client.

## Scope

- Explicit, on-demand health checks for an owned connection/guidance/delivery
  artifact: client/version/OS scope, fixed transport contract, expected tool
  inventory, enabled state where available, and last verified compatible
  evidence record.
- A signed/identified compatibility manifest with stable status codes:
  compatible, degraded, unknown, stale-evidence, or unsupported.
- Visible remediation guidance and exact owned-artifact disable/removal; no
  repair action occurs automatically.
- Client-specific regression fixtures that detect an upstream format, tool, or
  lifecycle change before any supported classification is retained.

## Non-goals

- Background polling, scheduled monitoring, telemetry, client analytics,
  client-account reads, automated upgrade/downgrade, silent config repair,
  shell hooks, editor process injection, source scanning, network proxying, or
  delivery of context packets.
- A health signal cannot broaden a client’s L1/L2/L3 authority or bypass its
  consent boundary.

## Required flow

1. A user explicitly invokes a health check against a named owned target.
2. Impresari verifies only the released local contract and published
   version/OS/evidence metadata; it never reads repository source to report
   health.
3. The result includes the observed scope, compatible range, evidence date,
   checks performed, degradation reason, and safe next step.
4. The user may explicitly disable or remove the exact owned target. If the
   contract is unknown or stale, the integration degrades to manual MCP use.

## Acceptance criteria

- Diagnostics are source-free, bounded, deterministic, and read-only.
- No background process is started or retained. A check must not change client
  configuration, trust, sign-in, tool approval, source files, cache authority,
  shell settings, or networking permissions.
- Version/OS/lifecycle matching fails closed. A stale or unverified upstream
  client version is reported as degraded, never silently accepted.
- Tests cover healthy, stale, unsupported, malformed, missing-owned-entry, and
  client-unavailable states; they prove source immutability and exact-removal
  behavior.
- Each L4 assertion carries a client/scope/version/OS record and is withdrawn
  when its upstream surface no longer meets the recorded contract.

## Reassessment checkpoint

Reassess the compatibility manifest after every supported client release,
observed regression, or security boundary change. This is an on-demand
maintenance process, not an automated monitoring service.

## Admitted scopes

CI-4 covers the exact GitHub Copilot CLI v3 native-guidance artifact recorded
for CLI `1.0.80` and the exact Claude Code v2 project skill recorded for CLI
`2.1.241`, both on macOS aarch64. Each has an independent manifest and
regression gate over the shared source-free receipt contract. Client
availability, version, OS, architecture, date, manifest, and owned target are
caller-supplied; the checker performs no discovery and starts no client
process. All other clients and surfaces remain outside CI-4 until independently
admitted.
