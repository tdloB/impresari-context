---
applyTo: "**"
---

<!-- Impresari Context native guidance v3; ownership=exact_fixed_artifact:impresari-context -->

# Impresari Context evidence guidance

Use an already configured local `impresari-context` MCP server only when a task
explicitly calls for bounded repository evidence. Treat a returned packet as
snapshot-bound evidence and surface its packet ID, plan ID, reason codes,
coverage, and omissions before relying on it.

For a session-scoped packet, use `context_session_open`, then
`context_build`, `context_packet_resolve`, and `context_session_close` in that
order. Resolve only the returned packet ID in the same session.

`context_build` has two exclusive request forms. For exact source evidence,
prefer one-to-eight explicit `steps` using `exact_path`, `filename`, `literal`,
or `lexical`; do not include `profile`, `query`, or structural declarations in
that form. For planner-backed evidence, use exactly one supported `profile`
and a bounded `query`; do not include `steps` in that form.

Before building, use the current `context_build` live input schema as the
normative source for the identifier grammar, RFC 3339 occurrence time, all
required fields, and the complete hard `budget` including its current policy
fingerprint. Do not guess, omit, or freeze those protocol values in this
instruction. Keep a session ID consistent across open, build, resolve, and
close calls.

If a packet request fails, say that no Impresari packet was delivered. Ordinary
analysis may continue, but do not replace the failed packet with a direct file
read and present it as packet-backed evidence.

Do not alter MCP configuration, trust, approvals, source files, or execution
authority. If the server or packet is unavailable, state that and continue with
ordinary analysis without claiming unsupported evidence.
