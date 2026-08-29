# ADR-0060: Copilot Programmatic-Prompt Guided Delivery

- Status: Accepted; implementation and live admission pending
- Date: 2026-08-28
- Deciders: Impresari Context maintainers
- Related: [CI-3c PRD](../product/ci-3c-copilot-cli-guided-delivery-prd.md),
  [CI-3c ARD](../architecture/ci-3c-copilot-cli-guided-delivery-ard.md),
  [ADR-0042](0042-planner-backed-guided-context-delivery.md), and
  [ADR-0046](0046-explicit-guided-delivery-intent-contract.md)

## Decision

Implement a narrow GitHub Copilot CLI L3 adapter using the documented
non-interactive `--prompt` surface. The adapter injects a separately reviewed,
byte-verified packet envelope at prompt start, so delivery does not depend on
model selection of an MCP tool.

Copilot runs in an empty disposable directory with all built-in MCP servers,
model tools, custom instructions, temporary-directory access, user questions,
remote control/export, and auto-update disabled. Provider network remains
available only because the hosted Copilot model requires it. The adapter grants
no URL or network-capable tool and passes no source or cache path.

Authentication is owned by Copilot in a caller-supplied dedicated
`COPILOT_HOME`. Impresari validates only the directory boundary; it never reads,
copies, exports, or deletes credential state.

## Consequences

- The supported lifecycle is programmatic prompt start, not interactive chat,
  ACP, MCP tool selection, VS Code, or autopilot.
- A successful process is insufficient: the adapter requires the admitted JSON
  terminal event and rejects any tool-execution event.
- Exact-version and platform admission remains pending until two successful
  live records and the complete local/hosted gates pass.
- Provider network is honestly disclosed rather than mislabeled as a
  no-network sandbox.
