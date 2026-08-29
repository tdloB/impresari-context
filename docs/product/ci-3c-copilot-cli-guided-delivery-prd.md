# Impresari Context — CI-3c: GitHub Copilot CLI Guided-Delivery PRD

- Status: Approved for implementation; live admission pending
- Date: 2026-08-28
- Authority: Founder-approved client-integration roadmap and autonomous delivery directive
- Governing roadmap: [Client Integration Depth Roadmap](client-integration-roadmap.md)
- Dependency: [CI-3a delivery-intent contract](client-integration-l3-delivery-intent-prd.md)
- Architecture requirements: [CI-3c Copilot CLI delivery ARD](../architecture/ci-3c-copilot-cli-guided-delivery-ard.md)

## Objective

Deliver one separately reviewed deterministic packet to GitHub Copilot CLI's
documented programmatic prompt surface without depending on model tool choice.
The Copilot process receives no source workspace, MCP server, built-in tool,
URL permission, file permission, remote-control permission, or mutation grant.

## Scope

- Admit only GitHub Copilot CLI `1.0.80` on macOS aarch64 through one
  non-interactive `--prompt` invocation with JSON event output.
- Require CI-3a's exact `github_copilot_cli` / `programmatic_prompt` /
  `prompt_start` identity, explicit consent, verified snapshot, planner packet,
  canonical packet identity, and hard budget.
- Split preview and apply. Apply rehydrates the saved preview, re-verifies every
  binding, requires an expected packet ID and `--apply`, then starts the client.
- Use a caller-named, canonical, non-symlinked `COPILOT_HOME`, an explicitly
  named GitHub CLI authentication directory used in place, and a separate
  disposable runtime. Impresari never reads, copies, exports, or deletes
  credential state.
- Disable built-in MCP servers, remote control/export, auto-update, custom
  instructions, user questions, temporary-directory access, and every model
  tool. Retain only the provider network that Copilot itself requires.
- Record a bounded terminal receipt and remove only the disposable runtime.

## Non-goals

Interactive sessions, ACP, autopilot, repository instructions, project MCP,
source reads, tool execution, shell hooks, URL access, provider proxying,
credential discovery, model-output retention, or claims for VS Code Copilot.

## Acceptance criteria

- Preview and unapplied apply perform no client I/O.
- Canonical packet bytes, envelope, packet/plan/snapshot identities, client
  version, lifecycle, and preparation receipt are verified before process I/O.
- The child receives a bounded prompt, an empty disposable working directory,
  no tools, no source paths, no MCP configuration, and no remote-control path.
- Unsupported version, absent authentication, malformed JSON events, timeout,
  nonzero exit, or any observed tool execution fail closed with stable reasons.
- Tests cover exact envelope bytes, serialized-preview alteration, preview
  purity, home/runtime separation, zero-tool command construction, terminal
  event validation, bounded stderr, and cleanup.
- L3 admission requires two successful live records with immutable source,
  empty runtime parents, exact packet binding, zero tool executions, and no
  credential copy or deletion.

## Reassessment checkpoint

Any Copilot version, prompt/event contract, permission flag, authentication,
platform, or lifecycle change requires independent reassessment. Programmatic
prompt delivery is not evidence for the interactive CLI or VS Code extension.
