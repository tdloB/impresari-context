# ADR-0144: Bound a Structural Query by Its Own Ceiling

- Status: Accepted
- Date: 2026-09-08
- Related PRD: [Structural Query Output Bound](../product/structural-query-output-bound-prd.md)
- Architecture: [Structural Query Output Bound](../architecture/structural-query-output-bound-ard.md)
- Extends: [ADR-0134](0134-truncate-a-disclosure-at-its-ceiling-rather-than-discarding-it.md) — which settled this for the disclosure above the query, and not for the query

## Context

The structural query measured its serialized result against
`budget.requested` — the **consumer's delivery ceiling**, 16,384 bytes as the
evaluator sends it — and returned `BudgetExceeded` when it did not fit.

Those are two different quantities. `requested` bounds the packet a consumer
receives. The value measured against it was the traversal's nodes and edges as
the engine holds them: an intermediate working set, never delivered, with
delivery bounded separately and far lower.

Measured across the twenty-two astropy SWE-bench Verified tasks, one seed's
traversal serializes to:

| statistic | bytes |
| --- | --- |
| minimum | 2,493 |
| median | 13,971 |
| 90th percentile | 15,726 |
| maximum | 17,562 |

Against a 16,384 ceiling. The median successful task spent 85% of it and the
90th percentile 96%; the three failing tasks missed by under 1.2 KiB apiece.
A bound that ordinary work lands on is not a bound.

The consequence was total failure: the traversal discarded whole after the
repository reads that produced it were already spent, and an error carrying no
sign that a limit was involved. Three of twenty-two tasks produce no map on
current `main`; six of twenty-two on a build predating the 4 MiB structural
worker ceiling.

This also explains an unexplained result recorded on 2026-09-07: raising
`MAX_SEED_TRAVERSAL_MATCHES` from 16 to 32 "broke `context_build`" for no
identified reason. More matches produce a larger traversal, which crossed this
same ceiling. It was never a separate defect.

## Decision

The structural query is bounded by `MAX_STRUCTURAL_QUERY_OUTPUT_BYTES`, a
closed engine constant, and a result exceeding it is narrowed rather than
refused.

One mebibyte is about sixty times the observed maximum, deliberately loose: the
real limiter remains the traversal's own node and edge counts, and this bound
exists only so a pathological result is never held whole.

Narrowing drops edges from the tail and never nodes. That is a constraint, not
a preference — the query's consumer resolves every edge's source against
`result.nodes` and fails hard on a miss, so dropping an edge can only shrink
what must be present while dropping a node could break that closure. Edge order
is by `edge_id`, so the retained prefix is deterministic for a snapshot.

A narrowed result reports `truncated` and the reason code
`structural_query_output_limit_reached`. A limit that bites has to be visible.

The consumer's `requested` is still parsed and a malformed budget still
rejected. Ceasing to bound by a value is not ceasing to validate it.

## Consequences

The three tasks that produced nothing now produce maps of 62, 110, and 77
items across four to six files each. Narrowing did not engage for any of them:
they were never oversized results, only ordinary ones measured against the
wrong ceiling.

Delivery is unchanged. The packet a consumer receives is governed by the packet
builder's own bound, exactly as before.

Any recall figure computed over this corpus before this change was computed
where a seventh to a quarter of tasks could not build. Those denominators need
restating before the figures are compared with later ones.

## Alternatives considered

**Raise `requested` at the call site.** Rejected: it would enlarge the packet
the consumer receives to fix an intermediate bound, coupling two things that
should not be coupled, and would leave the fail-whole behaviour intact.

**Keep failing, but with a clearer error.** Rejected: ADR-0134 already settled
that a bound reached after reads are spent should yield what it allows. A
better-worded refusal still throws away work already paid for.

**Drop nodes as well as edges to fit tighter.** Rejected: it breaks the closure
the consumer requires, converting a size problem into a hard failure elsewhere.

**Remove the bound entirely.** Rejected: an unbounded intermediate result is
held whole in memory, and the traversal constants alone are not a byte bound.
