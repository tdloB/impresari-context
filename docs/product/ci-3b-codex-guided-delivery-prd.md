# Impresari Context — CI-3b: Codex Guided-Delivery PRD

- Status: Complete; recorded-scope L3 admitted
- Date: 2026-08-28
- Authority: Founder-approved client-integration roadmap and autonomous delivery directive
- Governing roadmap: [Client Integration Depth Roadmap](client-integration-roadmap.md)
- Dependency: [CI-3a delivery-intent contract](client-integration-l3-delivery-intent-prd.md)
- Architecture requirements: [CI-3b Codex delivery ARD](../architecture/ci-3b-codex-guided-delivery-ard.md)

## Objective

Provide a narrow, explicit Codex App Server delivery path for one already
previewed deterministic context packet. Impresari must receive no credential
value and the client process must receive no workspace root, cache path, tool
grant, network sandbox permission, or source-writing authority. Codex alone
may manage authentication inside the explicit dedicated `CODEX_HOME`.

## Scope

- Admit only Codex App Server `0.150.0-alpha.8` on macOS aarch64 through
  `initialize`, the required `initialized` notification, an `account/read`
  preflight, an ephemeral read-only `thread/start`, read-only/no-network
  `turn/start`, and a successfully completed `turn/completed`.
- Require CI-3a's exact `codex` / `app_server_ephemeral` / `turn_start`
  identity, explicit consent, verified workspace/snapshot binding, planner
  result, canonical packet identity, and hard budget.
- Split preview and apply. Preview performs planner work only; apply accepts
  the exported preview artifact, re-derives canonical bytes, verifies every
  packet/plan/snapshot/receipt/envelope binding, and still requires `--apply`.
- Start a direct child process in one guarded temporary directory, with a
  cleared environment, explicit dedicated authenticated `CODEX_HOME`,
  temporary current directory,
  ephemeral thread, read-only sandbox, and disabled tool-network access.
- Deny every App Server authority request, return a visible bounded receipt,
  terminate the child, and delete the exact temporary runtime directory.

## Non-goals

- Discovery or copying of the normal Codex home, automatic handoff, hooks, background
  processes, packet retry/retention, model-output capture, source mutation,
  repository code execution, or admission outside the recorded exact scope.

## Acceptance criteria

- Preview and `apply` without `--apply` never start Codex and never mutate the
  workspace.
- Apply sends only a byte-verified packet no larger than 512 KiB, keeping the
  complete JSON-RPC line within the one-mebibyte protocol limit.
- A mismatched, altered, stale, malformed, or unsupported preview fails before
  client I/O. A version mismatch or client failure returns `no_delivery`; an
  authority request or timeout returns `degraded` with a stable reason.
- A supplied home without usable provider authentication returns
  `codex_auth_unavailable` before thread creation and packet delivery. The
  adapter never copies credentials from another Codex home. The supplied home
  must be canonical, non-symlinked, and separate from the disposable runtime.
- Tests prove exact-byte envelope encoding, serialized-preview rehydration,
  alteration rejection, no-client preview, no-authority delivery receipt, and
  fail-closed authority handling.
- A live record includes the exact binary/client version, platform, protocol
  scope, packet identity, receipt outcome, temporary-runtime cleanup, and
  source hash. A degraded record is evidence of safe behavior, not delivery.

## Reassessment checkpoint

The two successful authenticated-home lifecycle records satisfy the L3 gate
only for Codex App Server `0.150.0-alpha.8` on the recorded macOS arm64 scope.
Any client, version, protocol, platform, sandbox, authentication, or lifecycle
change requires independent reassessment before the claim can move.
