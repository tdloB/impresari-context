# Impresari Context — CI-3d: Claude Code Guided-Delivery PRD

- Status: Approved for implementation; live admission pending
- Date: 2026-08-29
- Authority: Founder-approved client-integration roadmap and autonomous delivery directive
- Governing roadmap: [Client Integration Depth Roadmap](client-integration-roadmap.md)
- Dependency: [CI-3a delivery-intent contract](client-integration-l3-delivery-intent-prd.md)
- Architecture requirements: [CI-3d Claude Code delivery ARD](../architecture/ci-3d-claude-code-guided-delivery-ard.md)

## Objective

Deliver one separately reviewed deterministic packet to Claude Code's documented
programmatic print surface without depending on model tool choice or installing
persistent hooks. Claude receives no source workspace, MCP server, built-in
tool, slash command, session persistence, or mutation grant.

## Scope

- Admit only Claude Code `2.1.241` on macOS aarch64 through one safe-mode,
  non-interactive print invocation with streaming JSON input and output.
- Require CI-3a's exact `claude_code` / `safe_mode_print` / `prompt_start`
  identity, explicit consent, verified snapshot, planner packet, canonical
  packet identity, and hard budget.
- Split preview and apply. Apply rehydrates the preview, re-verifies every
  binding, requires an expected packet ID and `--apply`, then starts Claude.
- Use an empty disposable runtime and a caller-named authenticated user home in
  place. Impresari never reads, copies, exports, or deletes credential state.
- Require the exact echoed prompt, an initialization event with empty tool and
  MCP inventories, no `tool_use` content, and one successful terminal result.
- Retain no model response content and remove only the disposable runtime.

## Non-goals

Interactive sessions, installed hooks, CLAUDE.md, skills, plugins, custom
commands or agents, project MCP, source reads, model-selected retrieval,
provider proxying, credential discovery, model-output retention, or claims for
unadmitted Claude Code versions.

## Acceptance criteria

- Preview and unapplied apply perform no Claude process or provider I/O.
- Canonical bytes, envelope, packet/plan/snapshot identities, client version,
  lifecycle, and preparation receipt are verified before process I/O.
- The child receives the prompt on stdin, an empty working directory, safe
  mode, no tools, no MCP inventory, and no session persistence.
- Unsupported version, unavailable authentication, malformed events, altered
  prompt echo, nonempty tool/MCP inventory, any tool use, timeout, or nonzero
  exit fails closed with a stable reason.
- Tests cover exact envelope bytes, preview alteration, preview purity,
  runtime/home separation, fixed command construction, event validation,
  bounded stderr, and cleanup.
- L3 admission requires two successful live records with immutable source,
  exact packet binding, zero tool executions, and explicit authorization for
  sending bounded workspace-derived evidence to Anthropic's service.

## Reassessment checkpoint

Any Claude version, authentication, safe-mode, print/event contract, platform,
or lifecycle change requires independent reassessment. This surface is not
evidence for interactive Claude Code or persistent hooks.
