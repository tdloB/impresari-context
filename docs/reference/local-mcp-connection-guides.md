# Local MCP Connection Guides

- Version: 1.0
- Published: 2026-08-23
- Classification: Generic local MCP guides, not first-class integrations
- Supported transport: local stdio only

These guides render the fixed-launch contract for three common coding clients.
They do **not** run any command, modify a client configuration, install a hook,
or edit a repository. A person must review and invoke any shown command or file
change themselves.

Before using a guide, build or install a specific local
`impresari-context-mcp` binary. Do not substitute an unpinned `latest` download
command. Pick an existing, dedicated cache directory outside the source
workspace. The MCP process never executes workspace code.

Every configuration must preserve these fixed launch values:

- `--workspace`: the one intended source workspace;
- `--cache`: an existing separate cache directory, never a workspace child;
- `--consumer-id`: an opaque stable identifier selected by the integrator; and
- `--role`: the intended policy role.

The process now records its local UTC startup time by default. Do not add a
fixed `--occurred-at` value to a persistent client configuration; reserve that
flag for deterministic tests and rehearsals.

## Codex

Codex's local CLI exposes `codex mcp add <name> -- <command> [args...]`. After
reviewing the absolute paths and opaque identifiers, a user may manually run:

```text
codex mcp add impresari-context -- \
  /absolute/path/to/impresari-context-mcp \
  --workspace /absolute/path/to/source-workspace \
  --cache /absolute/path/to/separate-cache \
  --consumer-id consumer_codex_local \
  --role local_user
```

Verify the client-managed configuration with `codex mcp get
impresari-context`. Remove only the owned entry with `codex mcp remove
impresari-context`. A limited local-stdio lifecycle check has passed for Codex
CLI `0.149.0-alpha.4.1`; its exact scope and remaining gaps are recorded in the
[Codex conformance record](../verification/phase-0-codex-local-mcp-conformance.md).
This guide still makes no assertion about a maintained Codex configuration
scope, version range, or packet-corpus equivalence; those are required before
first-class admission.

## Claude Code

Claude Code documents local stdio registration through `claude mcp add`. The
following is a user-invoked, local-scope example:

```text
claude mcp add impresari-context --scope local -- \
  /absolute/path/to/impresari-context-mcp \
  --workspace /absolute/path/to/source-workspace \
  --cache /absolute/path/to/separate-cache \
  --consumer-id consumer_claude_local \
  --role local_user
```

Verify with `claude mcp get impresari-context`; remove only the same entry with
`claude mcp remove impresari-context`. Do not use a project-shared `.mcp.json`
unless the team has separately reviewed the fixed workspace, separate cache,
binary provenance, and source-control consequences.

When validating a JSON configuration file before use, run:

```text
impresari-context doctor claude-config <workspace-root> <separate-cache> <mcp-json>
```

## Cursor

Cursor documents local stdio entries in a project `.cursor/mcp.json` or user
`~/.cursor/mcp.json`. The project location is an external repository change and
therefore is not created by Impresari Context. A user who has intentionally
chosen the project scope may review an entry such as:

```json
{
  "mcpServers": {
    "impresari-context": {
      "type": "stdio",
      "command": "/absolute/path/to/impresari-context-mcp",
      "args": [
        "--workspace", "${workspaceFolder}",
        "--cache", "${env:IMPRESARI_CONTEXT_CACHE}",
        "--consumer-id", "consumer_cursor_local",
        "--role", "local_user"
      ]
    }
  }
}
```

`IMPRESARI_CONTEXT_CACHE` must resolve to an existing directory outside the
workspace. Never set it to `${workspaceFolder}/.cache` or any other workspace
child. Cursor's own UI can enable, disable, and remove the entry; removal must
affect only `impresari-context` and preserve other `mcpServers` entries.

Before allowing Cursor to load it, run:

```text
impresari-context doctor cursor-config <workspace-root> <separate-cache> .cursor/mcp.json
```

## What these guides do not establish

These guides do not yet establish a maintained version range, supported OS
matrix per client, clean-install behavior, direct-engine/MCP packet equivalence,
configuration-parser conformance, safe automated removal, or client lifecycle
coverage. The [compatibility matrix](compatibility-matrix.md) therefore keeps
Codex, Claude Code, and Cursor in the **Generic local MCP** category.

Source references for the host-side configuration surfaces: the installed
Codex CLI help, [Claude Code MCP documentation](https://docs.anthropic.com/en/docs/claude-code/mcp),
and [Cursor MCP documentation](https://cursor.com/docs/mcp).
