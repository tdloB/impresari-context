# ADR-0146: Name the File a Relationship Points Into

- Status: Accepted
- Date: 2026-09-10
- Related PRD: [Target File Disclosure](../product/target-file-disclosure-prd.md)
- Architecture: [Target File Disclosure](../architecture/target-file-disclosure-ard.md)
- Refines: [ADR-0145](0145-spread-structural-seeds-across-files.md)

## Context

A map entry takes its symbol from the relationship's **target** node and its
path from the **source**. So an entry reads `sampled.py — BaseTimeSeries` while
`BaseTimeSeries` is declared in `timeseries/core.py`, and `core.py` is never
named. A consumer following that entry is pointed at the wrong file.

The target node is resolved and already in hand: the entry looks it up to read
the symbol name, then discards its path.

ADR-0145 established that a map can only name a file a **seed** landed in, and
spread seeds so more files qualify. This is the other half of the same
constraint. A file the traversal *reached* is knowable without being seeded,
and was being withheld.

Measured over the twenty-two astropy tasks: of the nine reference files the map
fails to name, **three are already present as relationship targets** —
`timeseries/core.py` (13033), `table/table.py` (13236), and
`nddata/mixins/ndarithmetic.py` (14995). The first is the file behind the
zero-recall case investigated across three separate lines of work.

## Decision

A map entry whose relationship resolves into a different file names that file.

An entry whose relationship stays inside one file names none; repeating the
source path on every entry would grow the scarcest part of the packet while
telling a consumer nothing. An unresolved relationship names none, because
there is no resolved node to name.

The evaluator admits a disclosed target path only when the frozen source
allowlist already admits it — the same rule an entry's own path obeys.

## Consequences

Map file recall rises from 18 of 27 to 21 of 27 on the measured corpus, from
data the traversal already produced. No traversal is extended, no repository
read is added, and no node is looked up that was not already looked up.

`item_handle` is unchanged, because it hashes the edge and the edge carries the
target node. `map_id` **does** change, because it hashes the rendered items; any
pinned or cached map identity from an earlier build is stale.

The honest cost is precision. Across the corpus, entries hold 23 distinct files
no entry currently names — a 32% increase in map breadth — of which three are
reference files. **Roughly one added file in eight is a file the change
touched.** Whether the other seven help a consumer orient or distract it is not
measurable without a graded run, and this record does not claim to know.

## Alternatives considered

**Emit the target path on every entry.** Rejected: most relationships stay
inside one file, so the common case would repeat the entry's own path and
inflate the map for no information.

**Replace `display_path` with the target's path.** Rejected: it discards where
the relationship was observed, which is what makes an entry verifiable against
the snapshot, and would simply move the error rather than fix it.

**Raise traversal depth so target files become sources.** Measured on
2026-09-07: at depth two a second file does appear, but the edge budget becomes
binding and the cost lands on every task. This is free by comparison and
reaches the same files.
