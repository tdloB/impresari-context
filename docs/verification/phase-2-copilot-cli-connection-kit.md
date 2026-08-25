# Phase 2 GitHub Copilot CLI connection-kit record

- Status: Generic local MCP; managed temporary-config model-directed session-open recorded
- Date: 2026-08-25
- Client: GitHub Copilot CLI `1.0.80`, macOS aarch64

## Evidence recorded

After user-owned installation, sign-in, and workspace trust, an isolated
rehearsal used the versioned managed kit to explicitly install, validate, and
remove a temporary MCP configuration for this session only. The test disabled
Copilot's built-in MCP server, remote control, automatic update, and custom
instructions; it exposed and permitted only
`impresari-context(context_session_open)`. The model-directed client event
stream included that exact MCP tool. The temporary configuration target was
absent after exact removal, the workspace digest was unchanged, and no
persistent client MCP configuration changed.

The rehearsal also supplies a malformed disposable additional-MCP configuration
to Copilot in `--malformed-copilot-config-only` mode. The client must reject it
before any tool call; the source workspace digest is verified unchanged.

## Classification and remaining gaps

Copilot remains **Generic local MCP**. A conversational model's tool choice
cannot establish deterministic client conformance. First-class admission still
requires a complete lifecycle and packet-equivalence record plus supported
version/operating-system coverage. This current record proves managed
configuration parsing/discovery, malformed configuration rejection, exact
removal, and a bounded one-tool real-client smoke only.

## Gemini CLI note

Gemini CLI `0.56.0` authenticated successfully, but its current free-tier
service rejected normal client startup as unsupported. This is an external
service eligibility blocker, not a product or configuration failure in
Impresari Context. No Gemini lifecycle claim is made.
