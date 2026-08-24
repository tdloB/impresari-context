# Local MCP Connection Guides

- Version: 1.3
- Published: 2026-08-23
- Classification: Generic local MCP guides, not first-class integrations
- Supported transport: local stdio only

These guides render the fixed-launch contract for six common coding clients.
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

Codex supports local stdio MCP servers and shares one configuration across the
ChatGPT desktop app, Codex CLI, and Codex IDE extension. The maintained
pre-admission kit uses a **trusted project** configuration at
`.codex/config.toml`; it never writes that file. A user must intentionally
create or edit the project file and review the exact absolute paths:

```toml
[mcp_servers."impresari-context"]
command = "/absolute/path/to/impresari-context-mcp"
args = [
  "--workspace", "/absolute/path/to/source-workspace",
  "--cache", "/absolute/path/to/separate-cache",
  "--consumer-id", "consumer_codex_local",
  "--role", "local_user"
]
enabled = true
default_tools_approval_mode = "prompt"
```

The project must be trusted before Codex loads `.codex/config.toml`. Validate
the exact entry without launching Codex or editing any configuration:

```text
impresari-context doctor codex-config \
  /absolute/path/to/source-workspace \
  /absolute/path/to/separate-cache \
  .codex/config.toml
```

Then inspect the resolved client entry with `codex mcp get impresari-context
--json`. To remove this project-scoped entry, manually delete only the
`[mcp_servers."impresari-context"]` table; do not delete the entire
`.codex/config.toml` file or unrelated server entries. The user-level
alternative is the Codex CLI command `codex mcp add <name> -- <command>
[args...]`; it is intentionally not automated or used by this kit.

This pre-admission kit has been locally checked against Codex CLI
`0.149.0-alpha.4.1` on macOS aarch64. Its exact evidence and remaining
first-class gaps are recorded in the [Phase 1 Codex kit record](../verification/phase-1-codex-connection-kit.md).

## Claude Code

Claude Code documents local stdio registration through `claude mcp add`. Its
current CLI defaults to local scope when no `--scope` is supplied; the
following is therefore a user-invoked local-scope example:

```text
claude mcp add --transport stdio impresari-context -- \
  /absolute/path/to/impresari-context-mcp \
  --workspace /absolute/path/to/source-workspace \
  --cache /absolute/path/to/separate-cache \
  --consumer-id consumer_claude_local \
  --role local_user
```

Verify with `claude mcp get impresari-context`; remove only the same entry with
`claude mcp remove impresari-context`. Do not use a project-shared `.mcp.json`
unless the team has separately reviewed the fixed workspace, separate cache,
binary provenance, and source-control consequences. In particular,
`--scope project` creates or updates `.mcp.json`; Impresari Context never runs
that mutation on the user's behalf.

When validating a JSON configuration file before use, run:

```text
impresari-context doctor claude-config <workspace-root> <separate-cache> <mcp-json>
```

The generic kit has additionally been exercised through Claude Code CLI
`2.1.241` on macOS aarch64 using an isolated one-run `--mcp-config` and
`--strict-mcp-config`; the test did not register a persistent server. Its
evidence and remaining admission gaps are recorded in the [Phase 1 Claude Code
kit record](../verification/phase-1-claude-code-connection-kit.md).

## Cursor

Cursor documents local stdio entries in a project `.cursor/mcp.json` or user
`~/.cursor/mcp.json`. The project location is an external repository change and
therefore is not created by Impresari Context. A user who has intentionally
chosen the project scope may review an entry such as:

