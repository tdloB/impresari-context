# Package-Proximate Nomination Reach — Architecture Requirements and Design

- ARD ID/version: IC-PPNR-ARD-130 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-04.
- Governing PRD: [IC-PPNR-130](../product/package-proximate-nomination-reach-prd.md).
- Decision:
  [ADR-0130](../decisions/0130-reach-by-package-proximity-not-import-following.md).

## The measurement that chose the design

The design question was which edge to follow out of a nominated file. Three
candidates were measured offline against the sixteen reference files that map
recall currently misses, using the real corpus and the real nomination ranking:

```text
                        reference files reached (of 16)
import edge             ▓ 1
same package            ▓▓▓▓▓▓▓▓▓▓ 10
either                  ▓▓▓▓▓▓▓▓▓▓ 10
```

Import following adds nothing that package proximity does not already cover.
The intuition behind it — a change ripples along the dependency graph — is not
what these tasks look like. A change lands in `io/fits/card.py` and its
neighbour `io/fits/header.py`, in four files under `coordinates/builtin_frames/`,
in `io/ascii/html.py` beside `io/ascii/rst.py`. The unit of change is the
package, and the report happens to name one member of it.

## Anchor depth

Expanding from more than the top-ranked file was measured too:

| anchor | reference files reached | files added (mean) |
| --- | --- | --- |
| top 1 | 8 of 16 | 10 |
| top 2 | 9 of 16 | 13 |
| top 3 | 9 of 16 | 23 |

The second anchor buys one file for three; the third buys nothing for ten. The
scoped extraction budget is a fixed allowance divided across the scope, so every
added file thins the ones already there. Top-1 is the knee, and it is where this
lands.

## Where reach sits

```text
task signals
     │
     ▼
exact task paths ──┐
                   ├──▶ direct nominations (ranked, ≤16)
identifier index ──┘            │
                                ├──▶ anchor = best direct nomination
                                │         │
                                │         ▼
                                │    package siblings (≤ reach ceiling)
                                ▼         │
                          ordered scope ◀─┘   reach strictly last
                                │
                                ▼
                          dense scoped graph
```

Reach runs after ranking, never inside it. It cannot promote a file over a
directly nominated one, and it cannot displace one: the direct list is complete
before the anchor is read.

## Ordering is a budget decision

`MAX_SCOPED_FACTS` is a whole-scope allowance. Reach files come last in seed
order, and extraction spends the allowance in two passes: nominated files divide
the whole allowance between themselves, and reach files divide only what
survives.

Both halves are needed, and the second was learned by measurement. Seed ordering
alone left extraction dividing the allowance equally across the scope, so
admitting twelve siblings cut every nominated file's share from 1,750 facts to
1,000. Measured, that cost `astropy-13236` its reference file — `table/table.py`
no longer fit its share — for a net map file recall of 10 of 27 against a
baseline of 11.

Equal division makes reach a trade. The tiered pass makes it an addition: a file
the product inferred cannot take density from a file the task named, which is
the property that makes reach safe to add at all.

Without it, admitting speculative files starves the file the task pointed at
directly — the exact failure
[ADR-0128](../decisions/0128-extract-structure-for-nominated-files-not-whole-repositories.md)
was written to end.

## A package is the immediate parent directory

Not a language package declaration, not an `__init__.py` boundary, not a
build-system module. The immediate parent directory is what was measured, it
needs no per-language resolver, and it holds for every admitted language without
a table of exceptions.

The rule is deliberately dumb. A smarter definition would need a resolver per
language, and the measurement says the dumb one already captures the effect.

## Bounds

| bound | why |
| --- | --- |
| reach files admitted | a pathological directory cannot flood the scope |
| anchor count (one) | measured knee; more anchors dilute without reaching |
| recursion depth (none) | siblings only; transitive reach is unmeasured |

The reach ceiling is separate from `MAX_NOMINATED_FILES` so the two cannot be
confused in a receipt, and so raising one never silently raises the other.

### The ceiling admits a package whole

The ceiling is a guard, not a selector. Truncation orders by path, and a package
is not alphabetical by relevance, so a tight ceiling discards the neighbourhood
it was asked to admit.

Measured on the corpus, packages holding a missed reference file carry 3 to 28
admitted files, and the missed file sits at alphabetical rank 1 to 23 within its
package:

| task | missed reference file | package | rank |
| --- | --- | --- | --- |
| 13236 | `table/table.py` | 21 | 20 |
| 13398 | `builtin_frames/itrs.py` | 28 | 23 |
| 14182 | `io/ascii/rst.py` | 20 | 17 |
| 14508 | `io/fits/card.py` | 12 | 2 |

A ceiling of twelve cut the first three — the exact files reach exists to admit.
Thirty-two clears every package observed.

Widening a ceiling is normally a density risk. Here it is not, because the
tiered allowance already prevents reach from spending a nominated file's facts.
The two changes are only safe together.

## Closed constants

The anchor rule, the package rule, and the reach ceiling are compile-time
constants, never caller-supplied. A consumer able to widen reach could walk the
scope toward a file it already wanted, which is oracle authority arriving
through a configuration field — the case `SEC-INV-012` exists to refuse.

## Disclosure

A reach file carries `package_proximate_reach` as its reason code, and any
nomination containing one carries
`structural_scope_includes_reach_expanded_files` among its unknowns. The two
together let a consumer separate what the task asked for from what the product
guessed, which is the honesty requirement that governs every other partial
result in this system.

## What this does not promise

Reachability is a ceiling, not recall. Admitting a file into the scope makes its
declarations available to the map; it does not put them in the map. The PRD's
acceptance criteria therefore measure map file recall on the corpus rather than
reachability, and require two files of improvement — one is inside the noise of
a twenty-two task sample.

## Preserved invariants

`SEC-INV-002`, `SEC-INV-003`, `SEC-INV-007`, `SEC-INV-011`, and `SEC-INV-012`
hold unchanged. Reach reads the snapshot's existing path inventory, performs no
additional repository read, writes nothing, executes nothing, and retains no
source content.
