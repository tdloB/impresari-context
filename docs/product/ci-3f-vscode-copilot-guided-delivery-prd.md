# Impresari Context — CI-3f: VS Code Copilot Guided-Delivery PRD

- Status: Admitted for VS Code `1.134.0` on macOS arm64
- Date: 2026-08-29
- Authority: Founder-approved client-integration roadmap
- Governing roadmap: [Client Integration Depth Roadmap](client-integration-roadmap.md)
- Dependency: [CI-3a delivery-intent contract](client-integration-l3-delivery-intent-prd.md)
- Architecture requirements: [CI-3f ARD](../architecture/ci-3f-vscode-copilot-guided-delivery-ard.md)

## Objective

Deliver one separately reviewed deterministic packet to VS Code Copilot's
documented chat CLI without relying on model-selected MCP tools or exposing the
source workspace. Because this UI-opening surface has no machine-readable model
response stream, launcher success is never delivery proof; an exact operator-
observed packet acknowledgment is required to complete the receipt.

## Scope

- Admit only VS Code `1.134.0` on macOS arm64 through `code chat --mode ask
  --new-window <bounded-prompt> -` from an empty disposable cwd.
- Require the exact CI-3a `vscode_copilot` / `chat_cli_ask` / `chat_open`
  identity, explicit consent, current snapshot, planner packet, budget, and
  canonical packet identity.
- Split preview, apply, and confirmation. Rehydrate every preview binding
  before process I/O. Apply requires the global `--apply` gate and expected
  packet ID. Confirmation requires the pending receipt plus identical expected
  and operator-observed packet IDs.
- Pipe only the reviewed envelope through stdin context and pass a bounded
  positional prompt that submits the turn and requests the exact packet ID.
  Pass no workspace, source-file, folder, profile, extension, MCP, tool, trust,
  approval, edit, or agent flag.
- Use existing VS Code authentication in place without inspecting, copying,
  exporting, printing, or deleting credential state.

## Non-goals

An Impresari extension, Language Model API request, chat participant, prompt
rewriter, prompt file, source attachment, existing-window reuse, Agent/Edit
mode, MCP selection, automated UI inspection, provider-response retention,
credential discovery, or inferred delivery from a zero launcher exit.

## Acceptance criteria

- Preview and unapplied apply start no VS Code process.
- Unsupported versions, altered previews, wrong packet IDs, malformed receipts,
  launcher failures, timeouts, oversized output, and mismatched observations
  fail closed.
- Tests cover exact packet bytes, preview rehydration, launch ambiguity,
  confirmation binding, path separation, authority denial, and cleanup.
- L3 admission requires two authorized synthetic launches where the operator
  visibly observes Copilot acknowledge each exact packet ID and the source hash
  plus runtime cleanup remain unchanged.

## Reassessment checkpoint

Any VS Code version, `code chat`, Ask-mode, stdin, profile, Copilot, or UI
receipt change requires reassessment. A future machine-readable response stream
may replace manual confirmation only after an independent decision record.
