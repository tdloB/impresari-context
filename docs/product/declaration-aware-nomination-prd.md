# Declaration-Aware Nomination PRD

## Document Control

- PRD ID/version: IC-DAN-131 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-04.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Declaration-Aware Nomination ARD](../architecture/declaration-aware-nomination-ard.md).
- Governing decision:
  [ADR-0131](../decisions/0131-admit-and-rank-identifiers-by-declaration.md).
- Governing objective: [CLAUDE.md](../../CLAUDE.md) — 98% quality at 78% compression.

## Problem

Nomination admits a task token as a code identifier on its **shape**: an
interior separator, or a lowercase letter followed by an uppercase one. The rule
exists to keep prose out, and
[ADR-0124](../decisions/0124-classify-task-signals-by-code-shape-not-token-position.md)
is right that token position cannot do that job.

Shape cannot do it either. The rule admits `TimeSeries`, `SkyCoord`,
`_required_columns` and `__array_ufunc__`, and rejects every single-word name:

| token | admitted today |
| --- | --- |
| `TimeSeries`, `SkyCoord`, `ValueError`, `_required_columns` | yes |
| `Header`, `Card`, `Quantity`, `Table`, `Column`, `WCS`, `HDUList`, `ndarray` | **no** |

Single-word class names are the most common shape a Python class takes, and they
are invisible to nomination.

Measured, this is the largest single failure mode. Three of the eleven tasks that
miss their reference file return **no map at all** — `structural_seed_unavailable`,
because the task yielded no admitted identifier. In every one of the three, the
rejected word names a class declared in exactly the reference file:

| task | rejected word | declared in | is a reference file |
| --- | --- | --- | --- |
| `astropy-8707` | `Header` | `astropy/io/fits/header.py` | yes |
| `astropy-8872` | `Quantity` | `astropy/units/quantity.py` | yes |
| `astropy-14598` | `Card` | `astropy/io/fits/card.py` | yes |

It also explains why every earlier attempt to buy recall with more identifier
capacity failed. Raising the identifier ceiling from 16 to 64 changed nomination
recall not at all, and freeing slots by rejecting version-shaped noise changed
it not at all. Neither could help: the identifiers that mattered were never
admitted at any ceiling.

A second, related defect sits beside it. Nomination ranks a file by **how many**
task identifiers it contains, and counts a passing mention exactly as it counts
a definition. `Header` appears in hundreds of astropy files and is *declared* in
one. Ranking that treats those alike picks the wrong anchor, and every later
stage — seeding, traversal, and package reach — inherits that choice.

## Product Outcome

The repository decides what looks like code. A task token is admitted as an
identifier when the repository declares something by that name, whichever shape
the name takes — and a file that *declares* a task identifier outranks a file
that merely mentions it.

## Functional Requirements

1. Record, for each admitted file at preparation, the names it declares, in
   addition to the identifiers it contains. Reading stays one pass per file.
2. Detect a declaration lexically: a declaration keyword at the start of a line,
   after optional modifiers, followed by a name. No parser, no per-language
   resolver, and no second read.
3. Admit a task token as a code identifier when it passes the existing shape
   rule **or** the index records it as declared somewhere in the snapshot. The
   shape rule is unchanged and keeps working when no index is present.
4. Rank a file that declares a task identifier above a file that only mentions
   one. Ties inside each ground break as they do today, by identifier count and
   then by path, so nomination stays deterministic.
5. Bound declarations explicitly: a maximum retained per file, and the existing
   identifier length ceiling. Any breach is recorded, never silent.
6. Bind declarations to the same snapshot as the index and refuse to answer for
   another.
7. Retain names and portable paths only. No source bytes, spans, excerpts, or
   line numbers.
8. Widening admission must not widen authority. A token admitted because the
   repository declares it is repository-derived, never consumer-derived.
9. Never consult a reference change, accepted patch, or test outcome.

## Acceptance Criteria

- `Header`, `Card`, `Quantity`, `Table`, `WCS` and `HDUList` are admitted when
  the snapshot declares them, and are not admitted when it does not.
- A prose word the repository never declares is still rejected, including the
  capitalised prose the shape rule was written to exclude.
- A file declaring a task identifier is nominated above one that mentions it.
- Declaration lookup performs zero repository reads, proven by construction.
- Bounds are enforced and breaches recorded; a stale snapshot is refused.
- A static check proves the module reaches no oracle, execution, or network
  surface and retains no source bytes.
- **Measured, offline, over all twenty-two astropy tasks:** map file recall
  improves by at least two reference files against the 11 of 27 baseline, and no
  reference file currently recalled is lost.
- Every task that returns `structural_seed_unavailable` today returns a map.
- The full repository gate passes.

## Non-Goals

- Replacing the shape rule. It is the fallback whenever no index is available
  and it stays exactly as it is.
- Parsing. Declaration detection is lexical; the structural worker remains the
  only parser.
- C-family method declarations. A method that declares a return type rather than
  a keyword — `public static void main(...)` — yields no name. Type
  declarations are covered in every admitted language; method declarations only
  where a keyword introduces them.
- Semantic or fuzzy matching. Exact names only.
- Resolving which declaration a task means when several files declare one name.
  Ranking discloses the ambiguity; it does not resolve it.
