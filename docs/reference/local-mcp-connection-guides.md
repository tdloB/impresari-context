# Local MCP Connection Guides

- Version: 1.5
- Published: 2026-08-26
- Classification: Local MCP guidance; the compatibility matrix is authoritative
- Supported transport: local stdio only

These guides render the fixed-launch contract for six common coding clients.
Codex, Claude Code, Cursor, and GitHub Copilot CLI are first-class only for
their recorded client/version/OS scopes; the other named client guides remain
generic unless their matrix row says otherwise.
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

Codex supports local stdio MCP servers through its active **user-level Codex
home** configuration (`$CODEX_HOME/config.toml`, or the default Codex home).
On the recorded Codex CLI/App Server version, a `.codex/config.toml` file is
not a runtime MCP configuration source—even after the containing folder is
trusted. Do not put an Impresari MCP entry in a repository file expecting
Codex to load it.

Use Codex's supported command to make a reviewed user-level registration:

```text
codex mcp add impresari-context -- \
  /absolute/path/to/impresari-context-mcp \
  --workspace /absolute/path/to/source-workspace \
  --cache /absolute/path/to/separate-cache \
  --consumer-id consumer_codex_local \
  --role local_user
```

Verify only that named entry with `codex mcp get impresari-context`. Remove
only that same entry with `codex mcp remove impresari-context`; do not delete
the complete user configuration or unrelated servers.

The versioned Impresari kit can render, inspect, validate, preview, install,
update, or remove an **explicitly named** user-level TOML target. It never
discovers a default client home, changes project trust, signs in, or grants
MCP approval. Validate a reviewed target without launching Codex:

```text
impresari-context doctor codex-config \
  /absolute/path/to/source-workspace \
  /absolute/path/to/separate-cache \
  /absolute/path/to/CODEX_HOME/config.toml
```

The maintained disposable admission rehearsal uses an empty, explicit
`CODEX_HOME` under `/private/tmp`; it applies and removes the kit's exact
entry there, then proves the direct App Server lifecycle and packet
equivalence. It never writes the user's actual Codex configuration or creates
project trust.

This managed user-configuration surface has been locally checked against
Codex CLI `0.149.0-alpha.4.1` on macOS aarch64. Its exact evidence and
recorded-scope limits are in the [Phase 1 Codex kit record](../verification/phase-1-codex-connection-kit.md).

## Claude Code

Claude Code documents local stdio registration through `claude mcp add`. Its
current CLI defaults to local scope when no `--scope` is supplied; the
following is therefore a user-invoked local-scope example:

```text
claude mcp add --scope local --transport stdio impresari-context -- \
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

The managed connection has been exercised through Claude Code CLI `2.1.241` on
macOS aarch64 in two separate disposable paths: a strict one-run
`--mcp-config` lifecycle with direct packet equivalence and a native
`claude mcp add/get/remove --scope local` lifecycle rooted in an explicit empty
`/private/tmp` Claude home. The latter checks only the exact named entry and
removes it again; neither path reads or changes the user's actual Claude
configuration. Its evidence and recorded-scope limits are in the [Phase 1
Claude Code kit record](../verification/phase-1-claude-code-connection-kit.md).

For an independently reviewed local-scope rehearsal, first preview a
disposable Claude home, workspace, and cache:

```text
ruby scripts/rehearse-claude-native-local-scope.rb \
  --prepare-root /private/tmp/impresari-claude-l1-admission
# inspect the preview, then rerun the same command with --apply;
# finally run it with --temporary-root using that exact root
```

The normal product path remains the explicit user-invoked command above. The
rehearsal never targets a default home or turns a project-shared `.mcp.json`
into an implicit configuration target.

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
        "--workspace", "/absolute/path/to/source-workspace",
        "--cache", "/absolute/path/to/separate-cache",
        "--consumer-id", "consumer_cursor_local",
        "--role", "local_user"
      ]
    }
  }
}
```

The cache must be an existing directory outside the workspace; never use a
workspace child such as `.cache`. The documented Cursor stdio form infers its local transport from
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
its available tools. Enable only the reviewed named entry with
`cursor agent mcp enable impresari-context`; disable only that same entry with
`cursor agent mcp disable impresari-context` before removing it. These commands
change Cursor's local approval state and must be explicit user actions.

The managed project connection has been checked against Cursor Agent CLI
`3.17.8` (`2026.08.11-e8db854`) on macOS aarch64: malformed configuration
fails closed; native enable/list-tools/disable works; the bounded four-tool
Agent-mode lifecycle returns a packet identical to a direct MCP control packet;
and the owned project configuration is removed exactly. The recorded-scope
limits are in the [Phase 1 Cursor kit record](../verification/phase-1-cursor-connection-kit.md).

For an independently reviewed isolated rehearsal, first create a disposable
project outside any repository:

```text
ruby scripts/rehearse-cursor-preadmission.rb \
  --prepare-project-root /private/tmp/impresari-cursor-l1-admission
# inspect the preview, then rerun the same command with --apply
```

The preparation command creates only the reported `workspace` and separate
`cache` directories under `/private/tmp`; it neither creates `.cursor/mcp.json`
nor enables an MCP server. The native rehearsal is separately preview-first
and performs exact enable/list-tools/disable/removal only in its named
disposable project; it never targets a default Cursor configuration or a user
source project.

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

The managed project connection has been checked against GitHub Copilot CLI
`1.0.80` on macOS aarch64: malformed configuration fails closed; the native
project entry is discovered through `copilot mcp list/get`; a bounded four-tool
prompt lifecycle returns a packet identical to a direct MCP control packet; and
only the owned project entry plus a temporary workspace-trust value are removed.
The recorded-scope limits are in the [Phase 2 Copilot CLI kit
record](../verification/phase-2-copilot-cli-connection-kit.md).

For an independently reviewed isolated rehearsal, first preview a disposable
Copilot home, project, and cache:

```text
ruby scripts/rehearse-copilot-native-project.rb \
  --prepare-root /private/tmp/impresari-copilot-l1-admission
# inspect the preview, then rerun the same command with --apply;
# finally run it with --temporary-root using that exact root
```

The rehearsal uses an isolated `COPILOT_HOME` and creates a temporary trust
entry only for its named empty workspace. It uses no additional MCP
configuration, removes only its owned project entry and that exact trust value,
and never reads or writes a user's Copilot home. The normal product path
remains the reviewed project entry above; user folder trust remains explicit.

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

These guides alone do not establish a maintained version range, supported OS
matrix, clean-install behavior, configuration-parser conformance, or safe
automated removal. The [compatibility matrix](compatibility-matrix.md) is the
authoritative classification: Codex, Claude Code, Cursor, and GitHub Copilot
CLI are First-class only for their individually recorded client/version/OS
scopes. Gemini CLI and VS Code Copilot have read-only preadmission guides and
validators only and remain **Generic local MCP**.

Source references for the host-side configuration surfaces: [OpenAI Codex MCP
documentation](https://learn.chatgpt.com/docs/extend/mcp), [OpenAI Codex
configuration basics](https://learn.chatgpt.com/docs/config-file/config-basic),
[Claude Code MCP documentation](https://code.claude.com/docs/en/mcp),
[Cursor MCP documentation](https://docs.cursor.com/context/model-context-protocol),
[Gemini CLI MCP documentation](https://github.com/google-gemini/gemini-cli/blob/main/docs/tools/mcp-server.md),
[GitHub Copilot CLI MCP documentation](https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/add-mcp-servers),
and [VS Code MCP documentation](https://code.visualstudio.com/docs/agent-customization/mcp-servers).
