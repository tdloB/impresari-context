# ADR-0130: Reach by Package Proximity, Not Import Following

- Status: Accepted as a decision; implementation measured neutral and held unmerged
- Date: 2026-09-04
- Related PRD: [Package-Proximate Nomination Reach](../product/package-proximate-nomination-reach-prd.md)
- Architecture: [Package-Proximate Nomination Reach](../architecture/package-proximate-nomination-reach-ard.md)
- Extends: [ADR-0128](0128-extract-structure-for-nominated-files-not-whole-repositories.md)

## Context

Map file recall stands at 11 of 27 reference files across twenty-two accepted
changes. Fifteen of the sixteen misses are files the task report never names, so
they are unreachable from task signals no matter how the ranking is arranged.
Three independent measurements agree that identifier matching is exhausted: the
slot curve is flat from 16 candidates to 64, the truncation-ordering sweep is
flat across every ratio tested, and the misses are absent from the report text.

Further reach has to come from repository structure. The intended mechanism was
following imports and inheritance out of a nominated file.

Measured against the sixteen missed files, import edges reach **1**. Files in
the same package as a nominated file reach **10**. The union is also 10 —
import following contributes nothing package proximity does not already cover.

Anchor depth was measured separately. Expanding from the top-ranked nominated
file reaches 8 of 16 and adds about 10 files. Two anchors reach 9 and add 13.
Three anchors reach 9 and add 23.

## Decision

Expand nomination by package proximity: admit files sharing the immediate parent
directory of the single highest-ranked directly nominated file, bounded by a
closed reach ceiling, ordered strictly after every directly nominated file.

Do not build import, inheritance, or call-graph following.

Define a package as the immediate parent directory. Keep the anchor rule, the
package rule, and the reach ceiling as compile-time constants. Set the ceiling
high enough to admit an ordinary package whole: it is a guard against a
pathological directory, not a selector, because truncation orders by path and a
package is not alphabetical by relevance.

Spend the fact allowance in two passes. Nominated files divide the whole
allowance between themselves and reach files divide only what survives, so
admitting a sibling cannot thin a file the task named.

Mark reach files with their own reason code and record an explicit unknown
whenever a nomination contains one, surfaced to the consumer so a partial map
says which kind of partial it is.

Claim no improvement until map file recall is measured offline over the whole
corpus and gains at least two reference files without losing any currently
recalled.

## Consequences

The reachable ceiling rises from 11 of 27 to at most 19 of 27. That is a
ceiling, not a result: admitting a file to the scope makes its declarations
available to the map without putting them in it.

Reach and the tiered allowance are one change and are only safe together. Built
without the tier, reach divided `MAX_SCOPED_FACTS` equally across a wider scope,
cutting every nominated file's share from 1,750 facts to 1,000; measured, that
cost `astropy-13236` its reference file for a net 10 of 27 against a baseline of
11. The tier restores each nominated file's share to exactly what it was before
reach existed, which is what turns reach from a trade into an addition — and it
is also what makes a wide reach ceiling safe.

Rejecting import following saves an import resolver per admitted language, which
is a substantial and recurring cost, and the measurement says it would have
bought one file out of sixteen. This record exists mainly so the idea is not
re-proposed from intuition later; it is a plausible design that the corpus does
not support.

The package rule is deliberately crude. A language-aware definition may reach
further, but nothing measured says so yet, and the crude rule needs no
per-language resolver.

No security invariant changes. Reach performs no additional repository read and
this record grants no execution, network, publication, or submission authority.

## Measured outcome

Three variants were built and measured over the twenty-two astropy tasks:

| variant | map file recall | bytes |
| --- | --- | --- |
| baseline, no reach | 11 of 27 | 18,995,114 |
| reach, allowance divided equally | 10 of 27 | 18,989,514 |
| reach, tiered allowance | 11 of 27 | 19,027,888 |
| reach, tiered allowance, ceiling 32 | 11 of 27 | 19,027,888 |

Reach does not move map file recall. The PRD requires two files gained and none
lost; it gained none. Widening the ceiling from twelve to thirty-two changed no
delivered map at all.

The reachability ceiling that motivated this decision was measured offline with
a nomination ranking that reimplemented the engine's rules rather than calling
them, and the two rankings do not agree on which file anchors a task. Reach
expands from the anchor's package, so an anchor in the wrong package expands in
the wrong direction — measured on `astropy-13398`, the anchor sits in
`astropy/coordinates` while all four reference files sit in
`astropy/coordinates/builtin_frames`, a different package under this decision's
own rule. The 8-of-16 ceiling therefore overstates what reach can reach in the
product.

Classifying the eleven remaining failures says the same thing from the other
side. Three produce no seed at all and so no map; two seed into vendored
`astropy/extern` code; one exceeds the repository read ceiling before it
answers. Six of eleven are seed-selection failures that no scope widening can
address, and reach is aimed only at the other half.

The decision stands — package proximity is the right reach mechanism, and
import following is measurably not — but the implementation is held unmerged
until nomination picks a better anchor. Reach cannot be evaluated on its merits
while the file it expands from is the wrong one.

## Re-measured on a better anchor

Combined with declaration-aware nomination
([ADR-0131](0131-admit-and-rank-identifiers-by-declaration.md)), which gives
reach the anchor it lacked:

| variant | map files | map symbols | bytes |
| --- | --- | --- | --- |
| declaration-aware alone | 14 of 27 | 10 of 34 | 19,293,774 |
| declaration-aware plus reach | 15 of 27 | 11 of 34 | 19,310,728 (**+0.09%**) |

Reach is now strictly positive — one reference file gained, none lost, for
essentially no bytes — where on the old anchor it was neutral at best. That is
still one file, and one file is inside the noise band this corpus supports, so
it does not yet clear the PRD's two-file bar.

The per-task detail says why, and it is the more useful result. The gain came
from `astropy-14309`, where reach admitted `io/fits/connect.py` and traversal
happened to walk into it. The two tasks predicted to gain — `astropy-8707`
missing `io/fits/card.py`, `astropy-13033` missing `timeseries/core.py`, each a
sibling of a file already in the map — produced **byte-identical maps**.

Reach admitted those siblings to the graph and the map never reached them.
Scope and map are separate: reach controls which files a graph covers, while
seeds and traversal control which files a map returns. Reach pays only when
traversal incidentally walks into a newly admitted file, which is why its effect
is small and hard to predict.

That makes traversal, not scope, the binding constraint on map recall, and it is
the finding this decision's implementation is most useful for having produced.
