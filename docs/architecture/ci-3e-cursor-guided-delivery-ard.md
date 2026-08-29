# Impresari Context — CI-3e Cursor Guided-Delivery ARD

- Status: Implemented; recorded-scope L3 admitted
- Date: 2026-08-29
- Product requirement: [CI-3e PRD](../product/ci-3e-cursor-guided-delivery-prd.md)
- Decision: [ADR-0062](../decisions/0062-cursor-ask-mode-programmatic-guided-delivery.md)

## Boundary

Cursor's print surface is programmatic but retains tool capability. The safe
boundary is therefore process and workspace isolation rather than a claim that
Cursor has no tool definitions. Impresari supplies one reviewed packet to an
ask-mode, sandbox-enabled process in an empty disposable workspace and rejects
every observed `tool_call` event.

## Components

- `context-adapters` admits the exact Cursor identity tuple.
- `context-cursor-agent` owns envelope construction, preview rehydration,
  silent authentication status, process isolation, stream validation, and the
  bounded receipt.
- `context-cli` exposes `client delivery cursor preview` and separately gated
  `client delivery cursor apply`.
- `cursor-agent-delivery.schema.json` publishes the contract.

## Trust and authority rules

- Apply requires `--apply`, the expected packet identity, and exact preview
  rehydration before client I/O.
- Binary, runtime parent, and authenticated user home are absolute, canonical,
  real, and non-overlapping.
- The environment is cleared except `HOME`, `PATH`, and an already-present
  `CURSOR_API_KEY` passed only to Cursor. Existing login configuration remains
  in the named home and is used in place.
- Cursor's `status` output is discarded; only success/failure crosses the
  boundary. This result is carried explicitly and is never inferred.
- The fixed invocation is `--print --output-format stream-json --mode ask
  --sandbox enabled --trust --workspace <empty-runtime>` with the prompt on
  stdin. Trust applies only to the newly created empty runtime required by
  Cursor's non-interactive lifecycle.
- Source and cache paths never enter the child. No source trust, force, auto-review,
  MCP approval, plugin, additional directory, worktree, or resume flag exists.
- Output is bounded and model text is discarded. Any tool-call start degrades
  delivery even if Cursor later reports success.

## Failure and cleanup

Pre-I/O failures are `no_delivery`; ambiguous post-start evidence is
`degraded`. There is no broader fallback. A private runtime directory is
created for one attempt and exactly removed on every return path.

## Verification

Deterministic fake transports and typed unit tests cover bindings and negative
events. Live admission requires two authorized external deliveries, with the
source hash and empty runtime parent verified after each attempt.

Two authorized deliveries satisfying those requirements are recorded in the
[CI-3e verification record](../verification/ci-3e-cursor-guided-delivery.md).
