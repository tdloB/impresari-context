# Target File Disclosure PRD

## Document Control

- PRD ID/version: IC-TFD-146 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Target File Disclosure ARD](../architecture/target-file-disclosure-ard.md).
- Governing decision:
  [ADR-0146](../decisions/0146-name-the-file-a-relationship-points-into.md).
- Refines: [ADR-0145](../decisions/0145-spread-structural-seeds-across-files.md).

## Problem

A map entry names a symbol from one file while attributing it to another, and
never names the file that declares it.

`symbol_label` is taken from the relationship's **target** node.
`display_path` is taken from its **source**. So an entry reads
`sampled.py — BaseTimeSeries` when `BaseTimeSeries` is declared in
`timeseries/core.py`, and `core.py` is never mentioned. A consumer following
that entry is pointed at the wrong file.

The target node is resolved and already held in memory — the entry looks it up
to obtain the symbol name, then discards its path.

Measured over the twenty-two astropy tasks, of the nine reference files the map
fails to name, **three are already present as relationship targets**:

| task | file the accepted change touches |
| --- | --- |
| `astropy__astropy-13033` | `astropy/timeseries/core.py` |
| `astropy__astropy-13236` | `astropy/table/table.py` |
| `astropy__astropy-14995` | `astropy/nddata/mixins/ndarithmetic.py` |

## Product Outcome

A relationship that resolves into a different file names that file, so a
consumer can follow the entry to where the symbol actually lives.

## Functional Requirements

1. A map entry whose relationship resolves into a file other than its own
   names that file.
2. An entry whose relationship stays inside one file names no target file.
   Repeating the source path on every entry is noise.
3. An unresolved relationship names no target file.
4. The evaluator admits a disclosed target file only when the frozen source
   allowlist already admits it, exactly as it treats an entry's own path.
5. No additional traversal, repository read, or search is performed.

## Acceptance Criteria

- On the twenty-two-task corpus, map file recall improves and no task regresses.
- Delivered bytes rise only by the disclosed paths.
- A test proves a cross-file relationship names the target file, and fails
  without the change.
- A test proves a same-file relationship and an unresolved one name none.
- The full repository gate passes, including adversarial and fuzz suites.

## Non-Goals

- Reaching files the traversal never resolved. This discloses what is already
  held; it does not search further.
- Changing nomination, seeding, or traversal depth.
- Any provider request or paid evaluation.
