# Host-Executed Context Hooks PRD

## Document Control

- PRD ID/version: IC-HEH-126 / 1.2.
- Status: Accepted for implementation.
- Date: 2026-09-14. Version 1.1 was dated 2026-09-13, and 1.0 2026-09-03.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Host-Executed Context Hooks ARD](../architecture/host-executed-context-hooks-ard.md).
- Governing decisions:
  [ADR-0126](../decisions/0126-answer-host-executed-operations-without-execution-authority.md),
  for the host entry points (1.1),
  [ADR-0162](../decisions/0162-serve-output-reduction-to-host-hooks-over-standard-input.md),
  and for terminal escape sequences (1.2),
  [ADR-0164](../decisions/0164-remove-terminal-escape-sequences-when-reducing-tool-output.md).
- Governing objective: [CLAUDE.md](../../CLAUDE.md) — substitution, never addition.

## Problem

Context is currently supplied *alongside* the agent's native repository tools,
which remain available and are still used. The product therefore adds input
instead of replacing it.

A six-arm measurement makes the cost concrete. Against an identical task, the
treatment arm performed **14** repository reads where its baseline performed
**7**, took 23 provider requests against 20, and consumed 253,502 input tokens
against 122,473. A 10,453-byte packet accounted for roughly 46% of the increase
by being re-sent on every request; the remaining 54% was the agent doing *more*
work, not less.

That is the failure the governing objective names directly: a run whose
treatment performs more repository reads than its baseline has failed, however
good the context looked.

The product cannot fix this from where it currently sits. An extra tool offered
beside an agent's existing tools can only add. Something must intercept the
operation the agent was going to perform anyway.

## Product Outcome

A host-invoked hook channel in which **the host performs every operation and
Impresari only transforms data**.

Two shapes:

- **Read substitution.** Before the host performs a repository read or search,
  it may offer the request to Impresari, which returns a compact, span-exact,
  hash-attested answer. The host decides whether to use it.
- **Output reduction.** After the host has executed something itself, it may
  offer the output to Impresari, which returns a bounded selection of the parts
  that matter, with omissions recorded.

Impresari gains no new authority in either shape. It executes nothing, opens no
socket, and cannot compel the host to do anything.

## Functional Requirements

1. Impresari executes no new process. `SEC-INV-007` is unchanged: only the
   pinned structural worker may be launched, and a hook launches nothing.
2. The hook transport is bounded newline-delimited JSON over stdin and stdout,
   with an empty environment, explicit byte ceilings, and a deadline.
3. A read-substitution response contains only spans recovered from the
   authorized workspace snapshot, each carrying a content hash and byte range.
   It never synthesizes content and never returns bytes it did not read from
   the admitted source.
4. An output-reduction response selects from the bytes the host supplied. It
   never adds, rewrites, or paraphrases them, and records what it dropped.
   (1.2) It also removes terminal escape sequences from the lines it keeps, and
   records how many bytes it removed (ADR-0164).
5. Hook payloads are untrusted data. Repository content, tool output, command
   text, and paths inside a payload cannot alter policy, capability, budget, or
   selection authority. `SEC-INV-003` and `SEC-INV-012` apply unchanged.
6. The host is the decision-maker. Impresari expresses no veto, no approval, and
   no instruction; a response is an offer the host may discard.
7. Every response declares what it substituted and what it omitted, so a host
   can account for the exchange honestly and a measurement can attribute
   savings.
8. Failure is closed and static. A malformed, oversized, or undecidable payload
   yields a closed category and no partial content.
9. The hook adds no persistence. It writes nothing to the source workspace and
   retains no cross-invocation memory.
10. A host reaches output reduction through `impresari-context hook
    output-reduction`.
    - One exchange request goes in on standard input.
    - One response, or one closed failure line, comes out on standard output.
    - The command takes no global options and needs no workspace.
11. A Claude Code adapter maps a finished `Bash` call's `PostToolUse` payload
    onto the same rule.
    - It returns the tool's own result with only `stdout` and `stderr`
      shortened.
    - It adds a note stating how much was kept and that nothing was reworded.
      (1.2) When it removed terminal escape sequences, the note also says how
      many bytes.
    - It prints nothing in any case it does not own, and always exits 0.

## Acceptance Criteria

- A static scan proves the hook module contains no process spawn, no network
  client, no credential read, and no write to the source workspace.
- A child-process test proves the hook binary does not read provider credentials
  even when both are present in the invoking environment.
- An adversarial corpus proves payload content cannot alter policy, capability,
  or budget, and cannot appear in control fields.
- A read-substitution response reproduces exact source bytes verifiable against
  the recorded hash and span; a mutated workspace fails closed.
- An output-reduction response is a strict subsequence of supplied bytes, with
  recorded omissions.
- (1.2) Each returned line is a supplied line with only its terminal escape
  sequences removed, and the response counts the bytes removed.
- Oversize, malformed, NUL, non-UTF-8, and deadline cases fail closed with a
  static category and no disclosure.
- The exchange command (1.1):
  - returns the engine's response for a valid request;
  - returns a closed category and exit 1 for each of these: malformed JSON,
    empty input, an unknown field, invalid base64url, a zero budget, an
    unsupported schema, and oversized input.
- The exchange command (1.2) returns a colored line without its escape
  sequences, and counts them.
- The Claude Code adapter (1.1):
  - keeps a long passing run's result line within 8 KiB;
  - keeps every field of the original result besides `stdout` and `stderr`;
  - returns only lines of each stream, in their original order;
  - prints nothing for another event or tool, an interrupted, background or
    image result, output within the budget, or an unreadable payload.
- The Claude Code adapter (1.2) removes terminal escape sequences from both
  streams, a stream kept whole included, and its note names the bytes removed.
- A static scan of the command module and of the adapter module finds no
  process, socket, environment, or file access. The command's output is
  byte-identical with and without provider credentials in its environment.
- The full repository gate passes.

## Non-Goals

- Running commands, editing files, or guarding writes. Those are host authority
  and remain outside the product.
- Cross-session memory or durable learned knowledge.
- Provider requests, agent orchestration, or benchmark execution.
- Replacing the MCP surface. Hooks are an additional integration shape.
- Rewriting a command before the host runs it. In Claude Code, a failing
  command reaches only `PostToolUseFailure`, which cannot replace output.
  Reaching failing runs would need such a rewrite, and that needs its own
  decision.
- Installing a hook. Recipes are copied by hand.
