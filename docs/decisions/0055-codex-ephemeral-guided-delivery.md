# ADR-0055: Codex Ephemeral Guided Delivery

- Status: Accepted; recorded-scope L3 admitted
- Date: 2026-08-26
- Deciders: Impresari Context maintainers
- Related: [CI-3b PRD](../product/ci-3b-codex-guided-delivery-prd.md),
  [CI-3b ARD](../architecture/ci-3b-codex-guided-delivery-ard.md),
  [ADR-0042](0042-planner-backed-guided-context-delivery.md), and
  [ADR-0046](0046-explicit-guided-delivery-intent-contract.md)

## Decision

Admit a narrow experimental Codex App Server delivery adapter only for the
generated-schema scope of Codex CLI `0.150.0-alpha.8` on macOS aarch64. The
adapter consumes a separately reviewed CI-3a preview, re-verifies its exact
canonical packet bytes, and starts a direct, ephemeral, no-network,
read-only App Server thread only under an explicit `--apply` action.

All approval and authority requests are denied and the child is terminated.
The adapter retains no model output, packet, runtime, or client configuration
after the command. It has no fallback hook or automatic retry.

The client must complete `initialize` plus `initialized`, and `account/read`
must confirm usable provider authentication before thread creation. The
adapter does not project credentials from the user's normal Codex home into
the isolated runtime. ADR-0059 governs the explicit authenticated-home
boundary used for recorded-scope L3 admission.

## Consequences

- Preview/apply separation preserves audit integrity: apply does not rerun a
  previously recorded intent or reopen the source workspace.
- A compatibility upgrade requires an exact version/schema/lifecycle review
  rather than accepting a broad Codex version range.
- Safe `no_delivery` and `degraded` outcomes never support admission on their
  own; L3 rests on the two successful records in the CI-3b verification file.
- The small packet ceiling is an intentional protocol safety boundary, not a
  general planner budget change.
