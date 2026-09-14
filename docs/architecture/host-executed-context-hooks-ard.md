# Host-Executed Context Hooks — Architecture Requirements and Design

- ARD ID/version: IC-HEH-ARD-126 / 1.2.
- Status: Accepted for implementation.
- Date: 2026-09-14. Version 1.1 was dated 2026-09-13, and 1.0 2026-09-03.
- Governing PRD: [IC-HEH-126](../product/host-executed-context-hooks-prd.md).
- Decisions:
  [ADR-0126](../decisions/0126-answer-host-executed-operations-without-execution-authority.md),
  for the host entry points (1.1),
  [ADR-0162](../decisions/0162-serve-output-reduction-to-host-hooks-over-standard-input.md),
  and for terminal escape sequences (1.2),
  [ADR-0164](../decisions/0164-remove-terminal-escape-sequences-when-reducing-tool-output.md).

## The distinction this design rests on

```text
                 who performs the operation?
                             │
        ┌────────────────────┴────────────────────┐
        │                                         │
   the product                               the host
        │                                         │
  control plane                             evidence plane
  (execution authority)                     (no authority)
        │                                         │
   not this design                          this design
```

A context layer that runs commands must be trusted with execution. A context
layer that only *answers about* operations the host performs need not be. The
second is strictly weaker in capability and strictly stronger in deployability,
and it is the position Impresari already asserts in `SEC-INV-007`.

Hooks are the mechanism that lets a no-authority product still displace work.

## Data flow

```text
host decides to read / search                 host has already executed
        │                                              │
        ▼                                              ▼
   offer request                                  offer output bytes
        │                                              │
        ▼                                              ▼
   ┌──────────────────────────────────────────────────────────┐
   │ hook process: env_clear, bounded stdio, deadline,         │
   │ no spawn, no socket, no workspace write                   │
   └──────────────────────────────────────────────────────────┘
        │                                              │
        ▼                                              ▼
  spans + hashes + omissions              selected bytes + omissions
        │                                              │
        └──────────────► host decides whether to use ◀──┘
```

The arrow never reverses. Impresari receives, transforms, and returns. It does
not call back, request, instruct, or block.

## Trust zone

Hooks introduce **Z8 — host hook channel**, adjacent to Z4 (consumer interface)
and inheriting its rules. Payloads arriving on Z8 carry the highest sensitivity
of their contents and are classified as untrusted repository-derived content on
entry, exactly as workspace bytes are in Z2.

The critical property: Z8 is an *inbound data* zone, not a capability zone.
Nothing arriving on it can grant, widen, or redirect authority. A payload that
contains the text of a policy, a tool schema, an instruction, or a path is still
only bytes to be selected from.

## Why substitution is safe here but addition was not

Read substitution returns spans recovered from the already-authorized workspace
snapshot, each with an independently computed content hash and byte range, under
`SEC-INV-011`. The host can verify every returned byte against the source it
already controls. A hook therefore cannot smuggle content: anything it returns
that does not verify is rejected by the host, not by trust.

Output reduction is constrained further. The response must be a selection from
bytes the host supplied in the same exchange. Impresari cannot introduce a byte
the host did not already have, which removes the injection surface entirely for
that shape.

From exchange 1.1 (ADR-0164), a returned line may lose its terminal escape
sequences, and nothing else. Every returned byte is still one the host
supplied, in the order supplied. A host can check this by removing escape
sequences from its own lines by the same rule.

## Accounting

Every response declares the substitution it performed: bytes offered, bytes
returned, and what was omitted and why. From exchange 1.1 it also declares the
escape bytes removed from the returned lines. This exists so a measurement can
attribute savings honestly rather than inferring them, and so the governing
objective's rule — treatment must not read more than baseline — is checkable
from the record rather than from belief.

## Host entry points (1.1)

ADR-0162 opens output reduction to hosts through two commands of the
`impresari-context` binary.
- Each reads one bounded payload from standard input and writes at most one
  JSON line to standard output.
- Both are matched on the raw arguments before global options are parsed. So
  neither reaches the engine, a workspace, the clock, or an identifier seed.

| Command | Reads | Writes | Exit |
| --- | --- | --- | --- |
| `hook output-reduction` | One exchange request, at most 12 MiB | The response, or `{schema_name, schema_version, error}` with a closed category | 0, or 1 on a closed failure; 74 if standard output fails |
| `hook claude-code post-tool-use` | Claude Code's `PostToolUse` payload, at most 16 MiB | A replacement, or nothing | Always 0 |

- Input past a ceiling is drained rather than left in the pipe, and is treated
  as malformed.
- Nothing is read from the environment. The recipe that wires a hook, which
  follows separately, runs the command with an empty environment.

### Claude Code adapter

`context_claude_code::output_hook` maps Claude Code's payload onto
`context_engine::host_hooks::reduce_host_text`.
- `reduce_host_text` applies the ADR-0160 rule and bounds without the base64
  envelope. From 1.2 it also applies the ADR-0164 escape removal.
- `reduce_host_output` calls it too, so both paths select the same lines.
- A stream kept whole goes through `remove_terminal_escapes` instead (1.2).

```text
PostToolUse payload: a Bash result that finished, not interrupted,
not in the background, not an image?
        │ no ──► print nothing
        ▼ yes
stdout + stderr > 8 KiB? ── no ──► print nothing
        │ yes
        ▼
split the budget: a stream within half of it is kept whole,
the other gets the rest; select from each with 2 context lines;
remove terminal escape sequences from both (1.2)
        │
        ▼
smaller than offered? ── no ──► print nothing
        │ yes
        ▼
updatedToolOutput = tool_response with stdout and stderr replaced
additionalContext = fixed note with byte and line counts, and the
                    escape bytes removed when there were any
```

- **Why the replacement is the payload's own result object:** Claude Code
  checks `updatedToolOutput` against the tool's output schema and keeps the
  original output on a mismatch.
- **What enters the output:** the note is fixed text and numbers. No payload
  byte enters it, and no byte the host did not supply enters the output.
- **Which commands it reaches:** Claude Code routes a command that exits
  non-zero to `PostToolUseFailure`, which cannot replace output. The adapter
  therefore reaches only commands that succeed.

## Preserved invariants

`SEC-INV-002` (never writes to the source workspace), `SEC-INV-003` (repository
content is data), `SEC-INV-007` (no execution, network, or telemetry),
`SEC-INV-008` (source excluded from logs), `SEC-INV-009` (budgets everywhere),
`SEC-INV-011` (no exact-source authority without hash and span), and
`SEC-INV-012` (consumer input cannot broaden capability) all hold unchanged.

This design deliberately adds an integration surface without adding a capability.
If a future increment gives a hook the ability to perform the operation itself,
that is a different product and requires a new decision record.

## Deferred work

Per-client adapters map their own hook formats onto this channel in the
existing thin-adapter crates. No client gets a forked core. Still to come:

- **A Claude Code recipe** that wires `hook claude-code post-tool-use` to
  `Bash` calls of test and build runners. It waits until the ADR-0159 recipe
  check can hold a `PostToolUse` recipe to that shape.
- **Failing commands in Claude Code.** These need a `PreToolUse` command
  rewrite, and that needs its own decision.
- **The other clients**, each with a different hook model:
  - Copilot CLI's `postToolUse` can replace a result, through
    `modifiedResult`.
  - Codex can replace one only by blocking with feedback, and it caps hook
    output near 2,500 tokens.
  - Cursor and VS Code cannot replace shell output. They can only rewrite a
    command before it runs.
