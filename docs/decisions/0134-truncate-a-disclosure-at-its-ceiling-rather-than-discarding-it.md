# ADR-0134: Truncate a Disclosure at Its Ceiling Rather Than Discarding It

- Status: Accepted
- Date: 2026-09-04
- Related PRD: [Ceiling Truncation](../product/ceiling-truncation-prd.md)
- Architecture: [Ceiling Truncation](../architecture/ceiling-truncation-ard.md)
- Refines: [ADR-0121](0121-use-bounded-progressive-structural-disclosure.md)

## Context

A progressive disclosure that exceeds its session ceiling is discarded whole.
The consumer receives `cumulative_ceiling_exhausted` and no map.

The work is already done when that check runs. `astropy-13453` builds an 86-item
map after 11,068 repository reads and returns nothing; the reads are spent
whether or not the map is delivered.

It also makes the disclosure budget untunable. Raising the per-seed traversal
ceiling from 16 to 64 measured as a collapse — 19 of 27 reference files down to
13, delivered bytes down 19% — which reads as a selection regression and is not
one. Five tasks had produced maps of 334 to 424 items against a 256-item ceiling
and been discarded; a sixth exceeded the read ceiling.

The byte direction is what gives it away. A traversal that selected worse would
deliver more bytes and fewer right ones. Fewer bytes *and* fewer right ones
means responses were vanishing.

## Decision

When a map holds more items than its session ceiling admits, disclose the number
the ceiling admits and record that it was truncated. Do not discard the
response.

Truncate before computing the map identity, so the identity, the disclosed
items, the session's lookup targets and the consumption accounting all describe
the same set. Keep the traversal's own order. Mark the map partial and disclose
`progressive_item_ceiling_reached`.

## Consequences

At the shipped traversal ceiling this changes nothing measurable, because no map
reaches 256 items there. It is admitted as correctness — a request that has
already paid its cost should receive what the ceiling allows — and as the
precondition for tuning any disclosure budget.

With truncation, the same raised traversal ceiling that measured 13 of 27
delivers 19 of 27 files and 18 of 34 symbols, against 15 of 34 at the shipped
ceiling. Whether to raise it is now a real trade — three more reference symbols
for 6.5% more bytes — rather than a collapse, and it is a separate decision from
this one.

The ceiling still bounds what a consumer receives. It stops determining whether
the consumer receives anything, which was never what a bound was for.

`astropy-13453` is unfixed. It exceeds the repository-read ceiling, and reads
cannot be truncated after they happen. The overage is structural: the evidence
planner issues one repository scan per task signal, so 1,869 tracked files
become 11,068 reads. That is the shape
[ADR-0129](0129-build-a-bounded-task-identifier-index-at-preparation.md) already
fixed once for nomination, and applying it to the evidence path is the next
change rather than this one.

No security invariant changes. Truncation only narrows a response. This record
grants no execution, network, publication, or submission authority.
