# Ceiling Truncation PRD

## Document Control

- PRD ID/version: IC-CT-134 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-04.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Ceiling Truncation ARD](../architecture/ceiling-truncation-ard.md).
- Governing decision:
  [ADR-0134](../decisions/0134-truncate-a-disclosure-at-its-ceiling-rather-than-discarding-it.md).
- Governing objective: [CLAUDE.md](../../CLAUDE.md) — 98% quality at 78% compression.

## Problem

When a progressive disclosure exceeds its session ceiling, the whole response is
discarded. The consumer receives `cumulative_ceiling_exhausted` and no map.

The work has already been done at that point. `astropy-13453` builds an 86-item
map, reads the repository 11,068 times, and returns nothing — the reads are
spent either way, and discarding delivers no value for them.

It also makes the disclosure budget untunable. Raising the per-seed traversal
ceiling from 16 to 64 sends five further tasks over the 256-item ceiling, each
returning 334 to 424 items and each discarded whole:

| | map files | tasks returning no map | bytes |
| --- | --- | --- | --- |
| traversal ceiling 16 | 19 of 27 | 1 | 19,633,830 |
| traversal ceiling 64 | 13 of 27 | **6** | 15,880,994 |

Recall falls by six files and delivered bytes fall 19%, which reads as a
selection regression and is not one. No budget can be evaluated while breaching
it deletes the answer.

## Product Outcome

A disclosure that exceeds its ceiling returns what the ceiling allows, and says
that it did. The ceiling still bounds what a consumer receives; it stops
determining whether the consumer receives anything.

## Functional Requirements

1. When a map holds more items than the session's item ceiling admits, disclose
   the number the ceiling admits rather than discarding the response.
2. Truncate before the map identity is computed, so the identity, the disclosed
   items, the session's lookup targets, and the consumption accounting all
   describe the same set.
3. Record truncation explicitly: the map's state is partial and an omission
   names the ceiling that bit.
4. Never exceed the ceiling. Truncation admits fewer items, never more.
5. Stay deterministic. Identical inputs truncate to an identical set.
6. Leave a disclosure that fits entirely unchanged.

## Acceptance Criteria

- A map exceeding the item ceiling returns exactly the ceiling's worth of items,
  marked partial, with the ceiling disclosed.
- A map within the ceiling is byte-identical to today's.
- The disclosed items, map identity and lookup targets agree after truncation.
- **Measured, offline, over all twenty-two astropy tasks:** at the shipped
  traversal ceiling this changes nothing, because no map exceeds the item
  ceiling there. At a raised traversal ceiling it recovers every task the
  discard loses.
- The full repository gate passes.

## Measured Outcome

| | map files | map symbols | tasks with no map | bytes |
| --- | --- | --- | --- | --- |
| ceiling 16, discard (`main`) | 19 of 27 | 15 of 34 | 1 | 19,633,830 |
| ceiling 64, discard | 13 of 27 | 14 of 34 | 6 | 15,880,994 |
| ceiling 64, **truncate** | **19 of 27** | **18 of 34** | **1** | 20,908,386 |

Truncation recovers all six reference files the discard loses and three further
reference symbols.

**At the shipped traversal ceiling of 16 this change measures nothing**, because
no map reaches 256 items there. It is admitted as correctness and as the
precondition for tuning a budget, not as a recall improvement.

The one task still returning no map is `astropy-13453`, which exceeds the
repository-read ceiling rather than the item ceiling. Reads cannot be truncated
after the fact; bounding them is separate work.

## Non-Goals

- Bounding repository reads during a scan. `astropy-13453` needs the evidence
  path to stop reading at its budget, which is a different change.
- Choosing a traversal ceiling. This makes that choice measurable; it does not
  make it.
- Ranking which items survive truncation. The traversal's own order is kept.
