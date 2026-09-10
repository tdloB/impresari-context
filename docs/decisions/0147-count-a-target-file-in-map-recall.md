# ADR-0147: Count a Target File in Map Recall

- Status: Accepted
- Date: 2026-09-10
- Related PRD: [Recall Report Target Attribution](../product/recall-report-target-attribution-prd.md)
- Architecture: [Recall Report Target Attribution](../architecture/recall-report-target-attribution-ard.md)
- Follows: [ADR-0146](0146-name-the-file-a-relationship-points-into.md)

## Context

ADR-0146 made a map entry name the file its relationship resolves into, as
`target_display_path`. The recall scorer read only `display_path`, so it could
not count that. On current `main` it reported 20 of 27 reference files named
where the map named 21 — the metric was blind to the change it was meant to
measure.

The recall report schema is defined only in the scorer and pinned nowhere else.
Whether it needed a record at all was an open question; it is settled here.

## Decision

A `target_display_path` counts toward whether the map names a reference file,
and a reference file named only that way is not reported as missing. The number
of reference files named only through a target is reported separately, per case
and in total, rather than folded into the headline.

The report schema moves from 1.0 to 1.1, because what `map_file_recall` measures
has changed.

**The recall report schema is governed.** It is the instrument every recall
claim in this decision record rests on, and a silent change to its meaning is
indistinguishable from a change in the product. Any future change to what it
measures moves the version and carries a record, whether or not another
artefact pins it.

## Consequences

On the twenty-two-task astropy corpus, the same build scores:

| scorer | map file recall |
| --- | --- |
| 1.0 | 20/27 |
| 1.1 | 21/27, one via target (`astropy-14995`) |

Recall figures quoted from earlier records were measured under 1.0 and remain
correct statements of what 1.0 measured. They are not comparable with 1.1
figures without that qualification, and they are not re-scored.

The target-only contribution is visible in every report, so a later reader can
tell how much of a recall figure depends on target disclosure rather than on
which files the map was seeded into.

## Alternatives considered

**Fold target paths into the headline without attribution.** Rejected: it would
raise the number while hiding its source, reproducing the failure being
corrected one level higher.

**Report target recall as a separate metric and leave `map_file_recall`
unchanged.** Rejected: the scorer defines map recall as whether an agent is
pointed at the right place, and a target does point it there. Leaving it out
would keep the headline wrong by the metric's own definition.

**Change the meaning and keep the version at 1.0.** Rejected: two reports would
disagree on the same build while claiming to measure the same thing.

**Leave the schema ungoverned because nothing pins it.** Rejected: being
unpinned is what made a silent change to its meaning possible in the first
place.
