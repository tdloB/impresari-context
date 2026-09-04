# ADR-0131: Admit and Rank Identifiers by Declaration

- Status: Accepted
- Date: 2026-09-04
- Related PRD: [Declaration-Aware Nomination](../product/declaration-aware-nomination-prd.md)
- Architecture: [Declaration-Aware Nomination](../architecture/declaration-aware-nomination-ard.md)
- Refines: [ADR-0124](0124-classify-task-signals-by-code-shape-not-token-position.md)
- Extends: [ADR-0129](0129-build-a-bounded-task-identifier-index-at-preparation.md)

## Context

ADR-0124 decided that a task token is code by its shape rather than its position
in the text, and that decision was right about position. Shape has since been
tightened twice — an interior separator in ADR-0124's implementation, and
version-shaped rejection alongside it — and both tightenings were correct
against the noise they targeted.

Shape is nonetheless the wrong oracle, and it fails in one systematic direction.
It requires an interior separator or a lowercase-to-uppercase transition, so it
rejects every single-word name: `Header`, `Card`, `Quantity`, `Table`, `Column`,
`WCS`, `HDUList`, `ndarray`. That is the commonest shape a Python class takes.

Measured on twenty-two accepted astropy changes, three of the eleven tasks that
miss their reference file return no map at all, reporting
`structural_seed_unavailable` because the task produced no admitted identifier.
In each of the three, the rejected word names a class declared in exactly the
reference file: `Header` in `io/fits/header.py`, `Quantity` in
`units/quantity.py`, `Card` in `io/fits/card.py`.

This also explains a run of null results. Raising the identifier ceiling from 16
to 64 changed nomination recall not at all, and freeing slots by rejecting
version-shaped noise changed it not at all. Neither could: the identifiers that
mattered were never admitted at any ceiling. The conclusion drawn at the time —
that identifier matching was exhausted — was wrong, and it was wrong because
every measurement of the ceiling reimplemented or inherited the same shape rule.

A second defect sits beside it. Nomination ranks by how many task identifiers a
file contains and counts a passing mention exactly as it counts a definition.
`Header` occurs in hundreds of astropy files and is declared in one.

## Decision

Record what each admitted file declares while preparation already reads it, and
admit a task token as a code identifier when it passes the existing shape rule
**or** the snapshot declares that name.

Detect declarations lexically: a declaration keyword at the start of a line,
after optional modifiers, followed by a name. Use one keyword union across
admitted languages rather than a table per language.

Rank by weighted evidence rather than by ground: score a file as its mentions
plus three times its declarations, breaking ties by path. A declaration is
strong evidence, not overriding evidence — measured, treating it as an absolute
tier gained six reference files and lost five, because a file declaring one task
identifier displaced one declaring the central type and mentioning most of the
rest.

Keep the shape rule as the fallback whenever no index is available. Bound
declarations per file, bind them to the index's snapshot, and retain names and
portable paths only.

## Consequences

Three tasks that return no map today have a direct path to their reference file,
and every later stage inherits a better anchor: seeds resolve into it and
traversal expands from it. A wrong anchor is not one bad pick — it is the wrong
starting point for everything downstream, so this sits upstream of any later
work on selection.

The keyword union is cruder than a grammar and is meant to be. Fifteen languages
would otherwise need fifteen tables, each a place to drift out of step with the
worker. A false positive admits a name that a task must still mention before it
changes anything.

Requiring the keyword at line start is what keeps prose out. It is not a return
to classifying by position: a declaration has a position and a prose mention
does not, which is the distinction ADR-0124 could not draw when looking at a
token alone.

Admission widens; authority does not. A token is admitted because the repository
declares it, repository content is data under `SEC-INV-003`, and nothing a
consumer sends can add a declaration — so `SEC-INV-012` is untouched. The index
still holds names and paths only and cannot become a retrieval path around
`SEC-INV-011`.

Measured over twenty-two accepted astropy changes, map file recall rises from 11
of 27 to 14 of 27 and symbol recall from 4 of 34 to 10 of 34, for 1.6% more
delivered bytes. Empty maps fall from three to one and vendored-code seeding
from three to two.

Six reference files are gained and three lost. The three were incidental
passengers in broader maps this change sharpened, and the PRD's "none lost"
criterion — written before the measurement — is recorded as failed rather than
rewritten, because sharpening selection cannot be purely additive.

No security invariant changes. This record grants no execution, network,
publication, or submission authority.
