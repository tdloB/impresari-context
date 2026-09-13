# Ceiling Truncation — Architecture Requirements and Design

- ARD ID/version: IC-CT-ARD-134 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-04.
- Governing PRD: [IC-CT-134](../product/ceiling-truncation-prd.md).
- Decision:
  [ADR-0134](../decisions/0134-truncate-a-disclosure-at-its-ceiling-rather-than-discarding-it.md).

## The cost is already sunk

```text
       traversal runs ─▶ items built ─▶ identity computed ─▶ ceiling check
            │                 │                                    │
       reads spent       memory spent                         ✗ discard all
```

By the time the ceiling is tested, the repository has been read, the graph
traversed and the items constructed. Discarding does not refund any of it. The
consumer pays the full cost of the request and receives nothing.

Returning the ceiling's worth of items costs no more than discarding them and
honours the same bound, because the bound is on what a consumer *receives*.

## Why this looked like a selection problem

Raising the per-seed traversal ceiling from 16 to 64 measured as a collapse:
19 of 27 reference files down to 13, and delivered bytes down 19%.

The byte direction is the tell. A wider traversal that selected worse would
deliver *more* bytes and fewer right ones. Delivering fewer bytes and fewer
right ones means responses were disappearing, and six were: five over the
256-item ceiling at 334 to 424 items each, one over the read ceiling.

With truncation the same traversal ceiling delivers 19 of 27 files and 18 of 34
symbols. The budget was never the problem; the response to breaching it was.

This is worth recording as a diagnostic pattern. **A change that reduces both
quality and delivered bytes is usually not a worse selection — it is a failure
being counted as an answer.**

## Truncate before identity

```text
items ─▶ truncate to ceiling ─▶ public items ─▶ map identity ─▶ accounting
```

The map identity hashes the disclosed items. Truncating after it would produce
an identity describing a set the consumer never received, and would leave the
session holding lookup targets that are not in the map.

Truncating first keeps four things describing one set: the identity, the
disclosed items, the session's lookup targets, and the consumption accounting.

## What truncation keeps

The traversal's own order, unchanged. Items arrive ranked by the breadth-first
walk from ranked seeds, so the surviving prefix is the part nearest the seeds —
which is the part the seed ranking already judged most relevant.

Re-ranking at truncation time would be a second, different relevance judgement
layered on the first, and there is no measurement supporting one.

## Disclosure

A truncated map is `partial`, and carries `progressive_item_ceiling_reached`
among its omissions. A consumer can tell a map that ends because the graph ended
from one that ends because a budget did — two different reasons to go looking
further, and only one of them means asking again would help.

## What this does not fix

`astropy-13453` exceeds the repository-read ceiling, not the item ceiling. Reads
cannot be truncated after they happen; that path has to stop reading at its
budget instead, which is a change to the evidence planner rather than to the
disclosure boundary.

The read overage there is itself structural: the planner issues one repository
scan per task signal, so reads grow as signals × files, and 1,869 tracked files
become 11,068 reads. That is the same shape
[ADR-0129](../decisions/0129-build-a-bounded-task-identifier-index-at-preparation.md)
already fixed once for nomination — scan once, match many — and it is recorded
here as the next place to apply it.

## Preserved invariants

`SEC-INV-002`, `SEC-INV-003`, `SEC-INV-007` and `SEC-INV-011` hold unchanged.
Truncation removes items from a response; it adds no capability, reads nothing,
and can only narrow what a consumer receives.
