# ADR-0139: Raise the Structural Worker Response Ceiling to 4 MiB

- Status: Accepted
- Date: 2026-09-06
- Related PRD: [Seed-Scoped Structural Extraction](../product/seed-scoped-structural-extraction-prd.md)
- Architecture: [Seed-Scoped Structural Extraction](../architecture/seed-scoped-structural-extraction-ard.md)
- Also serves: [Host Read Substitution](../product/host-read-substitution-prd.md), [ADR-0137](0137-answer-the-symbol-a-map-names-not-the-path-it-sits-in.md)
- Refines: [ADR-0010](0010-structural-worker-protocol-and-isolation.md), [ADR-0128](0128-extract-structure-for-nominated-files-not-whole-repositories.md)

## Context

The structural worker bounds its response by the admitted budget's `requested`
field, and over that bound it returns a *prefix* of the fact list rather than
failing. The server has always requested 1 MiB.

At a measured 380 bytes per fact in a worker response, 1 MiB buys about **2,764
facts**. Real modules want more than that. Measured fact demand for eight astropy
modules against that threshold:

| file | facts wanted | fits in 1 MiB? |
| --- | --- | --- |
| `table/table.py` | 6,146 | no |
| `io/fits/header.py` | 3,734 | no |
| `units/quantity.py` | 3,489 | no |
| `io/ascii/core.py` | 3,381 | no |
| `io/fits/card.py` | 2,693 | yes |
| `coordinates/sky_coordinate.py` | 2,210 | yes |
| `timeseries/sampled.py` | 604 | yes |
| `timeseries/core.py` | 188 | yes |

Every file above the threshold lost declarations; every file below lost none.
The split is exactly at 2,764, which is what makes this a ceiling rather than a
tuning question.

The visible cost is a graph that stops partway through a file. On
`astropy/io/fits/header.py` every declaration after roughly line 1,920 was
missing, including seven whole top-level classes — from a graph that reported no
shortfall of its own until [ADR-0137](0137-answer-the-symbol-a-map-names-not-the-path-it-sits-in.md)
made it say so.

Across ten astropy tasks and thirty-nine nominated files, this ceiling was the
binding constraint on **11 of the 12 truncated files**. The fact allowance
([ADR-0138](0138-divide-a-fact-allowance-by-what-each-file-needs.md)) accounted
for the twelfth.

Nothing external requires 1 MiB. The chain above it is self-imposed: the store
caps a serialized graph at 16 MiB because this project calls
`set_limit(SQLITE_LIMIT_LENGTH, 16 MiB)`, and SQLite's own default for that limit
is 1 GB. The transport permits a 16 MiB response frame. 4 MiB is the largest a
`ResourceBudget::conservative` admits without widening a validated range, so it
is the largest step available without a contract change.

## Decision

Request 4 MiB rather than 1 MiB for every structural build this server performs.

Do not widen the budget contract's `requested` range, do not raise the 16 MiB
store cap, and do not change `MAX_RESPONSE_BYTES`. 4 MiB was sufficient for every
file in the measured corpus, so the larger changes are not justified by evidence
and are not made.

## Consequences

### What it buys: every symbol a map names becomes answerable

ADR-0137 answers a host's read offer with one named declaration, and reports
`symbol_not_declared_in_path` when the graph does not hold it. Off a truncated
graph that answer is a false negative, and a host that trusts it skips the read
and loses the declaration.

Measured over 494 declarations in eight astropy modules:

| | 1 MiB | 4 MiB |
| --- | --- | --- |
| symbols answerable | 378 of 494 (76.5%) | **494 of 494 (100%)** |
| median answer size | 0.8% of file | 0.7% of file |

Every one of the 116 unanswerable symbols was in a file whose graph this ceiling
truncated. Raising it removes the false-negative class entirely on this corpus.
That is the reason to make this change.

### What it does not buy: map recall

Measured over all twenty-two astropy SWE-bench tasks, each at its own base
commit, scoring the disclosure map against the accepted patch:

| | 1 MiB | 4 MiB |
| --- | --- | --- |
| map file recall | 19 of 27 | **19 of 27** |
| map symbol recall | 13 of 34 | **13 of 34** |
| non-empty maps | 22 of 22 | 22 of 22 |

**Identical. Not one task moved.** Recovering 34.6% more structure changed no
map. Whatever binds map recall at 19 of 27, per-file structural density is not
it — which is the same shape of result as ADR-0125, where better seed ranking
moved no recall number, and it is recorded here so density is not proposed again
from intuition.

### Cost

Measured over ten astropy tasks, thirty-nine nominated files, on `main`:

| | 1 MiB | 4 MiB |
| --- | --- | --- |
| files truncated | 12 (30.8%) | **4 (10.3%)** |
| structure recovered | 52,293 facts | **70,401 facts (+34.6%)** |
| map build wall clock | 32,817 / 30,668 ms | 31,950 / 31,493 ms |
| peak resident memory | 407.4 MB | **407.3 MB** |
| cache on disk, 7 builds | 109.9 MB | 125.6 MB (+14.3%) |

**There is no measurable latency cost.** Two runs of each configuration put the
within-configuration spread (2,149 ms) above the between-configuration
difference, so the honest statement is that the change does not move build time,
not that it improves it.

**There is no memory cost.** Peak resident memory is identical to within 0.1 MB.
The response buffer is transient and per file; the peak is set by the
whole-repository startup graph, which this does not touch.

**The cost is disk.** About 2.2 MB more cache per scoped build, 14% over the
measured corpus, in a cache that already exists and is content-hash keyed.

The four files still truncated at 4 MiB are truncated by the fact allowance, not
by this ceiling; ADR-0138 addresses those, and with both changes the corpus has
no truncated file.

No security invariant changes. A larger admitted response is still bounded, still
validated against the same contract, and still parsed from source the caller
authorized. This grants no execution, network, publication, or submission
authority.

### Confirmed through the harness, not only offline

When this was first written the benefit was an offline property of an answer: no
agent could take a substitution offer, so the ceiling's value could not be
observed. The evaluation harness now holds a product session open and follows
the pointers a map emits, and the ceiling shows up there exactly as measured
here.

Following every pointer a map emitted over the same astropy subset:

| response ceiling | returned of what the named files weigh | truncated answers |
| --- | --- | --- |
| 1 MiB | 288,249 of 1,197,340 — 24% | **6** |
| 4 MiB | 317,708 of 2,799,300 — 11% | **0** |

At 1 MiB six offers come back disclosing a truncated graph, and a host taking
those answers has to read the file whole — losing the saving precisely on the
large files where it is worth most. At 4 MiB none do.

That is the evidence this record said it wanted before the change was worth
merging, and it is why it is no longer held.

## Not claimed

A better map. Map recall is measured above and does not move at either ceiling.
A token saving is still unmeasured: the harness can now put the offer in front
of an agent, but whether an agent given cheaper reads spends fewer tokens is a
graded run that has not been made.
