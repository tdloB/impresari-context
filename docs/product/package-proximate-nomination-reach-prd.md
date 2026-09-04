# Package-Proximate Nomination Reach PRD

## Document Control

- PRD ID/version: IC-PPNR-130 / 1.0.
- Status: Implemented; acceptance criteria not met, held unmerged.
- Date: 2026-09-04.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Package-Proximate Nomination Reach ARD](../architecture/package-proximate-nomination-reach-ard.md).
- Governing decision:
  [ADR-0130](../decisions/0130-reach-by-package-proximity-not-import-following.md).
- Governing objective: [CLAUDE.md](../../CLAUDE.md) — 98% quality at 78% compression.

## Problem

Nomination admits only files the task itself points at: a path the task writes
out, or a file holding an identifier the task names. Measured map file recall
now stands at 11 of 27 reference files across twenty-two accepted changes.

Fifteen of the sixteen misses are files the task report never names. No amount
of better ranking over task signals reaches them, because the signal is absent.
The remaining reach has to come from the repository's own structure, not from
the report.

The obvious structural move is to follow imports and inheritance from a
nominated file. Measured against the sixteen missed files, that move is worth
**one**. Files in the same package as a nominated file account for **ten**.

| candidate reach mechanism | reference files reached |
| --- | --- |
| import edges from a nominated file | 1 of 16 |
| same package as a nominated file | 10 of 16 |
| either | 10 of 16 |

Import following is not a cheaper version of the right answer. It is close to
no answer, and it would have cost a resolved import graph to learn that.

## Product Outcome

Nomination admits, in addition to the files the task names, a bounded set of
files sitting in the same package as its best candidate — because a change that
touches one module usually touches its neighbours, and the task report names
only one of them.

Reach-expanded files are disclosed distinctly from directly nominated ones. A
consumer can always tell which files the task pointed at and which the product
inferred.

## Functional Requirements

1. Derive reach from exactly one anchor: the highest-ranked directly nominated
   file. Measured, sibling expansion from the top-ranked file reaches 8 of 16
   missed files while adding about 10 files; extending the anchor to the top
   three reaches 9 and adds about 23. The extra reach does not pay for the
   dilution.
2. Define a package as the anchor's immediate parent directory. The rule is
   language-neutral and holds for every admitted language.
3. Bound reach with a closed maximum reach-file count, separate from and
   additional to `MAX_NOMINATED_FILES`, set high enough to admit an ordinary
   package whole. Truncation orders by path, and a package is not alphabetical
   by relevance, so a tight bound discards the neighbourhood reach exists to
   admit. Reaching the bound is recorded, never silent.
4. Admit a reach file only when the snapshot tracks it and it is not already
   nominated. Reach never displaces a directly nominated file.
5. Order reach files strictly after every directly nominated file, in seed
   order *and* in extraction. Nominated files divide the whole fact allowance
   between themselves; reach files divide only what survives. Dividing it
   equally across the scope makes reach a trade rather than an addition, and
   measurably costs a reference file.
6. Mark every reach file with its own reason code, distinct from the direct
   nomination reasons, and record an explicit unknown whenever a nomination
   contains reach-expanded files.
7. Keep nomination deterministic. An identical snapshot and task yield an
   identical nomination, reach included, in a stable order.
8. Neither the anchor choice, the package rule, nor the bound may be
   caller-supplied. A consumer able to widen reach could steer selection, and
   steering is oracle authority (`SEC-INV-012`).
9. Never consult a reference change, accepted patch, or test outcome.

## Acceptance Criteria

- Reach expands from the top-ranked nominated file's directory only.
- A nomination with no direct candidates produces no reach; reach has no anchor
  and admits nothing.
- The reach bound is enforced and its breach recorded as an explicit unknown.
- Every reach file carries the reach reason code, and every nomination
  containing one carries the reach unknown.
- Reach files never appear before a directly nominated file, and admitting them
  leaves every nominated file's fact share exactly as it was before reach
  existed.
- Repeating nomination over identical inputs yields an identical result.
- A static check proves the module reaches no oracle, execution, or network
  surface.
- **Measured, offline, over all twenty-two astropy tasks before this is claimed
  to work:** map file recall improves by at least two reference files against
  the 11 of 27 baseline, and no reference file currently recalled is lost. One
  file is noise; two is a result.
- The full repository gate passes.

## Non-Goals

- Import, inheritance, or call-graph following. Measured at 1 of 16, it does not
  earn the resolver it would require. [ADR-0130](../decisions/0130-reach-by-package-proximity-not-import-following.md)
  records the measurement so the idea is not re-proposed from intuition.
- Recursive reach. Siblings of siblings are not admitted.
- Multi-anchor reach. Measured and rejected at this ceiling; revisit only
  against a measurement, not an argument.
- Raising `MAX_NOMINATED_FILES`. The slot curve is flat from 16 to 64; the
  missing files are not further down the same ranking.
