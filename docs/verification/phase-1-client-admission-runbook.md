# Phase 1 real-client admission runbook

- Status: Claude Code live temporary-configuration lifecycle and Cursor temporary configuration discovery recorded
- Date: 2026-08-23
- Scope: Claude Code and Cursor only

## Why this requires a person

Impresari Context deliberately does not install third-party clients, authenticate
into third-party accounts, or create, enable, disable, or remove their MCP
configuration. Those actions grant access or change external client state.
The user performs the one-time authentication step; all subsequent inspection
and Impresari Context conformance work is read-only and can proceed here.

Codex does not share this blocker: its deterministic local App Server
conformance is recorded in the [Codex connection-kit record](phase-1-codex-connection-kit.md).

## Claude Code admission record

Completed on macOS aarch64 with Claude Code CLI `2.1.241` after user-owned
installation and authentication. The one-run rehearsal used temporary
`--mcp-config` and `--strict-mcp-config`, completed the fixed MCP lifecycle,
proved direct-MCP packet equivalence, preserved the fixture workspace, and
confirmed that no persistent MCP server named `impresari-context` was
registered.

The result remains Generic local MCP until the user-reviewed local-scope
installation and single-entry removal record is completed. The detailed
evidence and remaining First-class criteria are in the [Claude Code kit
record](phase-1-claude-code-connection-kit.md).

The eventual user-reviewed registration, verification, and single-entry
removal commands are in the [local MCP connection guide](../reference/local-mcp-connection-guides.md#claude-code).

## Cursor preadmission record

Completed after the user signed into Cursor Agent CLI `3.17.8` on macOS
aarch64. A development-only rehearsal made an isolated temporary workspace,
separate cache, and `.cursor/mcp.json`; `cursor agent mcp list` discovered the
fixed local-stdio server. It did not invoke `enable`, did not call an AI model,
and preserved the temporary workspace after the configuration was written.

The Cursor CLI treats MCP enablement as a change to local approval state. A
real tool lifecycle therefore requires an intentional, user-owned approval of
the exact temporary entry. It must not be replaced by automatic approval.

The eventual user-reviewed configuration, inspection, and single-entry removal
guidance is in the [local MCP connection guide](../reference/local-mcp-connection-guides.md#cursor).

## Evidence still required for first-class admission

For each client, the admission record will capture:

- client version and OS/architecture;
- exact supported configuration scope and fixed local-stdio launch shape;
- tool discovery and complete session lifecycle against an isolated fixture;
- packet identity/equivalence and source-immutability results;
- malformed configuration behavior; and
- safe removal of only the Impresari Context entry.

No client is promoted to **First-class** until the public matrix and its
machine-readable manifest contain this evidence.
