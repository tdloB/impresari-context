# Recall Report Target Attribution — Architecture Requirements and Design

- ARD ID/version: IC-RRS-ARD-147 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Governing PRD: [IC-RRS-147](../product/recall-report-target-attribution-prd.md).
- Decision: [ADR-0147](../decisions/0147-count-a-target-file-in-map-recall.md).

## What the metric is for

The scorer documents map file recall as "the number that decides whether an
agent is pointed at the right place". An entry reading
`sampled.py — BaseTimeSeries` with a target of `timeseries/core.py` points an
agent at `core.py` as surely as an entry whose own path it is. By the metric's
own definition, a target counts.

## Two sets, not one

The scorer keeps the files a map names in two sets:

```text
entry_files  = { display_path }                        ← an entry's own path
map_files    = { display_path } ∪ { target_display_path }
```

Map recall is computed against `map_files`. The target-only contribution is
`|reference ∩ map_files| − |reference ∩ entry_files|`, which cannot go negative
because `entry_files ⊆ map_files`.

Keeping `entry_files` is what makes the attribution possible. Folding targets
into one set would raise the headline while hiding where the rise came from,
which is the same failure this record exists to correct, moved one level up.

## Missing means unnamed

A reference file named only as a target is removed from `missing_files`. A file
the map points a reader at is not missing from the map.

## The version is part of the measurement

This changes what `map_file_recall` measures, so the report schema moves from
1.0 to 1.1. On the same build a 1.0 scorer reports 20 of 27 and a 1.1 scorer 21
of 27. Without the version those two figures are indistinguishable, and a
reader comparing them would attribute a measurement change to a product change.

A historical 1.0 report is not re-scored. It remains a correct statement of
what 1.0 measured.

## The report schema is governed

The recall report schema is defined only in the scorer and pinned by nothing
else in the repository. It is nonetheless treated as governed: it is the
instrument every recall claim in the decision record rests on, and a silent
change to its meaning is indistinguishable from a change in the product. Any
future change to what it measures moves the version and carries a record.

## Preserved invariants

The scorer still links no engine crate and calls no model; its oracle-isolation
test is unchanged. It reads only output the product already wrote, and the
reference patch never reaches selection.
