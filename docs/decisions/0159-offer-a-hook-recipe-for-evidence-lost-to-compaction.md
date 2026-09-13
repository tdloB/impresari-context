# ADR-0159: Offer a Hook Recipe for Evidence Lost to Compaction

- Status: Accepted
- Date: 2026-09-13
- Related PRD: [Compaction Hook Recipe](../product/compaction-hook-recipe-prd.md)
- Architecture: [Compaction Hook Recipe](../architecture/compaction-hook-recipe-ard.md)
- Follows: [ADR-0154](0154-look-through-local-variables-when-building-a-map.md)

## Context

Claude Code runs project hooks on events. Some hooks can block, approve, deny,
or rewrite what an agent does. `SessionStart` and `UserPromptSubmit` can only
add text to the model's context.

When Claude Code compacts a conversation it replaces earlier turns with a
summary. Excerpts, maps, handles, and packet IDs that Impresari returned may
drop out, and nothing tells the model they are gone. It can then rely on a
handle or packet it no longer holds.

A hook that points at a task's files on every prompt would be more useful, but
it needs a nomination path fast enough for a hook's time limit. The product has
none: the identifier index is rebuilt by reading every file, and a full context
build took about 30 seconds per task on the astropy corpus. It would also add
context to every prompt rather than replace a read.

## Decision

1. Ship an opt-in recipe for Claude Code: a `SessionStart` hook matched only to
   `compact`. Its script prints a fixed note saying that earlier Impresari
   evidence may no longer be in context, and that `context_build` or a direct
   read remains available.
2. The script reads no repository file, opens no network connection, writes
   nothing, changes no setting, and always exits 0. It uses only an add-context
   event.
3. Impresari installs nothing. The user copies the script into the project's
   `.claude/hooks/` and merges the settings fragment into the project's
   `.claude/settings.json`. The recipe never points at user-wide settings.
4. The script's SHA-256 is published beside it. A gate check verifies the
   checksum, the event and matcher, the project-scoped command, a timeout of at
   most 30 seconds, and that no blocking output, network access or other
   program appears in the script.

## Consequences

The note costs a few hundred characters once after each compaction, and never
otherwise.

Offline, this can show only that the recipe is well formed and inert. Whether
agents then re-fetch evidence instead of trusting what they lost is what a
graded run measures.

A per-prompt pointer hook stays deferred until a nomination path can answer
within a hook's time limit and is measured.

## Alternatives considered

**Install the hook from the CLI.** Rejected. It would change a client's
configuration, which stays with the user.

**A startup hook that repeats the usage guidance.** Rejected. Claude Code
already shows the server's startup instructions.

**A hook on every prompt that nominates the task's files.** Deferred, for the
cost and the addition described in Context.

**A `PreToolUse` hook that substitutes reads.** Rejected. It can deny or rewrite
a tool call, which is authority the host keeps.
