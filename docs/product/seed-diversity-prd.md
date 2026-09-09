# Seed Diversity PRD

## Document Control

- PRD ID/version: IC-SD-145 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-08.
- Product owner: Aaron Boldt.
- Governing architecture: [Seed Diversity ARD](../architecture/seed-diversity-ard.md).
- Governing decision: [ADR-0145](../decisions/0145-spread-structural-seeds-across-files.md).
- Refines: [ADR-0125](../decisions/0125-select-ranked-seed-sets-and-traverse-to-definitions.md).

## Problem

A structural map names too few files, and which files it names is decided
almost entirely by one file.

A map item's path is its edge's **source** node's path, and at a traversal
depth of one only edges leaving a seed are selected. A map can therefore only
name a file a seed landed in. Seed candidates are ranked and tie-broken by
nomination position with no per-file rule, so the best-nominated file takes
every slot it can fill.

Measured over twenty-two astropy tasks, that left a median of four files named
per task, and on the most degenerate case a single file across forty-six map
items — twenty-three of which were the same entry repeated. Map file recall sat
at 18 of 27.

Eight seeds inside one module do not buy eight modules' worth of reach. They
traverse overlapping edges of the same file and return the same relationships
several times.

## Product Outcome

Seeds spread across the best-nominated files instead of concentrating in one,
so a map names more of the places a task might need.

## Functional Requirements

1. No single file contributes more than a closed maximum of the seed set.
2. Admission preserves rank. A file's surplus candidate is skipped, never
   promoted ahead of a better-ranked candidate elsewhere.
3. The per-file maximum is a closed constant, never caller-supplied, chosen
   against measured symbol recall and not only file recall.
4. A task whose candidates all lie in one file still seeds, up to that maximum.
5. Seed-limit disclosure continues to report that the ceiling turned candidates
   away, independent of the per-file rule.
6. Delivered bytes do not increase.

## Acceptance Criteria

- On the twenty-two-task corpus, map file recall improves and no task regresses.
- Delivered bytes are unchanged.
- Distinct map items rise while total map items do not.
- Map symbol recall does not regress. Spreading seeds must not trade symbols
  inside a file for files.
- A test proves one file cannot take every slot, and fails without the rule.
- A test proves admission never promotes a lower-ranked candidate.
- A test proves a single-file task still seeds.
- The full repository gate passes, including adversarial and fuzz suites.

## Non-Goals

- Changing nomination, traversal depth, or the seed ceiling itself.
- Changing what a map item contains or how it is rendered.
- Any provider request or paid evaluation.
