# ADR-0035: L1 managed client connection kits

- Status: Accepted for implementation
- Date: 2026-08-24
- Scope: Codex, Claude Code, Cursor, and GitHub Copilot L1 connections

## Decision

Build a shared, manifest-driven L1 managed-connection layer with thin
client-specific serializers and validators. It provides preview/render,
validate, explicit install, inspect, and exact owned-entry removal operations.
The layer permits a configuration write only when the caller invokes an
explicit install/remove operation against a named target after seeing the
preview; ordinary diagnostics remain read-only.

Every installed entry is versioned and ownership-marked. The layer accepts
only the fixed local stdio process contract: absolute Impresari binary,
explicit authorized workspace, separate cache, fixed consumer ID/role, and no
environment forwarding, remote URL, automatic approval, or client-specific
authority expansion.

## Constraints

- Refuse symlinks, ambiguous scope, oversized/malformed config, duplicate or
  conflicting Impresari entries, and removal of any unowned entry.
- Preserve unrelated client configuration; do not reset, reformat broadly, or
  change project trust, sign-in, approval, shell, editor, or source state.
- Client-level L1 promotion requires client/version/OS and disposable
  real-client evidence; model-directed tool selection is not deterministic
  conformance.
- If an upstream client cannot preserve these controls, degrade to L0.

## Consequences

The project can match competitors’ managed setup depth with more auditable,
reversible behavior. L2–L4 remain opt-in, separately governed work.

## References

- [CI-1 managed connections PRD](../product/client-integration-l1-managed-connections-prd.md)
- [Client Integration Depth Roadmap](../product/client-integration-roadmap.md)
- [ADR-0029: Progressive client integration depth and consent](0029-progressive-client-integration-depth-and-consent.md)
