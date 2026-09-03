# ADR-0128: Extract Structure for Nominated Files, Not Whole Repositories

- Status: Accepted
- Date: 2026-09-03
- Related PRD: [Seed-Scoped Structural Extraction](../product/seed-scoped-structural-extraction-prd.md)
- Architecture: [Seed-Scoped Structural Extraction](../architecture/seed-scoped-structural-extraction-ard.md)
- Refines: [ADR-0121](0121-use-bounded-progressive-structural-disclosure.md), [ADR-0125](0125-select-ranked-seed-sets-and-traverse-to-definitions.md)

## Context

Across twenty-two real astropy tasks on the full repository at the production
budget, map file recall was **0 of 27** and map symbol recall **0 of 34**.
Evidence file recall was 6 of 27. Only one task produced a non-empty map.

ADR-0125 improved seed selection and moved none of those recall numbers. It
raised non-empty maps from 1 to 6 of 22 and made ambiguity explicit, which is
better behaviour, but no map contained a file the accepted change touches. That
result is the useful one: it rules out selection as the binding constraint.

The binding constraint is capacity. The local store caps a serialized graph at
16 MiB and one admitted fact costs roughly 530 bytes of it, so a repository
affords about 31,600 facts in total. Across astropy's 1,172 supported files that
is about 27 facts per file at the ceiling and about one per file in production.
A module cannot expose its classes and methods in one fact.

The same product on a 47-file subtree, with the same task and budget, reaches
about 213 facts per file, returns sixteen relevant items, and finishes in 3.3
seconds rather than 20. The architecture is sound at density. A whole-repository
graph cannot be dense, because a fixed allowance divided by an unbounded file
count approaches nothing.

## Decision

Nominate a bounded set of candidate files from the task text and the current
snapshot, extract structure for only those files at the full per-file
allowance, and build, seed, and traverse the graph over that set.

Nomination consumes no reference change, accepted patch, or test outcome, and
its maximum is a closed constant rather than a caller input.

Every scoped result discloses that coverage is limited to nominated files, how
many were nominated, and why each was admitted. Report nomination recall
offline alongside map recall, because a file never nominated can never be
mapped.

## Consequences

Structural context becomes usable at ordinary repository size. Startup no longer
pays for a whole-repository pass, and the existing content-hash-keyed per-file
cache makes repeated nomination of the same file free.

The failure mode inverts and must be watched. A whole-repository graph is thin
but complete; a scoped graph is dense but partial. A nomination miss produces a
map that is confident, well-attributed, wrong, and — unlike today's empty map —
healthy-looking. Mandatory scope disclosure and nomination recall as the leading
metric exist for exactly that risk, and the measured 22% evidence recall
suggests nomination is where the ceiling currently sits.

ADR-0125's ranked seed sets are kept. They operate inside the scoped graph,
where the symbols they rank actually exist.

No security invariant changes. Scoping reduces work performed and data
retained; it grants no capability. This record grants no provider, grading,
publication, or submission authority.
