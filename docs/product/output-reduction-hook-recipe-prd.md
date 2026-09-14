# Output Reduction Hook Recipe PRD

## Document Control

- PRD ID/version: IC-ORR-163 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-13.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Output Reduction Hook Recipe ARD](../architecture/output-reduction-hook-recipe-ard.md).
- Governing decision:
  [ADR-0163](../decisions/0163-offer-a-hook-recipe-that-shortens-successful-test-and-build-output.md).
- Follows: [ADR-0162](../decisions/0162-serve-output-reduction-to-host-hooks-over-standard-input.md).

## Problem

Claude Code hands the model the whole output of a passing test run or build.
Much of that output is noise, such as progress lines and one line per passing
test, and the whole of it is sent again with every later request.
`impresari-context hook claude-code post-tool-use` can shorten it, but nothing
connects it to Claude Code.

## Product Outcome

A user can copy one recipe into a project so that a passing test run or build
reaches the model as at most 8 KiB of its own lines, with a note saying so. No
other command is affected.

## Functional Requirements

1. The recipe uses only `PostToolUse` with matcher `Bash`. Every entry names
   one test or build runner in Claude Code's `if` rule.
2. Its script runs only `impresari-context hook claude-code post-tool-use`,
   under an empty environment. When `impresari-context` is absent, it drains
   its input.
3. The script always exits 0, reads no file, opens no connection, and writes
   nothing.
4. The script's SHA-256 is published beside it, and the project-scoped command
   has a timeout of at most 30 seconds.
5. Impresari installs nothing. The user copies the recipe and can remove it by
   deleting what they added.

## Acceptance Criteria

- `scripts/check-hook-recipes.rb` passes, and fails when any of these is true:
  - an entry lacks a one-runner `if` rule;
  - the matcher is not `Bash`;
  - the recipe uses another event;
  - the fragment runs another script;
  - the script calls Impresari any other way;
  - the checksum does not match.
- The compaction recipe's rules are unchanged apart from its fragment's name.
- The full repository gate passes.

## Non-Goals

- Shortening failing commands. Claude Code gives them only
  `PostToolUseFailure`.
- Rewriting or running a command.
- Other clients' hook formats.
