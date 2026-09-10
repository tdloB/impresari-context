# Recall Report Target Attribution PRD

## Document Control

- PRD ID/version: IC-RRS-147 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Recall Report Target Attribution ARD](../architecture/recall-report-target-attribution-ard.md).
- Governing decision:
  [ADR-0147](../decisions/0147-count-a-target-file-in-map-recall.md).
- Follows: [ADR-0146](../decisions/0146-name-the-file-a-relationship-points-into.md).

## Problem

The project's own recall metric could not see the improvement it existed to
measure.

ADR-0146 made a map entry name the file its relationship resolves into, in a
`target_display_path` field. `impresari-context-recall-score` read only an
entry's `display_path`, so an entry pointing a reader at the reference file
through its target counted for nothing. On current `main` the scorer reported
20 of 27 reference files named where the map in fact named 21.

A metric that silently fails to count what a change adds cannot tell anyone
whether the change worked.

## Product Outcome

Map recall counts every file the map names, whether as an entry's own path or
as the file a relationship points into, and reports how much of the total came
from each so the source of a gain is visible.

## Functional Requirements

1. A `target_display_path` counts toward whether the map names a reference file.
2. A reference file named only as a target is not reported as missing.
3. The number of reference files named only through a target is reported per
   case and in total, separately from the headline.
4. A change to what the report measures moves the report schema version, so a
   report is never compared against one measuring something different.

## Acceptance Criteria

- On the twenty-two-task astropy corpus, current `main` scores 21 of 27, with
  one reference file attributed to a target.
- A test proves a reference file named only as a target counts and is
  attributed, and fails when target counting is disabled.
- The report schema is 1.1, and nothing outside the scorer depends on 1.0.
- The full repository gate passes.

## Non-Goals

- Changing what the product delivers. This changes only how it is measured.
- Re-scoring historical reports. A 1.0 report remains a valid 1.0 measurement.
