# Changed-Line Coverage PRD

## Document Control

- PRD ID/version: IC-CLC-156 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-12.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Changed-Line Coverage ARD](../architecture/changed-line-coverage-ard.md).
- Governing decision:
  [ADR-0156](../decisions/0156-score-changed-line-coverage-of-the-opening-evidence.md).
- Follows: [ADR-0147](../decisions/0147-count-a-target-file-in-map-recall.md).

## Problem

The recall report credits evidence with a file when any excerpt of it is
delivered, including a license header. Whether the evidence held the lines a fix
changes was measured only by a script outside the repository.

## Product Outcome

Every offline recall report states how much of the code an accepted change
touches was inside the opening evidence, at no model or network cost.

## Functional Requirements

1. For each case, count the reference change's removed lines and, for each
   insertion, the context line before it.
2. Count those lines found, stripped, inside an evidence excerpt from the same
   file.
3. Report per-case counts, totals and a floored percentage under report schema
   1.2, with every 1.1 field unchanged.
4. Stay offline and read only the product's written output.

## Acceptance Criteria

- On the twenty-two-task astropy corpus, the report reproduces the
  out-of-repository script's totals for `main` (40/213) and for the ADR-0155
  candidate (85/213), and the 1.1 totals are unchanged.
- Tests prove removed lines and insertion anchors are counted, and that a line
  delivered only from another file is not.
- The SBOM and lockfile are unchanged, and the full repository gate passes.

## Non-Goals

- Matching lines by position against the base revision.
- Changing any product behaviour.
