# ADR-0158: Tell a Session It Already Holds an Unchanged Read

- Status: Accepted
- Date: 2026-09-12
- Related PRD: [Repeat Read Notice](../product/repeat-read-notice-prd.md)
- Architecture: [Repeat Read Notice](../architecture/repeat-read-notice-ard.md)
- Amends: [ADR-0136](0136-substitute-a-host-read-with-declaration-spans.md)
- Respects: [ADR-0135](0135-reuse-a-verified-read-within-one-request.md)

## Context

Read substitution (ADR-0136) answers every offer in full. An agent that reads a
file again in the same session is sent the same declarations again, and pays for
them in its context twice. Comparable context tools answer a repeated read with
a notice that the content has not changed.

## Decision

1. Per session, the server remembers the **identity** of each read-substitution
   answer it delivered, and the request that delivered it. The identity is a
   hash over the path, the symbol, each span's range and content hash, the
   omitted spans and the unknowns. It never keeps the bytes.
2. When a session offers the same path and symbol again and the newly computed
   answer has the same identity, the server sends a **notice** instead of the
   spans. The notice names the earlier request and the answer identity, and
   says how many bytes it withheld. The source is read and verified as before;
   only the resend is skipped.
3. The notice is an offer. `repeat: true` returns the full answer, the host can
   always read the file itself, and the notice says both.
4. A changed source, a different symbol, or another session always gets the full
   answer. Closing a session forgets it. Past 4,096 remembered answers in one
   session, recording stops rather than evicting.

## Consequences

An agent that reads an unchanged file again receives a notice of a few hundred
bytes instead of the answer.

An agent whose context was compacted may no longer hold the earlier answer. It
then spends one more call, with `repeat`, or reads the file. The notice says so
in words, because an agent cannot be assumed to remember what it was sent.

No repository content is retained, so ADR-0135 still holds, and no authority is
added.

Offline, this shows only the behaviour. The saving depends on how often agents
repeat reads, which a graded run measures.

## Alternatives considered

**Answer a repeat from memory without reading the source.** Rejected. The
notice claims the answer is unchanged, so the answer is recomputed and compared.

**Remember the bytes and serve them again without a read.** Rejected by
ADR-0135: repository content must not outlive the request that read it.

**Recognise repeats across sessions.** Rejected. Another session may never have
been sent the answer.
