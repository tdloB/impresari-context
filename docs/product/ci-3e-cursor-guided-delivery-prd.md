# Impresari Context — CI-3e: Cursor Guided-Delivery PRD

- Status: Implemented; recorded-scope L3 admitted
- Date: 2026-08-29
- Authority: Founder-approved client-integration roadmap and explicit Cursor
  credential-boundary and external-delivery authorization
- Governing roadmap: [Client Integration Depth Roadmap](client-integration-roadmap.md)
- Dependency: [CI-3a delivery-intent contract](client-integration-l3-delivery-intent-prd.md)
- Architecture requirements: [CI-3e Cursor delivery ARD](../architecture/ci-3e-cursor-guided-delivery-ard.md)

## Objective

Deliver one separately reviewed deterministic packet to Cursor Agent's
documented programmatic print surface without relying on model tool selection
or exposing the source workspace. Cursor runs in ask mode with sandboxing in an
empty disposable workspace; any observed tool call invalidates delivery.

## Scope

- Admit only Cursor Agent CLI `2026.08.25-3e8eec8` on macOS aarch64 through
  one ask-mode, sandbox-enabled, non-interactive stream-JSON invocation.
- Require CI-3a's exact `cursor_agent` / `ask_mode_print` / `prompt_start`
  identity, explicit consent, snapshot, planner packet, hard budget, and
  canonical packet identity.
- Split preview and apply. Rehydrate and verify every binding before process
  I/O and require the expected packet ID plus `--apply`.
- Supply the prompt over stdin and set both cwd and `--workspace` to the same
  empty disposable runtime. Pass `--trust` only for that newly created empty
  runtime because Cursor otherwise refuses non-interactive execution. Never
  pass or trust the source workspace or cache.
- Use the caller-named authenticated user home in place. Run Cursor's silent
  `status` preflight and carry its boolean result explicitly. Never inspect,
  retain, print, copy, export, or delete account or credential content.
- Require exact prompt echo, exact runtime cwd initialization, zero tool-call
  starts, one successful terminal result, zero exit, bounded output, and exact
  runtime cleanup.

## Non-goals

Hooks, cloud workers, resumed sessions, source-workspace trust, force/yolo,
auto-review, MCP approval, plugins, worktrees, source reads, model-selected
retrieval, provider proxying, response retention, credential discovery, or
claims for other Cursor versions and surfaces.

## Acceptance criteria

- Preview and unapplied apply perform no Cursor process or provider I/O.
- Unsupported version, failed authentication status, malformed or oversized
  events, prompt or cwd drift, any tool call, timeout, and nonzero exit fail
  closed with stable reasons.
- Tests cover exact packet bytes, altered preview, preview purity, runtime/home
  separation, auth-status evidence, tool rejection, event validation, bounded
  stderr, source immutability, and cleanup.
- L3 admission requires two authorized live synthetic records with exact
  packet binding, successful auth status, immutable source, zero tool calls,
  and clean runtime removal.

## Reassessment checkpoint

Any Cursor version, print/event, ask-mode, sandbox, authentication, workspace,
or lifecycle change requires independent reassessment. Cursor documentation
states that non-interactive mode has broad tool access, so empty-workspace
isolation and observed zero tool calls remain mandatory.
