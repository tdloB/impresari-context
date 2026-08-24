# Phase 1 real-client admission runbook

- Status: Claude Code lifecycle recorded; Cursor awaiting user-owned sign-in
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
preserved the fixture workspace, and confirmed that no persistent MCP server
named `impresari_context_conformance` was registered.

The result remains Generic local MCP because model-directed tool selection is
not deterministic. The detailed evidence and remaining First-class criteria
are in the [Claude Code kit record](phase-1-claude-code-connection-kit.md).

The eventual user-reviewed registration, verification, and single-entry
removal commands are in the [local MCP connection guide](../reference/local-mcp-connection-guides.md#claude-code).

## Cursor admission prerequisite

1. Sign into the installed Cursor application or Cursor Agent CLI using its
   normal interactive flow.
2. Confirm the authenticated Agent CLI can execute the documented read-only
   `agent mcp list` command.
3. Tell Codex that Cursor is ready. Do not create or enable an Impresari
   Context MCP entry yet; the admission rehearsal will validate the exact
   project or user entry before use.

The eventual user-reviewed configuration, inspection, and single-entry removal
guidance is in the [local MCP connection guide](../reference/local-mcp-connection-guides.md#cursor).

## Evidence to collect after each prerequisite

For each client, the admission record will capture:

- client version and OS/architecture;
- exact supported configuration scope and fixed local-stdio launch shape;
- tool discovery and complete session lifecycle against an isolated fixture;
- packet identity/equivalence and source-immutability results;
- malformed configuration behavior; and
- safe removal of only the Impresari Context entry.

No client is promoted to **First-class** until the public matrix and its
machine-readable manifest contain this evidence.
