# ADR-0014: Local stdio MCP transport

- Status: Accepted for implementation
- Date: 2026-08-22
- Scope: Local MCP interoperability for the existing engine and process-local sessions

## Context

Consumers need a standard integration path without adopting an Impresari
orchestrator. MCP supports a child-process stdio transport in which newline-
delimited UTF-8 JSON-RPC messages travel over stdin/stdout. A network transport
would add authentication, origin validation, tenancy, service operation, and
remote-data risks that are not approved.

## Decision

Implement a separate `impresari-context-mcp` binary that prefers MCP revision
`2025-11-25` and accepts the compatible `2025-06-18` revision. During
initialization it returns the supported revision requested by the client. It
supports lifecycle initialization, ping, tool discovery, and a small fixed tool
set over stdio only.

- The workspace and cache roots are fixed by trusted process-launch arguments,
  never selected by tool input.
- Stdout contains MCP messages only; diagnostics use stderr.
- Input is newline-delimited, UTF-8 JSON-RPC 2.0 with bounded line size, bounded
  requests, no batching, and strict schemas.
- The server advertises tools only. It does not request roots, sampling,
  elicitation, prompts, logging, network access, or client-side execution.
- Tools delegate to the public engine/session APIs and add no orchestration,
  approval, native-read, execution, or filesystem authority.
- Process-local session references expire when closed or when the stdio process
  exits. No durable memory is created.
- No HTTP, sockets, background daemon, authentication system, or remote access
  is included.
- Repository text and tool results remain data and never become protocol or
  policy instructions.

## Initial tools

- `context_build`: build one verified packet from a bounded context plan and
  optionally attach it to an already-open process-local session.
- `context_session_open`: open a bounded consumer-owned session.
- `context_packet_resolve`: resolve an immutable packet reference for its owner.
- `context_session_close`: close a session and invalidate its references.

No tool mutates source, executes repository commands, invokes models, reaches a
network, or grants fallback access.

## Verification

- Lifecycle ordering, version negotiation, ping, and tool-list conformance.
- Direct-engine/MCP packet equivalence.
- Strict malformed, oversized, batch, duplicate-ID, pre-initialization, unknown-
  method, hostile-text, and stdout-purity tests.
- Session ownership, limits, close, and process-exit invalidation tests.
- Source workspace before/after immutability checks.

## Explicitly deferred

HTTP/remote transport, authentication, executable extensions, privileged
extension capabilities, sampling, elicitation, durable sessions, and server-
initiated agent behavior require new ADR and threat-model approval.

## References

- [MCP lifecycle, revision 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle)
- [MCP lifecycle, revision 2025-06-18](https://modelcontextprotocol.io/specification/2025-06-18/basic/lifecycle)
- [MCP transports](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports)
- [MCP tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)
