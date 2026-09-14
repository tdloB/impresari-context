# Impresari Context hook recipes

Opt-in recipes for clients that run project hooks. Impresari never installs
them, and nothing in them changes a client's permissions, approvals, or
configuration beyond what you copy by hand. None can block, approve, deny, or
rewrite a tool call:

- the compaction recipe only adds text to the model's context;
- the output recipe replaces a finished command's output with a selection of
  that output's own lines, in order, and adds a note saying so.

## Claude Code: after compaction

When Claude Code compacts a conversation it replaces earlier turns with a
summary, and excerpts, maps, handles, and packet IDs that Impresari returned
may drop out of the model's context. This recipe adds a short note after each
compaction saying so, and that `context_build` or a direct file read is still
available.

| File | What it is |
| --- | --- |
| `claude/impresari-context-after-compaction.sh` | The hook. Prints a fixed note and exits 0. Reads no file, opens no connection, writes nothing. |
| `claude/after-compaction.settings.fragment.json` | The `SessionStart` entry, matched only to `compact`, with a 5-second timeout. |
| `claude/SHA256SUMS` | The SHA-256 of both recipe scripts. |

### Install, by hand, in one project

1. Check the script against its published checksum from `templates/hooks`:
   `shasum -a 256 -c claude/SHA256SUMS`.
2. Copy `claude/impresari-context-after-compaction.sh` to the project's
   `.claude/hooks/` directory and make it executable.
3. Merge `claude/after-compaction.settings.fragment.json` into the project's
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

## Claude Code: shorter output from passing test runs and builds

When a test run or build succeeds, Claude Code normally hands its whole output
to the model. This recipe replaces that output with at most 8 KiB of its own
lines, in their original order and without terminal color codes: the result,
warnings, source locations and the end of the output come first. A note tells the model how much was kept and how
to see the rest. It needs `impresari-context` with the `hook` command on
`PATH`; without it, the recipe does nothing.

| File | What it is |
| --- | --- |
| `claude/impresari-context-reduce-output.sh` | The hook. Hands Claude Code's payload to `impresari-context hook claude-code post-tool-use` with an empty environment, and exits 0. |
| `claude/reduce-output.settings.fragment.json` | `PostToolUse` entries for `Bash`, one per test or build runner, each with a 10-second timeout. |
| `claude/SHA256SUMS` | The SHA-256 of both recipe scripts. |

### Install, by hand, in one project

1. Check the script against its published checksum from `templates/hooks`:
   `shasum -a 256 -c claude/SHA256SUMS`.
2. Copy `claude/impresari-context-reduce-output.sh` to the project's
   `.claude/hooks/` directory and make it executable.
3. Merge `claude/reduce-output.settings.fragment.json` into the project's
   `.claude/settings.json`, keeping any hooks already there. Drop the runners
   you do not use. To add your own, copy an entry and name the runner in the
   same `Bash(<runner> *)` form.
4. Review the hook in Claude Code's `/hooks` view before relying on it.

### Remove

Delete the `PostToolUse` entries you added from `.claude/settings.json`, then
delete `.claude/hooks/impresari-context-reduce-output.sh`.

### What it does not do

- It never shortens a failing command. Claude Code reports a command that exits
  non-zero through `PostToolUseFailure`, which cannot replace output.
- It leaves every other command alone, such as `cat`, `grep` or `git diff`,
  where every line may matter.
- It never runs or rewrites a command, and it reads no repository file.
