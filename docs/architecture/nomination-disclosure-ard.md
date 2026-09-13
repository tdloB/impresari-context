# Nomination Disclosure — Architecture Requirements and Design

- ARD ID/version: IC-ND-ARD-142 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-07.
- Governing PRD: [IC-ND-142](../product/nomination-disclosure-prd.md).
- Decision:
  [ADR-0142](../decisions/0142-disclose-the-files-a-scoped-graph-was-built-over.md).

## What already exists

`build_task_scoped_structure` returns `(StructuralGraph, FileNomination)`, and
`FileNomination` is already the disclosure the PRD asks for: schema-versioned,
carrying each admitted file with its reason code and identifier count, the
number of candidates considered, and its own shortfall reasons.

The MCP server takes that value, projects two fields out of it — the file order
and the admitted identifiers — and drops the rest on the floor:

```text
build_task_scoped_structure ──▶ (graph, nomination)
                                        │
                          order, admitted_identifiers
                                        │
                                        ▼
                              seed selection, scope
                                        │
                                        ▼
                                      map          ← nomination gone
```

The change carries the value one step further:

```text
build_task_scoped_structure ──▶ (graph, nomination)
                                        │
                                        ├──▶ order, identifiers ──▶ seeding
                                        │
                                        └──▶ disclosure_map.scope
```

Nothing is computed that was not computed before.

## Why this is disclosure and not capability

The nomination is **derived** from the task text and the snapshot, both of which
the consumer already supplied. It is emitted, never read back: no field of it
reaches nomination, seeding, or traversal on any later request. `SEC-INV-012`
holds because there is no path from this output to any input.

Every disclosed path is one the graph was built over, and so one the map may
already name in an item. The disclosure widens no view. The evaluator filters
disclosed paths against its own frozen allowlist for the same reason it filters
map items, and keeps the product's own counts, so a filtered path shows as a gap
between `nominated_files` and the list rather than vanishing.

`SEC-INV-002`, `SEC-INV-003`, `SEC-INV-007` and `SEC-INV-011` are untouched: no
source is read, written, executed, or excerpted.

## Absence must not be ambiguous

A whole-repository fallback graph nominates nothing. Omitting the field there
would make "unscoped" and "an older build that discloses nothing" identical to a
consumer, which is the confusion this record exists to remove. The unscoped case
therefore emits the field with `scoped_to_nominated_files: false`, an empty file
list, and `structural_scope_whole_repository`.

## Cost

One serialization of a value already in memory, bounded by
`MAX_NOMINATED_FILES = 16` entries. No read, no traversal, no store access. On
the measured astropy task the disclosure is under 1.5 KiB against a 16 KiB
packet ceiling, and the packet is budget-bound on evidence rather than on the
map.

## What it makes measurable

Nomination recall — whether the files an accepted change touches were nominated
at all — becomes an offline comparison between the disclosed list and a
reference change, performed entirely outside the product. The ARD for
IC-SSSE-128 names it the leading indicator that bounds map recall from above.
Its first measurement retired nomination as the suspect behind zero map recall
on the task it was run against.
