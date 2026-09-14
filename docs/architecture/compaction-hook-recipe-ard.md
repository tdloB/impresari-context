# Compaction Hook Recipe — Architecture Requirements and Design

- ARD ID/version: IC-CHR-ARD-159 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-13.
- Governing PRD: [IC-CHR-159](../product/compaction-hook-recipe-prd.md).
- Decision: [ADR-0159](../decisions/0159-offer-a-hook-recipe-for-evidence-lost-to-compaction.md).

## Artifacts

`templates/hooks/` holds the recipe and its README:

- `claude/impresari-context-after-compaction.sh`: POSIX `sh`. It drains its
  input, prints a fixed note with a here-document, and exits 0. It carries the
  exact ownership marker the client-guidance templates use.
- `claude/settings.fragment.json`: one `SessionStart` entry with matcher
  `compact` and a `command` hook running the script from
  `"$CLAUDE_PROJECT_DIR"/.claude/hooks/`, with a 5-second timeout.
- `claude/SHA256SUMS`: the script's SHA-256, in `shasum -a 256 -c` format.

Claude Code adds a `SessionStart` hook's standard output to the model's context,
and a timeout or failure never blocks the session. The script prints plain text
rather than JSON so its output needs no schema.

## Gate check

`scripts/check-hook-recipes.rb`, run by `scripts/check.sh`, requires:

- the script to be at most 4,096 bytes, carry its ownership marker, and end with
  `exit 0`;
- its non-comment text to contain no blocking field, non-zero exit, URL, other
  program, or path into user-wide settings;
- `SHA256SUMS` to match the script;
- every event in the fragment to be `SessionStart` or `UserPromptSubmit`, every
  `SessionStart` matcher to be a documented source, every hook to be a
  `command` under the project's `.claude/hooks/`, with a timeout from 1 to 30
  seconds.

## Verification

Each rule is exercised by editing the recipe to break it and confirming the
check fails, then restoring it.
