# ADR-0162: Serve Output Reduction to Host Hooks Over Standard Input

- Status: Accepted
- Date: 2026-09-13
- Related PRD: [Host-Executed Context Hooks](../product/host-executed-context-hooks-prd.md)
- Architecture: [Host-Executed Context Hooks](../architecture/host-executed-context-hooks-ard.md)
- Follows: [ADR-0126](0126-answer-host-executed-operations-without-execution-authority.md),
  [ADR-0160](0160-keep-the-verdict-and-the-failure-when-reducing-tool-output.md)

## Context

ADR-0126 added an exchange in which a host offers output it has already
produced and gets back a bounded selection of its lines. ADR-0160 fixed how
lines are chosen. Nothing outside the module calls `reduce_host_output`, and
no host can reach it:
- The CLI has no command for it.
- An MCP tool would be the wrong shape. The agent would have to choose to
  call it after the output was already in its context, which adds rather than
  replaces.

Two hosts need it now.
- The benchmark harness will measure output compression by running a task's
  own tests. It needs an entry point it can call as a host, with no authority
  beyond the exchange.
- Claude Code has a hook that can replace a tool's result.

Checked against Claude Code 2.1.266:
- A `PostToolUse` hook's `hookSpecificOutput.updatedToolOutput` replaces the
  result before the model sees it.
- Claude Code checks the replacement against the tool's own output schema. If
  it does not match, Claude Code keeps the original output. For `Bash` that
  schema is an object: `stdout`, `stderr`, `interrupted`, and optional fields.
- Claude Code runs sibling hooks in parallel on the original output, and the
  last replacement wins.
- A `Bash` command that exits non-zero is a tool error. Claude Code then fires
  `PostToolUseFailure`, which can only add context. A `PostToolUse` hook
  therefore sees only commands that succeed.

## Decision

1. **The exchange command.** `impresari-context hook output-reduction` reads
   one exchange request from standard input:
   - The request is schema `impresari_context_output_reduction` 1.0, at most
     12 MiB. That is the 8 MiB offered-byte ceiling in base64url, plus the
     envelope.
   - It writes one JSON line: the response and exit 0, or
     `{schema_name, schema_version, error}` with a closed category and exit 1.
   - It takes no global options. It runs before any engine, workspace, or
     clock is set up.
2. **The Claude Code command.** `impresari-context hook claude-code
   post-tool-use` reads Claude Code's `PostToolUse` payload, at most 16 MiB.
   - It acts only on a finished `Bash` result that was not interrupted, sent to
     the background, or an image, and whose `stdout` and `stderr` together
     exceed 8 KiB.
   - It then prints `updatedToolOutput`: the payload's own `tool_response`, with
     only `stdout` and `stderr` replaced by selections of their own lines.
   - It also prints an `additionalContext` note. The note states the bytes and
     lines kept, that nothing was reworded, and how to see what was left out.
   - In every other case it prints nothing. It always exits 0.
3. **Where the code lives.**
   - The Claude Code mapping lives in the Claude Code adapter crate
     (`context_claude_code::output_hook`), as IC-HEH-ARD-126's deferred work
     planned.
   - The selection stays in `context_engine::host_hooks`. The new
     `reduce_host_text` applies the same rule and bounds without the base64
     envelope, and `reduce_host_output` now calls it.
4. **The budget.** It is 8 KiB with 2 lines of context, the point ADR-0160
   measured. Standard output and standard error share the budget. A stream
   that fits in half of it is kept whole, and the other stream gets the rest.
5. **No authority.** Neither command launches a process, opens a socket, reads
   the environment, or touches a workspace. A static scan in each module and a
   child-process test with provider credentials in the environment hold this.
6. **No recipe ships here.** A Claude Code recipe that wires the command in
   comes next. It needs the ADR-0159 hook-recipe check to accept a
   `PostToolUse` recipe that matches only `Bash`, is filtered to test and
   build runners, and calls only this command.

## Consequences

- The harness can measure output compression through the product's own code
  path. Any host with a result-replacing hook can use the exchange command.
- In Claude Code, the hook reaches only commands that exit 0: passing test
  runs, builds and installs. It does not reach failing test runs, which carry
  most of the test-output tokens.
  - Reaching failing runs would need a `PreToolUse` hook that rewrites the
    command to pipe its output through the reducer.
  - That changes the command the user approves, and it depends on the shell.
    It needs its own decision.
- A shortened call costs a note of about 200 bytes. The command never returns
  an unchanged copy, so it cannot overwrite a sibling hook's replacement with
  the original.
- The model can still need a line that was dropped. The note says lines were
  dropped and how to get them back.

## Alternatives considered

**An MCP tool.** Rejected. The agent would have to call it after the output
was already in its context.

**Returning the replacement as a string.** Rejected. Claude Code keeps the
original output when a replacement does not match the tool's output schema.

**Putting the note inside the output.** Rejected. The output must stay a
selection of the host's own bytes, as IC-HEH-126 FR-4 requires.

**Letting the adapter choose which commands to shorten from the command
text.** Rejected. Command text inside a payload cannot alter selection
authority (FR-5). The host's own hook filter decides which calls are offered.
