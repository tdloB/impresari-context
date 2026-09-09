# ADR-0145: Spread Structural Seeds Across Files

- Status: Accepted
- Date: 2026-09-08
- Related PRD: [Seed Diversity](../product/seed-diversity-prd.md)
- Architecture: [Seed Diversity](../architecture/seed-diversity-ard.md)
- Refines: [ADR-0125](0125-select-ranked-seed-sets-and-traverse-to-definitions.md)
- Explains: [#271](https://github.com/tdloB/impresari-context/pull/271), [#285](https://github.com/tdloB/impresari-context/pull/285) — both closed as measured-neutral

## Context

A map item's path is its edge's **source** node's path, and at a traversal
depth of one only edges leaving a seed are selected. A map can therefore only
name a file a seed landed in.

Seed candidates are ranked by `SeedRank`, then tie-broken by nomination
position. Neither ordering says anything about spreading, so the
best-nominated file took every slot it could fill. Eight seeds inside one
module traverse overlapping edges: the map came back repetitive rather than
wide. On the worst measured task, forty-six map items carried one file and
eighteen distinct entries, one of them repeated twenty-three times.

This is why widening nomination did nothing. #271 measured that package
proximity reaches ten of sixteen reference files, and #285 measured that
nominating 258 more files added one more nominated reference file and **zero**
to the map. Both were closed as neutral without an explanation. The explanation
is that nomination does not decide which files a map can name; seeding does.

## Decision

No single file contributes more than `MAX_SEEDS_PER_FILE` of the seed set.

Admission preserves rank: a file's surplus candidate is skipped, never promoted
ahead of a better-ranked candidate elsewhere, so the seed set remains a
rank-ordered subsequence and determinism for a snapshot is unchanged.

The maximum is a closed constant. A caller able to steer seed placement could
steer selection, which is oracle authority.

## Consequences

Measured over twenty-two astropy tasks, both arms from the same commit with
verified distinct binaries:

| seeds per file | map file recall | map symbol recall | items | distinct |
| --- | --- | --- | --- | --- |
| unlimited | 18/27 | 15/34 | 1,489 | 665 |
| 1 | 20/27 | 13/34 | 1,451 | 717 |
| 2 | 20/27 | 14/34 | 1,464 | 695 |
| **3** | **20/27** | **15/34** | 1,475 | 700 |

Delivered bytes are 289,184 at every setting, so none of this costs anything.

Two tasks gained and none regressed. Total items fell while distinct items rose,
which is the concentration argument confirming itself: repetition replaced by
reach.

The cap value was swept rather than assumed, and the sweep changed the answer.
Every cap wins the same two files, so the file gain does not depend on the
value. **Symbol recall is what chooses it.** Spreading harder names more files
and fewer symbols inside them: one seed per file names a median of eight files
but gives up two reference symbols. Three is the loosest cap that still spreads
and the only one that gains the files while losing no symbols. A cap of two —
the value this change was prototyped with — would have shipped a one-symbol
regression.

**The size of this result should not be overstated.** Two of twenty-two tasks
changed, on twenty-seven reference files. That is not a strongly powered
comparison, and it is a smaller effect than repairing the budget ceilings
delivered in the same corpus — ADR-0144 and #284 together moved recall from
12/27 to 18/27, three times further, without touching any selection rule.

This record does not help a task whose reference file is never nominated.
Nomination recall is 22 of 27 here and is untouched; those five remain out of
reach for any seeding rule.

## Alternatives considered

**Raise the seed ceiling above eight.** Rejected: it spends more traversal on
the same concentration, and the measured problem is placement, not count.

**Raise traversal depth so targets become sources.** Measured on 2026-09-07: at
depth two a second file does appear, but the binding constraint moves to the
edge budget and the cost lands on every task. Redistributing existing seeds is
free; this is not.

**Cap at one or two seeds per file.** Measured, and rejected on symbol recall:
one loses two reference symbols against the unlimited arm and two loses one,
while three loses none. All three win the same two files.
