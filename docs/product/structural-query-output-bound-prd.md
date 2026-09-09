# Structural Query Output Bound PRD

## Document Control

- PRD ID/version: IC-SQOB-144 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-08.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Structural Query Output Bound ARD](../architecture/structural-query-output-bound-ard.md).
- Governing decision:
  [ADR-0144](../decisions/0144-bound-a-structural-query-by-its-own-ceiling.md).
- Extends: [ADR-0134](../decisions/0134-truncate-a-disclosure-at-its-ceiling-rather-than-discarding-it.md).

## Problem

A structural map is not produced at all for a substantial share of ordinary
tasks, and the consumer is told only that the build failed.

Measured across the twenty-two astropy SWE-bench Verified tasks, the serialized
size of one seed's traversal is:

| statistic | bytes |
| --- | --- |
| minimum | 2,493 |
| median | **13,971** |
| 90th percentile | 15,726 |
| maximum | 17,562 |

The ceiling those were measured against was **16,384** — the consumer's packet
budget, which the engine reused as the query's output limit. The median
successful task therefore spent 85% of it, the 90th percentile 96%, and the
tasks that failed missed by **under 1.2 KiB apiece**.

Three of twenty-two tasks fail on the current product for this reason, and six
of twenty-two on a build without the 4 MiB structural worker ceiling. A limit
that ordinary work lands on is not a limit; it is a coin toss.

The failure is total. The traversal is discarded whole after the repository
reads that produced it have already been spent, and the error the consumer sees
carries no indication that a bound was involved.

## Product Outcome

A structural query is bounded by a ceiling of its own, sized for the
intermediate value it actually holds, and a result that exceeds it is narrowed
and disclosed rather than refused.

## Functional Requirements

1. The structural query's output bound is a closed constant, independent of any
   consumer-supplied delivery budget.
2. A result within that bound is returned unchanged.
3. A result exceeding it is narrowed to fit rather than failing the build.
4. Narrowing drops relationships, never nodes, so every retained relationship
   keeps the node it originates from.
5. Narrowing is deterministic for a snapshot: the same query narrows the same
   way every time.
6. A narrowed result is disclosed as truncated, with a stable reason code, so a
   consumer can tell that a bound bit.
7. A malformed consumer budget is still rejected. Ceasing to bound by it is not
   ceasing to validate it.

## Acceptance Criteria

- The three tasks that fail on current `main` produce maps.
- A test proves an oversized result is narrowed, disclosed, and still contains
  its nodes and the sources of every retained relationship.
- A test proves a result within the bound is returned byte-identical.
- Narrowing an oversized result completes in time proportional to the logarithm
  of its relationship count, not its square.
- The full repository gate passes, including adversarial and fuzz suites.

## Non-Goals

- Raising traversal breadth or depth. This record changes no seed, nomination,
  or traversal constant.
- Increasing what a consumer receives. The delivered packet stays bounded
  exactly as before.
- Diagnosing why individual tasks produce large traversals.
