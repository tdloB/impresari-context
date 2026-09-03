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
