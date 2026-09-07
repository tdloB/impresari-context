# ADR-0140: Deliver One Excerpt Once

- Status: Accepted
- Date: 2026-09-07
- Amends: nothing; this corrects delivered evidence rather than changing a decision

## Context

An evidence record is identified by its span. Two matches a few bytes apart in
one file are therefore two records — and the excerpt expanded around each can be
byte-identical.

Both were delivered. Measured on a forty-nine file astropy subset with an
evidence budget of twenty:

| | |
| --- | --- |
| records delivered | 20 |
| distinct excerpts among them | **12** |
| repeat copies | 8 |
| excerpt bytes delivered | 80,780 |
| bytes that were repeats | **32,198 (40%)** |

Seven of the twenty carried one identical ~1,500-byte window of
`astropy/timeseries/binned.py` — its licence header, imports and class
docstring. A reader met that text seven times, and the packet budget paid for it
seven times.

This also made the product unusable to a consumer that requires distinct
evidence. The evaluation harness derives an evidence handle from path, line
range and content hash, so the repeats collide and it rejects the whole
response. Above three evidence items no exchange completed at all, against a
bundle that asks for two hundred.

## Decision

Admit an evidence record only when the packet does not already deliver its
bytes. Two records agreeing on path, extraction method, kind and excerpt bytes
are one delivery.

Include provenance in that judgement. Structural-graph evidence and a literal
match can expand to the same window and remain two different things the product
found; collapsing them would erase what the product knows about how it was
found. Only records agreeing on method and kind are the same delivery.

Do not carry the withheld record's match positions forward on the surviving
record. Carrying them was implemented and measured, and it is wrong: the packet
drops evidence from the tail until it fits its byte budget, so growing a
surviving record can push another record out entirely. On a 4 KiB fixture that
traded a harmless repeat for the loss of the only structural-graph evidence in
the packet. The reader still receives the window those matches fall inside.

## Consequences

The same budget now buys distinct evidence. On the measured subset, twenty
records carry twenty distinct excerpts rather than twelve, so eight windows of
budget that were repeats are now new evidence.

A consumer requiring distinct evidence can raise its budget. The evaluation
harness completes at twenty evidence items against this product where it
previously failed above three.

No contract changes. `EvidenceExcerpt` is untouched, no schema version moves,
and a consumer sees fewer repeated records and nothing new to parse.

Evidence identity is unchanged for every delivered record. What changes is which
records are delivered, and only where the bytes were already going to be sent.

Nothing is claimed about correctness or tokens. This removes repetition from a
packet; whether a reader does better with the freed budget is a separate
measurement and is not made here.
