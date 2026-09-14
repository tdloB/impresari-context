# ADR-0163: Offer a Hook Recipe That Shortens Successful Test and Build Output

- Status: Accepted
- Date: 2026-09-13
- Related PRD: [Output Reduction Hook Recipe](../product/output-reduction-hook-recipe-prd.md)
- Architecture: [Output Reduction Hook Recipe](../architecture/output-reduction-hook-recipe-ard.md)
- Follows: [ADR-0159](0159-offer-a-hook-recipe-for-evidence-lost-to-compaction.md),
  [ADR-0162](0162-serve-output-reduction-to-host-hooks-over-standard-input.md)

## Context

ADR-0162 added `impresari-context hook claude-code post-tool-use`. It shortens a
finished Claude Code `Bash` result to at most 8 KiB of its own lines. On the
log of a real passing `./scripts/check.sh` run, it returned 8,191 of 115,287
bytes and ended with the run's last result lines. Nothing wires it into
Claude Code yet.

Three facts shape the recipe:
- **Only successful commands reach it.** Claude Code fires `PostToolUse` only
  for a command that exits 0. A failing command goes to `PostToolUseFailure`,
  which cannot replace output.
- **The reducer suits only some output.** It is built for build and test
  output (ADR-0160). On `cat`, `grep` or `git diff` it would drop lines the
  agent asked to read.
- **The existing check rules it out.** ADR-0159's recipe check admits only
  add-context events and forbids calling `impresari-context`. That is right
  for the compaction note, and it rules this recipe out.

## Decision

1. **Ship a second opt-in recipe.** It is a `PostToolUse` hook matched to
   `Bash`, with one entry per test or build runner.
   - Each entry is filtered by Claude Code's `if` rule, such as
     `Bash(pytest *)`, so no other command reaches it.
   - Each runs a script that hands the payload to `impresari-context hook
     claude-code post-tool-use` with an empty environment.
2. **Runners.** The fragment names 11 runners: `pytest`, `python -m pytest`,
   `cargo test`, `cargo build`, `go test`, `go build`, `npm test`, `npx jest`,
   `npx vitest`, `npx tsc` and `mypy`. These are tools the ADR-0160
   measurement covered. The user adds or drops entries by hand.
3. **The script.**
   - If `impresari-context` is on `PATH`, the script runs it under `env -i`.
     Otherwise it drains its input and exits 0.
   - It always exits 0. It reads no file, opens no connection, and writes
     nothing.
4. **The check.** The recipe check now holds each recipe to its own events and
   script.
   - This recipe may use only `PostToolUse`, with matcher `Bash`, and every
     entry must name one runner in its `if` rule.
   - Its script may call Impresari only through two fixed lines.
   - The compaction recipe keeps its add-context-only rule.
   - The compaction fragment is renamed
     `after-compaction.settings.fragment.json`, so each fragment is named for
     its recipe.
5. **No installation.** Impresari installs nothing, as ADR-0159 decided.

## Consequences

- A passing test run or build in Claude Code costs at most about 8 KiB, plus a
  note of about 200 bytes, instead of its full output.
- Failing runs keep their full output, and they carry most test-output tokens.
  Shortening them needs a `PreToolUse` command rewrite, which is its own
  decision.
- Claude Code decides whether a compound command such as `cd src && pytest`
  matches an `if` rule. One that does not match keeps its full output.
- The recipe needs an `impresari-context` with the `hook` command on `PATH`.
  Without it, the recipe does nothing.
- No Claude Code session has run it, because that would spend provider
  tokens. The gate check proves the recipe's shape, not its effect on an agent.

## Alternatives considered

**Match every `Bash` call and let the adapter choose by command text.**
Rejected. Command text in a payload cannot alter selection authority
(IC-HEH-126 FR-5). It would also shorten `cat` and `git diff`.

**One entry listing every runner.** Not possible. Claude Code's `if` field
takes one permission rule per entry.

**Install the hook from the CLI.** Rejected for the reason in ADR-0159: a
client's configuration stays with the user.
