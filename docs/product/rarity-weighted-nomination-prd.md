# Rarity-Weighted Nomination PRD

## Document Control

- PRD ID/version: IC-RWN-149 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Rarity-Weighted Nomination ARD](../architecture/rarity-weighted-nomination-ard.md).
- Governing decision:
  [ADR-0149](../decisions/0149-weight-nomination-by-identifier-rarity.md).
- Refines: [ADR-0131](../decisions/0131-admit-and-rank-identifiers-by-declaration.md).

## Problem

Nomination chooses up to sixteen files for dense structural extraction, and a
file it does not choose cannot appear in the structural map. Two gaps kept
reference files out.

Every task identifier counted the same, although a name held by one file
identifies it and a name held by thirty identifies none of them. On
`astropy-13398` the file declaring `ITRS`, the only file that does, placed
eighteenth behind files mentioning common names and was not nominated.

A module named the way code imports it was dropped. `astropy-14182` writes
`format="ascii.rst"`. The planner treats `ascii.rst` as a path, the snapshot
holds no such path, and `astropy/io/ascii/rst.py` was not nominated.

## Product Outcome

Nomination prefers the files holding the rare names a task uses, and nominates
the file a dotted module name denotes.

## Functional Requirements

1. Each task identifier is weighted by how rare it is in the repository, as an
   integer, so the ranking is identical on every platform.
2. A declaration still counts three times a mention of the same name.
3. A dotted task name the snapshot holds no path for nominates the one tracked
   file it denotes, read as a path suffix with the extension removed, under the
   reason `task_module_path`.
4. A dotted name that is also a directory, or that several paths end in,
   nominates nothing.
5. The nomination disclosure schema moves to 1.1.
6. Nomination stays bounded at sixteen files, deterministic, and free of oracle,
   execution and network authority.

## Acceptance Criteria

- On the twenty-two-task astropy corpus, nomination recall is 24 of 27 against
  22 of 27 on `main`, map file recall is 22 of 27 against 21, and no task loses
  a reference file.
- Tests prove that a rare declared name outranks files mentioning common ones,
  that a dotted module name nominates the file it denotes, and that a package
  name or an ambiguous name nominates nothing. The first two fail with their
  mechanism disabled.
- The full repository gate passes.

## Non-Goals

- Changing the sixteen-file ceiling or the sixteen task-signal slots.
- Changing which task tokens are admitted as identifiers.
- Changing packet evidence.
