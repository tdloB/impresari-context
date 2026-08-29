# CI-3f VS Code Copilot Guided Delivery — Architecture Requirements and Design

- Status: Implemented and live-admitted for the recorded scope
- Date: 2026-08-29
- Product requirement: [CI-3f PRD](../product/ci-3f-vscode-copilot-guided-delivery-prd.md)
- Decision: [ADR-0063](../decisions/0063-vscode-chat-cli-guided-delivery.md)

## Boundary

VS Code documents a chat CLI that opens a session with a supplied prompt, Ask
mode, a new window, and stdin. It does not expose the model response as a
machine-readable terminal stream. The safe boundary therefore separates exact
local launch evidence from explicit operator confirmation and never equates a
successful launcher exit with provider delivery.

## Components

- `context-adapters` admits the exact VS Code identity tuple.
- `context-vscode-copilot` owns canonical envelope construction, preview
  rehydration, empty-runtime launch, pending receipt, and exact confirmation.
- `context-cli` exposes separately gated `preview`, `apply`, and `confirm`.
- `vscode-copilot-delivery.schema.json` publishes the contract.

## Trust and authority rules

- Apply requires `--apply`, the exact expected packet, and complete preview
  rehydration before I/O.
- The fixed launch is `code chat --mode ask --new-window <bounded-prompt> -`
  with the exact envelope supplied as stdin context and an empty disposable
  cwd. VS Code requires the positional prompt to submit the turn; stdin alone
  only stages context. No source path or `--add-file` is passed.
- The child environment is cleared except `HOME`, `PATH`, and `TMPDIR`; no
  provider token is read or forwarded. VS Code owns its existing signed-in
  profile in place.
- No extension, profile, trust, MCP, tool, edit, agent, file, folder, or
  existing-window authority is installed or selected.
- Launcher output is bounded and discarded. The model response is never read
  programmatically. Confirmation binds only the exact visibly acknowledged
  packet ID to the pending receipt.

## Failure and cleanup

Pre-launch failures are `no_delivery`. Post-start ambiguity is `degraded`.
Successful launch is `confirmation_required`, never `delivered`. A private
empty runtime is removed on every return path. There is no broader fallback.

## Verification

Typed tests and fake transports cover local bindings and negative states. Live
admission requires two separately authorized synthetic handoffs plus operator
observation, source immutability, and cleanup evidence.
