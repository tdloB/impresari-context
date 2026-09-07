# Seed-Scoped Structural Extraction PRD

## Document Control

- PRD ID/version: IC-SSSE-128 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-03.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Seed-Scoped Structural Extraction ARD](../architecture/seed-scoped-structural-extraction-ard.md).
- Governing decision:
  [ADR-0128](../decisions/0128-extract-structure-for-nominated-files-not-whole-repositories.md).
- Governing objective: [CLAUDE.md](../../CLAUDE.md) — 98% quality at 78% compression.

## Problem

Structural context is unusable on a repository of ordinary size, and no amount
of better selection can fix it, because the symbols to select are never
extracted.

Measured across twenty-two real astropy tasks on the full repository at the
production budget:

| metric | value |
| --- | --- |
| map file recall | 0 of 27 (0%) |
| map symbol recall | 0 of 34 (0%) |
| evidence file recall | 6 of 27 (22%) |
| tasks with a non-empty map | 1 of 22 |

Improving seed ranking changed none of those recall numbers. It raised
non-empty maps from 1 to 6 of 22 and disclosed ambiguity honestly, which is
better behaviour, but not one of those maps contained a file the accepted
change touches.

The cause is capacity, not selection. A serialized structural graph is capped
at 16 MiB by the local store, and one admitted fact costs roughly 530 bytes of
that graph. The whole repository therefore affords about 31,600 facts. Spread
across astropy's 1,172 supported files that is about **27 facts per file** at
the absolute ceiling, and about **one per file** at the production budget. A
Python module cannot expose its classes and methods in one fact.

The same product on a 47-file subtree, with the same task and budget, produces
about 213 facts per file, returns sixteen relevant items, and completes in 3.3
seconds against 20 seconds for the whole-repository pass. The architecture works
when the graph is dense. Density is what a whole-repository graph cannot have.

## Product Outcome

Stop extracting thin structure for every file. Nominate a bounded set of
candidate files from the task, extract structure densely for only those, and
build the graph over that set.

A few dozen files at full per-file density is a small, dense graph that fits the
store ceiling comfortably, seeds reliably, and costs a fraction of a
whole-repository pass.

## Functional Requirements

1. Nominate a bounded set of candidate files using only the task text and the
   current workspace snapshot. The maximum is a closed constant, never
   caller-supplied.
2. Rank nomination deterministically: an exact task path, then a file
   containing an exact task identifier, then remaining lexical matches. Break
   ties by portable path.
3. Extract structure for nominated files at the full admitted per-file fact
   allowance. Density is the point; a nominated file must not be thinned.
3a. Divide the allowance by what each file needs, not evenly. An even division
   cuts a file that needs more than its share while the rest of the build leaves
   its shares unspent — measured, one build used 12,040 of 28,000 facts and still
   truncated a file at exactly its 2,154 share
   ([ADR-0138](../decisions/0138-divide-a-fact-allowance-by-what-each-file-needs.md)).
3b. Take each file's need from the parser, which is the only thing that knows it.
   A response reports the facts its file would yield with no ceiling, so the
   division is exact rather than estimated. Estimating it from file size would
   be a model of the parser living outside the parser, and source bytes per fact
   measures across a 1.9× range.
3c. Never reduce a file below what an earlier pass already gave it, and never
   place more than the unspent remainder. Redistribution must not be able to
   regress a build or exceed the allowance.
4. Build, seed, and traverse the graph over the nominated set only.
5. Reuse the existing content-hash-keyed per-file structural cache, so a file
   already extracted for an earlier task costs nothing to reuse.
6. Disclose scope explicitly. A consumer must be able to tell that structural
   coverage is limited to nominated files, how many were nominated, and why
   each was admitted. A scoped graph must never read as a whole-repository one.
7. When nothing is nominated, report explicit structural unavailability and
   continue ordinary retrieval. Never fail the request.
8. Never consult a reference change, accepted patch, or test outcome during
   nomination. Oracle isolation is a hard validity gate.
9. Report **nomination recall** — whether the files an accepted change touches
   were nominated at all — as an offline metric alongside map recall. It is the
   leading indicator: a file never nominated can never be mapped.

## Acceptance Criteria

- On the twenty-two-task astropy corpus, map file recall rises above the
  measured 0 of 27 baseline, and nomination recall is reported for every task.
- A scoped graph for a task nominating N files contains structure for those N
  files at full per-file density, and for no others.
- No nominated file is truncated by the worker's response ceiling at a density a
  real module needs. **Measured, ten astropy tasks, thirty-nine nominated
  files:** raising the requested response from 1 MiB to 4 MiB takes truncation
  from 12 files to 4 and the structure recovered from 52,293 to 70,401 facts
  (+34.6%), with no measurable change in build time and none in peak memory.
  **Map recall does not move** — 19 of 27 files and 13 of 34 symbols at both
  ceilings across all twenty-two astropy tasks — so the extra structure is
  reported as extra structure and not as a better map
  ([ADR-0139](../decisions/0139-raise-the-structural-worker-response-ceiling.md)).
- No nominated file is truncated by the fact allowance while that allowance goes
  unspent. **Measured on ten astropy tasks, thirty-nine nominated files:** fact-
  quota truncation falls from one file to none, and with the response byte
  ceiling also lifted, truncation across the corpus falls from twelve files to
  **zero** and the structure recovered rises 26%. At the shipped byte ceiling
  this requirement alone recovers 615 facts, because the byte ceiling binds first
  — that is reported rather than presented as a larger win.
- Whole-repository structural preparation is no longer required before a first
  request, and initialization time falls accordingly.
- Repeating a task with an unchanged snapshot reuses cached per-file structure
  and yields an identical map identity.
- A static check proves nomination has no path to reference-change data.
- Every scoped result carries its scope disclosure; a test proves a scoped
  graph cannot be mistaken for a complete one.
- The full repository gate passes, including adversarial and fuzz suites.

## Non-Goals

- Model-assisted nomination. Nomination stays lexical and structural.
- Raising the store's graph ceiling. This work removes the pressure on it
  rather than arguing with it.
- Cross-file dataflow, inheritance edges, or test-to-implementation
  association. Each needs its own record.
- Any provider request or paid evaluation.
