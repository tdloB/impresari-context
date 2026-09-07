# Seed-Scoped Structural Extraction — Architecture Requirements and Design

- ARD ID/version: IC-SSSE-ARD-128 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-03.
- Governing PRD: [IC-SSSE-128](../product/seed-scoped-structural-extraction-prd.md).
- Decision:
  [ADR-0128](../decisions/0128-extract-structure-for-nominated-files-not-whole-repositories.md).

## The inversion

Today structure is extracted first and the task arrives second:

```text
startup:   every file ──thin extraction──▶ whole-repository graph
request:   task ──▶ seed into that graph ──▶ map
```

The graph must therefore be thin enough to hold the entire repository, which is
what makes it useless on a repository of ordinary size.

The new order lets the task decide what is worth extracting:

```text
startup:   snapshot and lexical index only
request:   task ──▶ nominate ≤N files
                      │
                      ▼
              dense extraction for those files (cache-reused)
                      │
                      ▼
              small dense graph ──▶ seed ──▶ traverse ──▶ map
```

Nothing about the extraction itself changes. What changes is how many files it
runs over and how much allowance each one gets.

## Why density is the whole argument

A serialized graph is capped at 16 MiB by the local store, and one admitted fact
costs roughly 530 bytes of it. That affords about 31,600 facts in total.

| scope | supported files | facts per file | observed result |
| --- | --- | --- | --- |
| whole repository, production budget | 1,172 | ~1 | 0 of 27 file recall |
| whole repository, ceiling budget | 1,172 | ~27 | still no seed; 113 s |
| 47-file subtree | 47 | ~213 | 16 relevant items; 3.3 s |

### Dividing it evenly is the second half of the same problem

Scoping fixes the count. It does not fix the division. Each file receives
`remaining ÷ files remaining`, walked in snapshot order — alphabetical, and
unrelated to what any file needs. Leftovers flow forward, so a file walked late
can exceed an equal share; a file walked early cannot, however much the build
leaves unspent.

Measured on astropy, one build nominated 13 files, used 12,040 of its 28,000
facts, and truncated a file at exactly `28,000 ÷ 13 = 2,154`. **57% of the
allowance went unclaimed while a file was cut short.** The capacity was in the
same build, unspent, and unreachable.

So the allowance is divided by need. The worker reports
`total_facts_available` — what the file would yield with no ceiling — and the
unspent remainder is placed with the files that were cut, smallest shortfall
first, re-parsing only those.

Only the parser can supply that number honestly. The alternative is estimating
demand from file size, and measured source bytes per fact ranges 18.6 to 35.7
across eight astropy modules, with inversions: two files 1,200 bytes apart differ
by 1.7× in facts. An estimator is a model of the parser living outside it, and a
wrong conclusion has already been drawn in this project from exactly that.

The parse is not the cost. Tree-sitter builds the whole tree before a single fact
is emitted, so the ceiling was never saving a parse — it was abandoning a
half-finished walk over a tree already in memory, and discarding the count with
it. Finishing that walk is what makes the division exact.

Two properties keep this safe to apply unconditionally: no file is granted less
than it already holds, so a build cannot regress; and only the unspent remainder
moves, so the allowance cannot be exceeded. The cost is that a first pass which
spends everything leaves nothing to redistribute — a case the corpus did not
produce.

## Why density is the whole argument, continued

The budget is not the problem and the ceiling is not the problem. Dividing a
fixed allowance across an unbounded file count is the problem. Nominating
sixty-four files at two hundred facts each is roughly 12,800 facts — about
6.8 MiB, comfortably inside the ceiling, at a density the product demonstrably
uses well.

## Nomination boundary

Nomination consumes exactly two things: the task text and the current workspace
snapshot. It consumes no reference change, no accepted patch, and no test
outcome. That is the same input boundary seed selection already respects, and it
is enforced the same way — the engine has no path to oracle data, checked
statically.

Ranking is deterministic and total: an exact task path, then a file containing
an exact task identifier, then remaining lexical matches, ties broken by
portable path. The maximum is a closed constant, because a caller able to widen
nomination could steer it, and steering is oracle authority.

