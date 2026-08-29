# ADR-0043: Source-free client lifecycle maintenance

- Status: Accepted; GitHub Copilot CLI, Claude Code, Cursor, and VS Code Copilot scopes implemented
- Date: 2026-08-25
- Scope: CI-4 health, freshness, and compatibility signals

## Decision

Use explicit, source-free, read-only health checks over versioned compatibility
manifests for CI-4. A health check can describe an owned integration’s
compatibility and safe degradation path; it cannot monitor, repair, install,
approve, or otherwise control a third-party client.

## Rationale

Deep integration becomes unsafe when product maintenance turns into background
client surveillance or silent configuration changes. An on-demand contract
check provides useful freshness evidence while preserving user authority.

## Constraints

- Checks run only after a user invokes them against a named owned artifact.
- They use no repository source, account data, secrets, background process,
  network proxy, shell hook, or client mutation.
- Unknown, stale, malformed, unsupported, or missing evidence fails closed to
  a visible degraded state.
- The manifest must bind client, scope, version/OS range, released transport
  shape, lifecycle capability, evidence date, and retirement/remediation path.
- CI-4 is unavailable where a client does not have a stable official lifecycle
  surface that can preserve these boundaries.

## Consequences

Impresari can offer maintenance depth comparable to adjacent products without
claiming surveillance, automated healing, or opaque connection management.
The compatibility matrix remains the public truth and must demote stale claims.
Independent manifests now admit the exact Copilot CLI v3 instruction, Claude
Code v2 skill, Cursor v2 rule, and VS Code Copilot v3-guidance/L3-delivery
scopes without adding client discovery or repair.

## References

- [CI-4 lifecycle maintenance PRD](../product/client-integration-l4-lifecycle-maintenance-prd.md)
- [Client Integration Depth Roadmap](../product/client-integration-roadmap.md)
- [ADR-0029: Progressive client integration depth and consent](0029-progressive-client-integration-depth-and-consent.md)
