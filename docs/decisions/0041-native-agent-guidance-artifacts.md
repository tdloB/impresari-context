# ADR-0041: Native agent guidance artifacts

- Status: Accepted for implementation
- Date: 2026-08-24
- Scope: CI-2 native guidance for Codex, Claude Code, Cursor, and GitHub Copilot

## Decision

Implement CI-2 as versioned, original, project-scoped guidance artifacts over
a shared evidence-use contract with thin client-specific renderers and
validators. Each artifact is previewable, ownership-marked, opt-in, and exactly
removable. It may guide a user or conversational agent to request a bounded
Impresari packet; it cannot deliver a packet automatically.

## Rationale

Competitors demonstrate that native guidance makes a context product more
discoverable than an MCP entry alone. Treating guidance as an owned, auditable
artifact preserves that benefit without accepting hidden prompt injection,
model governance, or opaque client behavior.

## Constraints

- Client-specific artifacts retain the exact shared language on evidence,
  supported profiles, packet identity, reasons, omissions, and unavailable
  server behavior.
- No artifact may expand client authority, inject secrets, change trust or
  approval state, write source content, invoke a shell, or enable networking.
- Unknown client/version/scope behavior fails closed and remains at L0/L1.
- A model-directed tool call is smoke evidence only; the artifact contract,
  renderer, validator, and removal lifecycle must be deterministic.
- Dynamic request constraints, including identifier grammar and the fixed
  resource-policy fingerprint, remain authoritative MCP tool-schema values.
  They are not copied into an exact-owned guidance artifact, so a protocol
  correction does not strand an already installed artifact from safe removal.
- CI-1 L1 classification and CI-2 L2 classification are independent: no L2
  artifact can conceal missing managed-connection evidence.

## Consequences

The project can provide a maintainable native experience without binding core
evidence logic to any provider SDK or proprietary prompt format. L3 delivery
will later consume only deterministic-planner packets and must have a separate
consent and equivalence admission.

## References

- [CI-2 native guidance PRD](../product/client-integration-l2-native-guidance-prd.md)
- [Client Integration Depth Roadmap](../product/client-integration-roadmap.md)
- [ADR-0029: Progressive client integration depth and consent](0029-progressive-client-integration-depth-and-consent.md)
- [ADR-0035: L1 managed client connection kits](0035-l1-managed-client-connection-kits.md)