## Cost moves, and mostly downward

Whole-repository preparation currently costs about 20 seconds before the first
request can be answered, and it is paid whether or not the structure is used.
Under scoping, startup needs only the snapshot and lexical index, and structural
work happens per request over a few dozen files.

The existing per-file structural cache is keyed by content hash and toolchain
identity, so a file extracted for one task is free for every later task that
nominates it. Repeat work across a session converges toward zero.

### The response ceiling is the other half of density

Scoping decides how many files share the allowance. The worker's response
ceiling decides how much of any one file can come back at all, and it is a
separate bound that a denser scope runs into sooner.

`requested` bounds the response frame, and over it the worker returns a prefix
of the fact list rather than failing. At the 1 MiB the server asked for — about
2,764 facts at a measured 380 bytes each — the split across eight astropy modules
falls exactly at that threshold: `table.py` (6,146 facts), `header.py` (3,734),
`quantity.py` (3,489) and `ascii/core.py` (3,381) all lost declarations, while
`card.py` (2,693), `sky_coordinate.py` (2,210), `sampled.py` (604) and
`timeseries/core.py` (188) lost none.

Measured over ten tasks and thirty-nine nominated files, it was the binding
constraint on 11 of the 12 truncated files. Raising the request to 4 MiB takes
truncation to 4 files and structure recovered from 52,293 to 70,401 facts.

Nothing external requires 1 MiB. The store's 16 MiB graph cap is a
`set_limit(SQLITE_LIMIT_LENGTH, …)` this project makes, against a SQLite default
of 1 GB; 4 MiB is simply the largest a conservative budget admits without
widening a validated range, and it was enough for every file measured.

The cost is disk and nothing else: about 2.2 MB more cache per scoped build.
Build time does not move — two runs per configuration put the within-run spread
above the between-configuration difference — and peak resident memory is
identical to within 0.1 MB, because the response buffer is transient and per file
while the peak is set by the whole-repository startup graph.

What the extra structure is *for* is worth stating, because the obvious answer is
wrong. Measured across all twenty-two astropy tasks at their own base commits,
map file recall is 19 of 27 and symbol recall 13 of 34 — **identical at both
ceilings, with no task moving**. Recovering a third more structure produced no
better map.

Where it does land is read substitution. A host asking for one named declaration
gets a false negative when the graph never reached it, and 116 of 494 symbols
were unanswerable for exactly that reason. At 4 MiB all 494 answer. Density at
the file level serves the hook that reads a single file, not the map that ranks
across files — and that distinction is measured rather than assumed.

## The failure mode this introduces, stated plainly

A whole-repository graph is thin but complete. A scoped graph is dense but
partial. If nomination misses the file that mattered, the map will be confident,
well-attributed, and wrong — and unlike today it will look healthy.

Two controls answer that:

**Disclosure.** Every scoped result states that coverage is limited to nominated
files, how many were nominated, and why each was admitted. A scoped graph must
never be mistakable for a complete one, so a consumer knows when to look wider.

**Nomination recall as the leading metric.** A file never nominated can never be
mapped, so nomination recall bounds map recall from above. Measuring it offline
against reference changes tells us whether nomination is the ceiling before any
effort goes into ranking beneath it. The measured evidence file recall of 22%
suggests this is exactly where the ceiling currently sits.

## Preserved invariants

`SEC-INV-002`, `SEC-INV-003`, `SEC-INV-007`, `SEC-INV-009`, `SEC-INV-010`, and
`SEC-INV-011` all hold unchanged. Scoping reduces the work performed and the
data retained; it grants no capability. The structural worker contract, its
isolation, and its bounds are untouched.

## Relationship to earlier records

[ADR-0121](../decisions/0121-use-bounded-progressive-structural-disclosure.md)
established bounded progressive disclosure over a whole-repository graph.
[ADR-0125](../decisions/0125-select-ranked-seed-sets-and-traverse-to-definitions.md)
improved selection within that graph and measurably did not move recall,
because the graph did not contain what selection needed. This record changes
what the graph is built from; it keeps disclosure, bounds, determinism, and the
seed ranking those records established.
