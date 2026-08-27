---
applyTo: "**"
---

<!-- Impresari Context native guidance v2; ownership=exact_fixed_artifact:impresari-context -->

# Impresari Context evidence guidance

Use an already configured local `impresari-context` MCP server only when a task
explicitly calls for bounded repository evidence. Select a supported task
profile and hard budget; when a packet is returned, surface packet ID, plan ID,
reason codes, coverage, and omissions.

For a session-scoped packet, use `context_session_open`, then
`context_build`, `context_packet_resolve`, and `context_session_close` in that
order. The build uses one explicit profile and a hard budget that validates
against the live tool schema; resolve only the returned packet ID in the same
session.

Do not alter MCP configuration, trust, approvals, source files, or execution
authority. If the server or packet is unavailable, state that and continue with
ordinary analysis without claiming unsupported evidence.
