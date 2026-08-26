# Impresari Context — CI-3b: Codex Guided-Delivery PRD

- Status: Implemented; bounded live lifecycle record is degraded, not admitted
- Date: 2026-08-26
- Authority: Founder-approved client-integration roadmap and autonomous delivery directive
- Governing roadmap: [Client Integration Depth Roadmap](client-integration-roadmap.md)
- Dependency: [CI-3a delivery-intent contract](client-integration-l3-delivery-intent-prd.md)
- Architecture requirements: [CI-3b Codex delivery ARD](../architecture/ci-3b-codex-guided-delivery-ard.md)

## Objective

Provide a narrow, explicit Codex App Server delivery path for one already
previewed deterministic context packet. The client process must receive no
workspace root, cache path, credentials, configuration mutation, tool grant,
network sandbox permission, or source-writing authority.

## Scope

- Admit only Codex App Server `0.149.0-alpha.4.1` on macOS aarch64 through
  `initialize`, an ephemeral read-only `thread/start`, read-only/no-network
  `turn/start`, and `turn/completed`.
- Require CI-3a's exact `codex` / `app_server_ephemeral` / `turn_start`
  identity, explicit consent, verified workspace/snapshot binding, planner
  result, canonical packet identity, and hard budget.
- Split preview and apply. Preview performs planner work only; apply accepts
  the exported preview artifact, re-derives canonical bytes, verifies every
  packet/plan/snapshot/receipt/envelope binding, and still requires `--apply`.
- Start a direct child process in one guarded temporary directory, with a
  cleared environment, temporary `CODEX_HOME`, temporary current directory,
  ephemeral thread, read-only sandbox, and disabled tool-network access.
- Deny every App Server authority request, return a visible bounded receipt,
  terminate the child, and delete the exact temporary runtime directory.

## Non-goals

- Persistent Codex configuration, automatic handoff, hooks, background
  processes, packet retry/retention, model-output capture, source mutation,
  repository code execution, or an L3 promotion.

## Acceptance criteria

- Preview and `apply` without `--apply` never start Codex and never mutate the
  workspace.
- Apply sends only a byte-verified packet no larger than 512 KiB, keeping the
  complete JSON-RPC line within the one-mebibyte protocol limit.
- A mismatched, altered, stale, malformed, or unsupported preview fails before
  client I/O. A version mismatch or client failure returns `no_delivery`; an
  authority request or timeout returns `degraded` with a stable reason.
- Tests prove exact-byte envelope encoding, serialized-preview rehydration,
  alteration rejection, no-client preview, no-authority delivery receipt, and
  fail-closed authority handling.
- A live record includes the exact binary/client version, platform, protocol
  scope, packet identity, receipt outcome, temporary-runtime cleanup, and
  source hash. A degraded record is evidence of safe behavior, not delivery.

## Reassessment checkpoint

CI-3b does not alter L1/L2 classification. Promote Codex to L3 only after a
repeatable successful lifecycle record under this exact version/OS scope and a
fresh security/architecture review. Otherwise retain the adapter as an
experimental, explicit opt-in capability or remove it.
