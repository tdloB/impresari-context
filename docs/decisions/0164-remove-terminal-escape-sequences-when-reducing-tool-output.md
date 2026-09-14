# ADR-0164: Remove Terminal Escape Sequences When Reducing Tool Output

- Status: Accepted
- Date: 2026-09-14
- Related PRD: [Output Reduction Selection](../product/output-reduction-selection-prd.md)
- Architecture: [Output Reduction Selection](../architecture/output-reduction-selection-ard.md)
- Amends: [ADR-0160](0160-keep-the-verdict-and-the-failure-when-reducing-tool-output.md)
  decision 3, and [ADR-0162](0162-serve-output-reduction-to-host-hooks-over-standard-input.md)
  decisions 1 and 2
- Follows: [ADR-0126](0126-answer-host-executed-operations-without-execution-authority.md)

## Context

Tools that write to a terminal wrap their text in escape sequences for color,
bold, cursor movement and hyperlinks. Many do so even when their output is
captured, because a project forces it. astropy 5.1 and later set
`addopts = --color=yes` for pytest.

The benchmark harness ran this reducer on the real test output of 22
SWE-bench tasks, in its provider-free base run of 2026-09-14
(repository-context-eval ADR-0083). Ten of those tasks print colored pytest
output. On them, the ADR-0160 rules are partly blind:
- pytest's `E` detail starts with an escape sequence rather than `E`, so it is
  not seen as a failure;
- the result line's counts follow escape sequences, so it is not seen as a
  verdict;
- a location's file name ends in a reset code, so it is not seen as a location.

Failure words and names ending in `Error` inside a colored line still count.

The codes also cost the model. Claude Code 2.1.266 passes them through: a red
word printed by a `Bash` command reaches the model as `[31mred[0m`. In the
harness, whose tool result is JSON, escape characters made the colored tasks'
results a median 26% larger than the text they carried.

## Decision

1. **Remove escape sequences from each offered line before anything else looks
   at it.** The output is split into lines as before. Each line then loses its
   terminal escape sequences. Classification, the byte budget and the returned
   text all use what remains.
2. **What counts as an escape sequence.** It starts at the escape character
   (`ESC`, 0x1B) and follows ECMA-48 in its 7-bit form:
   - **A control sequence:** `ESC [`, then parameter bytes 0x30–0x3F, then
     intermediate bytes 0x20–0x2F, then one final byte 0x40–0x7E. This covers
     color, bold, erasing and cursor movement.
   - **A string control:** `ESC` followed by `]`, `P`, `X`, `^` or `_`, then
     everything up to and including a bell (0x07) or `ESC \`, on the same
     line. This covers hyperlinks and window titles. A hyperlink's visible
     text sits between its two sequences and is kept.
   - **An escape with intermediate bytes:** `ESC`, then bytes 0x20–0x2F, then
     one final byte 0x30–0x7E, such as `ESC ( B`.
   - **A two-byte escape:** `ESC` and one byte 0x30–0x7E, such as `ESC 7`.

   An escape character that opens none of these on the same line is removed
   on its own, and the bytes after it are kept. That covers an escape at the
   end of a line, one followed by any other byte, and an unfinished control
   sequence or string control.
3. **Nothing else in a line changes.**
   - Other control characters, such as a carriage return or a backspace, are
     kept.
   - A line without an escape character is returned byte for byte.
   - The response is still whole offered lines in their original order, and
     every returned byte is one the host supplied, in the order supplied.
     Reduction still cannot introduce a byte.
4. **The exchange becomes 1.1.**
   - The request is unchanged apart from its version.
   - The response adds `escape_bytes_removed`: the escape bytes removed from
     the returned lines.
   - A host checks a response by removing escape sequences from its own lines
     by the same rule, then comparing lines and summing the difference.
   - A 1.0 request fails closed with `unsupported_schema`.
5. **The Claude Code adapter removes them too.**
   - A stream kept whole loses its escape sequences, and a selected stream is
     selected as above.
   - When any were removed, the note says how many bytes.
   - The adapter still acts only on output over 8 KiB, as ADR-0162 set.

## Consequences

- Colored output is classified by its text. pytest's detail, locations and
  result line are recognized as they are in uncolored output. A re-run of the
  harness's base run will measure this on the ten colored tasks.
- The model never receives escape codes from reduction, which saves their
  bytes and tokens.
- For output without an escape character, selection and returned bytes are
  unchanged, and every earlier test passes unchanged.
- A host that checked 1.0 responses by exact line match must now match against
  its lines with escape sequences removed. The benchmark harness does this
  (repository-context-eval ADR-0084).
- Meaning carried only by color is lost.
  - Git's word-level color diffs mark changes only by color.
  - Some test runners highlight the differing characters inside expected and
    received lines; the lines themselves are kept.
  - The runners this reducer targets print their facts as text, for logs and
    pipes. A tool whose meaning lives only in color needs its own decision.
- Removal is linear. A scan that starts at an escape character stops at the
  next one, at a newline, or at its terminator.
- A progress bar redrawn with carriage returns keeps every frame, as before.

## Alternatives considered

**Keep the codes and make each rule look past them.** Rejected. Every rule
would have to skip codes the same way, which is removal by another name, and
the model would still pay for codes it cannot see as color.

**Ask tools not to use color, with `NO_COLOR` or `--color=no`.** Rejected. A
hook sees output after the command ran, the harness must run a task's command
unchanged, and a project can force color in its own settings.

**Remove only color codes.** Rejected. Erase and cursor codes, hyperlinks and
window titles are just as unreadable to a model, and one grammar covers them
all.

**Also clean output within 8 KiB in the Claude Code adapter.** Rejected for
now. A replacement carries a note of about 200 bytes, which can cost more than
the codes it removes from short output.
