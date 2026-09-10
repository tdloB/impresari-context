# ADR-0151: Select Structural Context by What It Is, Not by Identity

- Status: Accepted
- Date: 2026-09-10
- Related PRD: [Stable Structural Selection](../product/stable-structural-selection-prd.md)
- Architecture: [Stable Structural Selection](../architecture/stable-structural-selection-ard.md)
- Refines: [ADR-0145](0145-spread-structural-seeds-across-files.md)
- Unblocks: [#299](https://github.com/tdloB/impresari-context/pull/299), the Tree-sitter 0.27 upgrade
  recorded as ADR-0150

## Context

A disclosure map is chosen under limits. Up to eight seeds are admitted, no
more than three from one file; each seed's traversal keeps sixteen edges at
depth one; the traversals are merged; and a byte ceiling bounds the result.
Three of those choices were decided by identity:

- seed ties within one file, by node identity;
- the order a traversal takes a node's edges, and so which sixteen it keeps, by
  edge identity;
- the merged edge order, and so what the byte ceiling keeps, by edge identity.

Identities hash the workspace snapshot, whose identity hashes the checkout's
resolved path, and an edge identity also hashes its fact provenance, which
includes the recorded parser version. So the delivered map changed with where a
repository is checked out, with any commit, and with a relabel of the parser,
while no fact changed. Measured for the Tree-sitter 0.27 upgrade (#299), relabelling
the parser alone replaced 280 of 1,440 map items and moved map symbol recall
from 15 to 13 of 34.
The same build of `main`, checked out in a second directory, delivered 385 of its 1,440 map items differently and moved map symbol recall from 15 to 12 of 34.

Identity order is also indifferent to what an edge is. On the corpus, 57% of map
items were unresolved references, and hash order admitted them ahead of the
confirmed relationships a seed contains: `astropy-13579` lost the two methods
its fix changes, `pixel_to_world_values` and `world_to_pixel_values`, to such a
reshuffle.

## Decision

Choose by what a relationship is:

- A traversal takes a node's edges most confidently resolved first (confirmed,
  heuristic, unresolved, unsupported), then by where each occurs in the source,
  with its kind, its target's path, name and span, and its module breaking ties.
- Merging keeps seed order and each traversal's order; the first occurrence of
  an edge wins.
- A seed tie within one file goes to the declaration that comes first.

Identity breaks a tie only between entries identical in content. Identities are
unchanged and still bind a fact to its snapshot and provenance; they no longer
decide what is delivered.

## Consequences

Measured on the twenty-two-task astropy corpus with the product's recall scorer:

| build | where it ran | map items | map file recall | map symbol recall |
| --- | --- | --- | --- | --- |
| `main` | first directory | 1,440 | 22/27 | 15/34 |
| `main` | second directory | 1,439 | 22/27 | 12/34 |
| `main`, parser relabelled | first directory | 1,440 | 22/27 | 13/34 |
| this decision | first directory | 1,371 | 22/27 | 15/34 |
| this decision | second directory | 1,371 | 22/27 | 15/34 |
| this decision, parser relabelled | first directory | 1,371 | 22/27 | 15/34 |

With this decision the map is identical, item for item and in order, across the
checkout directory and across the parser label on all twenty-two tasks. Only
identities differ, as they must. Nomination and evidence are unchanged
throughout.

The maps are also smaller: 1,371 items against 1,440 and 1,747,867 delivered
bytes against 1,797,767. Symbol recall matches the count `main` reached in the
first directory but trades one symbol: it gains `__new__` on `astropy-8872` and
loses `_format_value` on `astropy-14508`. Against the second directory it
recovers three.
Items that repeat another item's visible content fall from 727 to 608, and
distinct items rise from 713 to 763. Repeats remain 44% of the map; collapsing
them is a separate decision.

The Tree-sitter upgrade in #299 can merge once this lands: with the parser
relabelled, the map no longer moves.

## Alternatives considered

**Source order alone.** Rejected. Measured from one directory with the rest of
this decision in place, it names 12 of 34 reference symbols against 15, delivers
668 distinct items against 763, and keeps 795 unresolved relationships against
618. In source order a seed's early unresolved references crowd out the
relationships it contains: `astropy-13579` loses both methods its fix changes.

**Keep identity order and remove the path and provenance from identities.**
Rejected: identities bind a fact to the snapshot and provenance that produced it,
which is what makes them verifiable. Selection should not depend on them.

**Rank edges by the task's identifiers.** Deferred: it needs its own measurement,
and a content order is the prerequisite for measuring it cleanly.
