# ADR-0063: Deliver reviewed packets through the VS Code chat CLI

- Status: Accepted; `1.134.0` live admission recorded
- Date: 2026-08-29
- Deciders: Impresari Context maintainers

## Context

VS Code Copilot L1/L2 proves managed extension-host MCP configuration and an
owned instruction, but both still depend on model tool selection. VS Code
documents `code chat` as a user-facing lifecycle that opens chat with a supplied
prompt, supports Ask mode and new windows, and accepts stdin. It does not return
a machine-readable model response.

## Decision

Admit only VS Code `1.134.0` through explicit preview/apply/confirm using
`code chat --mode ask --new-window <bounded-prompt> -` in an empty disposable
cwd. Pipe the exact reviewed packet envelope as stdin context, use the bounded
positional prompt to submit the turn, pass no source or source-file attachment,
and require a separately bound operator observation of Copilot's exact packet-ID
acknowledgment. Never infer delivery from launcher success.

Do not build an Impresari VS Code extension, call the Language Model API,
register a chat participant or tool, rewrite prompts, reuse an existing window,
or select Agent/Edit mode, MCP, trust, files, folders, or profiles.

## Consequences

- Delivery no longer depends on model-selected Impresari tools.
- The path remains explicit and uses stable documented VS Code CLI behavior.
- The final receipt includes human observation because the official surface has
  no machine-readable response stream. This is narrower and less automatable
  than the Copilot CLI, Claude Code, or Cursor L3 paths.
- Upstream CLI or UI drift withdraws the recorded claim.