```json
{
  "mcpServers": {
    "impresari-context": {
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
child. The documented Cursor stdio form infers its local transport from
`command` and `args`; the Impresari doctor accepts that form and also accepts
an explicit `"type": "stdio"`. It rejects `env`, remote-transport, and other
unrelated fields so the server cannot receive forwarded ambient environment.
Cursor's own UI can enable, disable, and remove the entry; removal must affect
only `impresari-context` and preserve other `mcpServers` entries.

Before allowing Cursor to load it, run:

```text
impresari-context doctor cursor-config <workspace-root> <separate-cache> .cursor/mcp.json
```

On a signed-in Cursor Agent CLI, `agent mcp list` reports the configured
server's source and transport, and `agent mcp list-tools <identifier>` lists
its available tools. Those are read-only inspection steps. Do not use
`agent mcp enable` or `agent mcp disable` as part of this kit: they change
Cursor's local approval state. An isolated project configuration has been
discovered by Cursor Agent CLI `3.17.8` on macOS aarch64 without enabling the
server; the evidence and remaining admission gaps are in the [Phase 1 Cursor
kit record](../verification/phase-1-cursor-connection-kit.md).

## Gemini CLI

Gemini CLI reads a project `.gemini/settings.json` `mcpServers` object. A user
who has reviewed that repository change may add only the fixed local-stdio
entry below. `trust: false` preserves Gemini's normal tool confirmation, while
`includeTools` prevents a future server tool from being silently exposed.

```json
{
  "mcpServers": {
    "impresari-context": {
      "command": "/absolute/path/to/impresari-context-mcp",
      "args": ["--workspace", "/absolute/path/to/source-workspace", "--cache", "/absolute/path/to/separate-cache", "--consumer-id", "consumer_gemini_local", "--role", "local_user"],
      "trust": false,
      "includeTools": ["context_session_open", "context_build", "context_packet_resolve", "context_session_close"]
    }
  }
}
```

Do not add Gemini's `env`, `cwd`, `url`, `httpUrl`, or headers fields. They are
not needed by Impresari Context. Validate the file without launching Gemini:

```text
impresari-context doctor gemini-config <workspace-root> <separate-cache> .gemini/settings.json
```

After a user starts Gemini in the reviewed workspace, `/mcp` is the
client-side inspection surface. Remove only the `impresari-context` object.

## GitHub Copilot CLI

GitHub Copilot CLI supports project-local `.mcp.json` and loads it after the
user confirms folder trust. It must not be installed with `copilot mcp add`,
because that command changes the user's global configuration.

```json
{
  "mcpServers": {
    "impresari-context": {
      "type": "local",
      "command": "/absolute/path/to/impresari-context-mcp",
      "args": ["--workspace", "/absolute/path/to/source-workspace", "--cache", "/absolute/path/to/separate-cache", "--consumer-id", "consumer_copilot_local", "--role", "local_user"],
      "tools": ["context_session_open", "context_build", "context_packet_resolve", "context_session_close"]
    }
  }
}
```

Do not add `env`, remote transport, headers, `--allow-all`, or automatic
approval flags. Validate before use:

```text
impresari-context doctor copilot-config <workspace-root> <separate-cache> .mcp.json
```

`copilot mcp list --json` is a read-only inspection step after user
installation, authentication, and folder trust. Remove only this project entry.

## VS Code Copilot

VS Code uses a workspace `.vscode/mcp.json` with a top-level `servers` object.
The user must review the file and VS Code's local-server prompt before start:

```json
{
  "servers": {
    "impresari-context": {
      "command": "/absolute/path/to/impresari-context-mcp",
      "args": ["--workspace", "${workspaceFolder}", "--cache", "/absolute/path/to/separate-cache", "--consumer-id", "consumer_vscode_local", "--role", "local_user"]
    }
  }
}
```

Do not add environment, URL, headers, input-variable, or automatic-approval
fields. Validate without launching VS Code:

```text
impresari-context doctor vscode-config <workspace-root> <separate-cache> .vscode/mcp.json
```

The MCP view provides inspection after the user opens the trusted workspace.
Remove only the `impresari-context` entry manually.

## What these guides do not establish

These guides do not yet establish a maintained version range, supported OS
matrix per client, clean-install behavior, configuration-parser conformance, or
safe automated removal. Codex has deterministic direct-tool and packet
evidence; Claude Code has one real-client, model-directed lifecycle record;
Cursor has authenticated temporary-configuration discovery but no approved
tool lifecycle. Gemini CLI, GitHub Copilot CLI, and VS Code Copilot have
read-only preadmission guides and validators only. The [compatibility
matrix](compatibility-matrix.md) therefore keeps all three in the **Generic
local MCP** category.

Source references for the host-side configuration surfaces: [OpenAI Codex MCP
documentation](https://learn.chatgpt.com/docs/extend/mcp), [OpenAI Codex
configuration basics](https://learn.chatgpt.com/docs/config-file/config-basic),
[Claude Code MCP documentation](https://code.claude.com/docs/en/mcp),
[Cursor MCP documentation](https://docs.cursor.com/context/model-context-protocol),
[Gemini CLI MCP documentation](https://github.com/google-gemini/gemini-cli/blob/main/docs/tools/mcp-server.md),
[GitHub Copilot CLI MCP documentation](https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/add-mcp-servers),
and [VS Code MCP documentation](https://code.visualstudio.com/docs/agent-customization/mcp-servers).
