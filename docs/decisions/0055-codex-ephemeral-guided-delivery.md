# ADR-0055: Codex Ephemeral Guided Delivery

- Status: Accepted for implementation; L3 admission remains pending
- Date: 2026-08-26
- Deciders: Impresari Context maintainers
- Related: [CI-3b PRD](../product/ci-3b-codex-guided-delivery-prd.md),
  [CI-3b ARD](../architecture/ci-3b-codex-guided-delivery-ard.md),
  [ADR-0042](0042-planner-backed-guided-context-delivery.md), and
  [ADR-0046](0046-explicit-guided-delivery-intent-contract.md)

## Decision

Admit a narrow experimental Codex App Server delivery adapter only for the
generated-schema scope of Codex CLI `0.149.0-alpha.4.1` on macOS aarch64. The
adapter consumes a separately reviewed CI-3a preview, re-verifies its exact
canonical packet bytes, and starts a direct, ephemeral, no-network,
read-only App Server thread only under an explicit `--apply` action.

All approval and authority requests are denied and the child is terminated.
The adapter retains no model output, packet, runtime, or client configuration
after the command. It has no fallback hook or automatic retry.

## Consequences

- Preview/apply separation preserves audit integrity: apply does not rerun a
  previously recorded intent or reopen the source workspace.
- A compatibility upgrade requires an exact version/schema/lifecycle review
  rather than accepting a broad Codex version range.
- The adapter can document safe `no_delivery` and `degraded` outcomes before a
  successful lifecycle completion exists. It must not claim Codex L3 admission
  on that evidence alone.
- The small packet ceiling is an intentional protocol safety boundary, not a
  general planner budget change.
