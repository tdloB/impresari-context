# ADR-0138: Divide a Fact Allowance by What Each File Needs

- Status: Accepted
- Date: 2026-09-06
- Related PRD: [Seed-Scoped Structural Extraction](../product/seed-scoped-structural-extraction-prd.md)
- Architecture: [Seed-Scoped Structural Extraction](../architecture/seed-scoped-structural-extraction-ard.md)
- Amends: [ADR-0128](0128-extract-structure-for-nominated-files-not-whole-repositories.md)

> ADR-0137 is a read-substitution record in flight on a separate branch; this
> record is independent of it and takes the next free number.

## Context

ADR-0128 established that density comes from scoping, and set a scoped allowance
of 28,000 facts shared across the files a task nominates. It fixed the right
problem — a whole-repository graph gave each file about one fact and scored 0 of
27 on map file recall.

It left the division of that allowance even. Each file receives
`remaining ÷ files remaining`, walked in snapshot order, which is alphabetical
and unrelated to how much any file needs. Leftovers do flow forward, so a file
processed late can receive more than an equal share — but a file processed early
cannot, however much the rest of the build leaves unspent.

Measured on astropy, ten SWE-bench tasks, reading the engine's own cached worker
responses:

| task | nominated | facts used | of 28,000 | truncated |
| --- | --- | --- | --- | --- |
| 13033 | 13 | 12,040 | 43% | 1 file, cut at exactly 2,154 |

`28,000 ÷ 13 = 2,154`, and the truncated file stopped at precisely that. **The
build left 15,960 facts — 57% of its allowance — unclaimed while cutting a file
short.** The capacity was present, in the same build, unspent. That is an
allocation defect, not a capacity one.

Fixing it requires knowing what each file would have yielded, and only the parser
knows that. The worker parses the whole file either way — the tree is built
before any fact is emitted — and then abandons a partly-finished walk over that
tree when the ceiling is reached, learning the total and discarding it.

The alternative considered and rejected was estimating demand from file size.
Measured, source bytes per fact ranges from 18.6 to 35.7 across eight astropy
modules — a 1.9× spread, with real inversions: `sky_coordinate.py` and
`header.py` are within 1,200 bytes of each other and differ by 1.7× in facts. An
estimator like that is a model of the parser living outside the parser, and this
project has already drawn a wrong conclusion from exactly that pattern.

## Decision

Report what a file yields. A successful worker response carries
`total_facts_available`: the facts the file would produce with no ceiling,
counted by completing the walk while still emitting only what the ceiling
permits. Protocol version becomes `1.1.0`.

The fact ceiling therefore no longer stops the walk. It withholds facts, and the
response says so by reporting a total larger than the facts it carries. The
nesting-depth bound still stops the walk, because unbounded recursion is a safety
question rather than a ceiling; a walk stopped that way reports a floor and
discloses it.

Place the unspent allowance with the files that were cut. After the equal-share
pass, divide whatever remains among short files smallest-shortfall first, and
re-parse only those.

Do not take facts back from a file that already has them, and place only the
unspent remainder. A build therefore cannot regress, and the total cannot exceed
the allowance.

Do not raise the 28,000 allowance. No measured build came close to exhausting
it — the heaviest used 68% — so the total is not what binds.

## Consequences

Every nominated file in the measured corpus receives its complete structure.
Across seven builds that produced a scoped graph:

| configuration | files truncated (of 39) | truncation cause |
| --- | --- | --- |
| before | 12 | 11 response bytes, 1 fact quota |
| **this change** | 12 | **12 response bytes, 0 fact quota** |
| before, 4 MiB response ceiling | 4 | 4 fact quota |
| **this change, 4 MiB response ceiling** | **0** | — |

The fact-quota truncation this record removes is gone in both columns. What it
buys at the shipped 1 MiB response ceiling is small — 615 facts on one file,
about 0.9% across the corpus — because at that ceiling the response byte limit
binds first almost everywhere.

That is stated rather than smoothed over: **this change alone barely moves the
shipped configuration.** Its value appears when the response ceiling stops
binding, and then it is decisive — 17,595 additional facts across the corpus, a
26% increase, and no truncated file anywhere. The two ceilings are independent
and the second one is a separate decision with its own memory and latency
surface.

Re-parsing costs a second worker invocation per short file, four across ten
tasks in this sample, and only for files that a first pass cut. A file whose
first pass was sufficient is never parsed twice.

The approach does not reach the globally fair division when the first pass
already spent the allowance; with nothing unspent there is nothing to place. That
case did not appear in the corpus, where the heaviest build left a third of its
allowance unused, and the properties above are what make the change safe to apply
unconditionally rather than gated on measurement.

Reporting a per-file total is new information in the response, derived entirely
from source the worker was already given and already parsed. It grants no
execution, network, publication, or submission authority, and no security
invariant changes.

## Measurement honesty

An earlier reading of this data was wrong and is corrected here. A first
measurement put map truncation at 10.3% and attributed all of it to the fact
quota. That run used a release binary left over from an unrelated 4 MiB
experiment, so the response ceiling was four times its shipped value and byte
truncations were invisible. At the real 1 MiB ceiling, 11 of 12 truncations are
byte truncations. The baseline in the table above was rebuilt from `main`.
