# Seed Diversity — Architecture Requirements and Design

- ARD ID/version: IC-SD-ARD-145 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-08.
- Governing PRD: [IC-SD-145](../product/seed-diversity-prd.md).
- Decision: [ADR-0145](../decisions/0145-spread-structural-seeds-across-files.md).

## Why a map names so few files

```text
map item.display_path  =  its edge's SOURCE node's path
traversal depth        =  1  ──▶ only edges leaving a seed are selected
                                  │
                       therefore  ▼
        a map can only name a file a SEED landed in
```

Nomination admits sixteen files. Traversal reaches further still. Neither
widens the map, because neither changes which nodes are sources of selected
edges. Only seeding does.

That makes seed placement, not nomination breadth and not traversal reach, the
thing that decides which files a map can name — which is why widening
nomination by package proximity moved recall by exactly zero (#271, #285).

## The concentration

Candidates are ranked by [`SeedRank`], then tie-broken by nomination position.
Both orderings are sound. Neither says anything about spreading, so the
best-nominated file fills every slot it can.

Eight seeds inside one module traverse overlapping edges. The result is not
eight modules' worth of relationships; it is one module's relationships,
several times over. On the worst measured task a map of forty-six items carried
one file and eighteen distinct entries, with a single entry repeated
twenty-three times.

## The rule

Admit in rank order; skip a candidate whose file already holds
`MAX_SEEDS_PER_FILE`. Skipping, not reordering: a file's surplus candidate is
dropped, and every other candidate keeps its position. The seed set stays a
rank-ordered subsequence of what `sort_seed_candidates` produced, so
determinism for a snapshot is unchanged.

Admission is extracted as `admit_seeds`, a pure function over ranked candidate
tuples, so the rule is testable without constructing a graph.

## Measured

Twenty-two astropy tasks, both arms built from the same commit with verified
distinct binaries, only this rule differing:

| seeds per file | map file recall | map symbol recall | items | distinct |
| --- | --- | --- | --- | --- |
| unlimited | 18/27 | 15/34 | 1,489 | 665 |
| 1 | 20/27 | 13/34 | 1,451 | 717 |
| 2 | 20/27 | 14/34 | 1,464 | 695 |
| **3** | **20/27** | **15/34** | 1,475 | 700 |

Delivered bytes are 289,184 at every setting: the packet is budget-bound, so
none of this costs anything.

Two observations decide the constant.

**The file gain does not depend on it.** Every cap wins the same two reference
files. Any rule that stops one file monopolising the seed set is enough; the
value is not what produces the improvement.

**Symbol recall does.** Spreading harder names more files and fewer symbols
inside them — at one seed per file the map names a median of eight files but
loses two reference symbols against the unlimited arm. Three is the loosest cap
that still spreads, and the only one that gains the files while giving up no
symbols.

Total items fall and distinct items rise at every setting, which is the
concentration argument confirmed: repetition replaced by reach.

## What this does not do

It does not raise the seed ceiling, traversal depth, nomination breadth, or the
byte budget. It redistributes a fixed number of seeds.

It cannot help a task whose reference file is never nominated. Nomination
recall is 22 of 27 on this corpus and is unchanged by this record; the five
files nomination misses remain out of reach for any seeding rule.

## Preserved invariants

`SEC-INV-002`, `SEC-INV-003`, `SEC-INV-007`, `SEC-INV-011` and `SEC-INV-012`
all hold. The rule consumes only the ranked candidate list already derived from
task signals and the snapshot, grants no capability, and is a closed constant a
caller cannot widen — a caller able to steer seed placement could steer
selection, which is oracle authority.
