# Phase 1 real-client admission runbook

- Status: Codex, Claude Code, and Cursor L1 admitted for recorded scopes
- Date: 2026-08-26
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

A second native rehearsal started with an explicit empty Claude `HOME` under
`/private/tmp`, used `claude mcp add/get/remove --scope local`, verified the
fixed connected stdio entry, removed exactly that entry, and confirmed its
absence. It did not inspect or change the user's actual Claude home. Together
these records admit Claude Code as **First-class** only for CLI `2.1.241` on
macOS aarch64. The detailed evidence and recorded-scope limits are in the
[Claude Code kit record](phase-1-claude-code-connection-kit.md).

## Cursor admission record

Completed after the user signed into Cursor Agent CLI `3.17.8`
(`2026.08.11-e8db854`) on macOS aarch64. A development-only rehearsal made an
isolated temporary workspace, separate cache, and `.cursor/mcp.json`; `cursor
agent mcp list` discovered the fixed local-stdio server.

The same isolated rehearsal supplied malformed `.cursor/mcp.json`. Cursor CLI
`3.17.8` exited without loading any MCP server, did not expose fixture source
in its diagnostic, and did not alter the fixture. This is a fail-closed
configuration behavior rather than a parse-error exit and is recorded as such.

The native lifecycle started from an explicit empty disposable project,
performed `cursor agent mcp enable/list-tools/disable` only for
`impresari-context`, and removed the owned entry. The bounded Agent-mode
lifecycle allowed only the four named Impresari MCP tools through a test-only
project permission file; shell, source read/write, and web calls were denied.
It returned a packet identical to the direct MCP control packet and preserved
the fixture source. Cursor Ask mode was recorded as blocking dynamic MCP
execution, so it is not the exercised lifecycle surface.

The rehearsal has a preview-first disposable-project preparation path:

```text
ruby scripts/rehearse-cursor-preadmission.rb \
  --prepare-project-root /private/tmp/impresari-cursor-l1-admission
# inspect the returned paths, then rerun with --apply
```

It creates only an empty `workspace` and separate `cache` under `/private/tmp`.
The native runner is separately preview-first and uses those exact bounded
paths for entry, approval, packet, and removal evidence.

## Evidence still required for first-class admission

For each client, the admission record will capture:

- client version and OS/architecture;
- exact supported configuration scope and fixed local-stdio launch shape;
- tool discovery and complete session lifecycle against an isolated fixture;
- packet identity/equivalence and source-immutability results;
- malformed configuration behavior; and
- safe removal of only the Impresari Context entry.

The public matrix and its machine-readable manifest promote only clients whose
individual evidence record is complete. Codex, Claude Code, and Cursor are
admitted for their explicitly recorded scopes; GitHub Copilot remains pending.
