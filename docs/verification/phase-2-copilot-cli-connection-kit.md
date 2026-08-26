# Phase 2 GitHub Copilot CLI connection-kit record

- Status: Generic local MCP; bounded temporary-config lifecycle and packet-equivalence recorded
- Date: 2026-08-25
- Client: GitHub Copilot CLI `1.0.80`, macOS aarch64

## Evidence recorded

After user-owned installation and sign-in, an isolated rehearsal used the
versioned managed kit to explicitly install, validate, and remove a temporary
MCP configuration for this session only. The test disabled Copilot's built-in
MCP server, remote control, automatic update, and custom instructions. It made
only the four core session/packet MCP tools available and used automatic
approval only for that already-restricted temporary tool set; it did not grant
any built-in, shell, file, network, or persistent authority.

The model-directed event stream completed `context_session_open`,
`context_build`, `context_packet_resolve`, and `context_session_close` in that
order. It required each tool execution to succeed with a structured result,
required the resolved packet to equal Copilot's delivered packet, and proved
that packet exactly equals an independent direct MCP packet from the same
fixture. The temporary configuration target was absent after exact removal,
the workspace digest was unchanged, and no persistent client MCP configuration
changed.

The rehearsal also supplies a malformed disposable additional-MCP configuration
to Copilot in `--malformed-copilot-config-only` mode. The client must reject it
before any tool call; the source workspace digest is verified unchanged.

## Classification and remaining gaps

Copilot remains **Generic local MCP**. A conversational model's tool choice is
live smoke evidence, not deterministic client conformance. The record proves
managed temporary-configuration parsing, malformed configuration rejection,
complete bounded lifecycle, direct packet equivalence, exact removal, and
source immutability on Copilot CLI `1.0.80` macOS aarch64. First-class
admission still needs a user-reviewed project-local `.mcp.json` install/trust/
single-entry-removal record for that declared scope.

## Gemini CLI note

Gemini CLI `0.56.0` authenticated successfully, but its current free-tier
service rejected normal client startup as unsupported. This is an external
service eligibility blocker, not a product or configuration failure in
Impresari Context. No Gemini lifecycle claim is made.
