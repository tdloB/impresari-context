# Impresari Context hook recipes

Opt-in recipes for clients that run project hooks. Impresari never installs
them, and nothing in them changes a client's permissions, approvals, or
configuration beyond what you copy by hand. Each uses only a hook event that
adds text to the model's context; none can block, approve, deny, or rewrite a
tool call.

## Claude Code: after compaction

When Claude Code compacts a conversation it replaces earlier turns with a
summary, and excerpts, maps, handles, and packet IDs that Impresari returned
may drop out of the model's context. This recipe adds a short note after each
compaction saying so, and that `context_build` or a direct file read is still
available.

| File | What it is |
| --- | --- |
| `claude/impresari-context-after-compaction.sh` | The hook. Prints a fixed note and exits 0. Reads no file, opens no connection, writes nothing. |
| `claude/settings.fragment.json` | The `SessionStart` entry, matched only to `compact`, with a 5-second timeout. |
| `claude/SHA256SUMS` | The script's SHA-256. |

### Install, by hand, in one project

1. Check the script against its published checksum from `templates/hooks`:
   `shasum -a 256 -c claude/SHA256SUMS`.
2. Copy `claude/impresari-context-after-compaction.sh` to the project's
   `.claude/hooks/` directory and make it executable.
3. Merge `claude/settings.fragment.json` into the project's
   `.claude/settings.json`, keeping any hooks already there. Use the project
   file only; the recipe is not meant for user-wide settings.
4. Review the hook in Claude Code's `/hooks` view before relying on it.

### Remove

Delete the `SessionStart` entry you added from `.claude/settings.json`, then
delete `.claude/hooks/impresari-context-after-compaction.sh`.

### What it does not do

It does not run on every prompt, read the repository, call the Impresari
server, or decide anything. A hook that points at a task's files on every
prompt would need a nomination path fast enough for a hook's time limit, and
would add context rather than replace reads; it is deferred until one exists
and is measured.
