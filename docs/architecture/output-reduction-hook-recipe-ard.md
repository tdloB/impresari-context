# Output Reduction Hook Recipe — Architecture Requirements and Design

- ARD ID/version: IC-ORR-ARD-163 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-13.
- Governing PRD: [IC-ORR-163](../product/output-reduction-hook-recipe-prd.md).
- Decision: [ADR-0163](../decisions/0163-offer-a-hook-recipe-that-shortens-successful-test-and-build-output.md).

## Artifacts

`templates/hooks/claude/` holds this recipe beside the compaction recipe:

- `impresari-context-reduce-output.sh`: POSIX `sh`, with the exact ownership
  marker. Its only commands are these two lines, then `exit 0`:

  ```sh
  bin=$(command -v impresari-context) || { cat >/dev/null; exit 0; }
  env -i "$bin" hook claude-code post-tool-use || true
  ```

- `reduce-output.settings.fragment.json`: one `PostToolUse` entry with matcher
  `Bash`, holding 11 `command` hooks. Each has an `if` rule naming one runner,
  runs the script from `"$CLAUDE_PROJECT_DIR"/.claude/hooks/`, and has a
  10-second timeout.
- `SHA256SUMS`: the SHA-256 of both recipe scripts.

How the pieces behave in Claude Code:
- Claude Code passes the finished call's payload on standard input.
- Whatever the command prints becomes the hook's output. That is either
  nothing, or the `updatedToolOutput` replacement and `additionalContext` note
  that ADR-0162 describes.
- A hook that times out or fails leaves the original output in place.

## Gate check

`scripts/check-hook-recipes.rb` holds every recipe to its own entry in a fixed
table. Shared rules for every recipe:

- the script is at most 4,096 bytes, carries its ownership marker, and ends
  with `exit 0`;
- its non-comment text contains no blocking field, non-zero exit, URL, other
  program, or path into user-wide settings;
- `SHA256SUMS` matches the script and lists only recipe scripts;
- every hook is a `command` that runs the recipe's own script from the
  project's `.claude/hooks/`, with a timeout from 1 to 30 seconds.

This recipe's own rules:

- **Fixed lines.** Each of the two fixed lines appears exactly once. They are
  set aside before the forbidden-content scan, so no other line may name
  `impresari-context`.
- **Event and matcher.** The only event is `PostToolUse`, and its matcher is
  `Bash`.
- **Runner filter.** Every hook has an `if` rule of the form
  `Bash(<runner> *)`, where the runner is lower-case letters, digits, spaces,
  dots, underscores and hyphens.

## Verification

Each rule is exercised by editing the recipe to break it, confirming the check
fails with that rule's message, and then restoring the recipe.
