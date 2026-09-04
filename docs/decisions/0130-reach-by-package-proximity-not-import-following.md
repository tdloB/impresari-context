# ADR-0130: Reach by Package Proximity, Not Import Following

- Status: Accepted
- Date: 2026-09-04
- Related PRD: [Package-Proximate Nomination Reach](../product/package-proximate-nomination-reach-prd.md)
- Architecture: [Package-Proximate Nomination Reach](../architecture/package-proximate-nomination-reach-ard.md)
- Extends: [ADR-0128](0128-extract-structure-for-nominated-files-not-whole-repositories.md)

## Context

Map file recall stands at 11 of 27 reference files across twenty-two accepted
changes. Fifteen of the sixteen misses are files the task report never names, so
they are unreachable from task signals no matter how the ranking is arranged.
Three independent measurements agree that identifier matching is exhausted: the
slot curve is flat from 16 candidates to 64, the truncation-ordering sweep is
flat across every ratio tested, and the misses are absent from the report text.

Further reach has to come from repository structure. The intended mechanism was
following imports and inheritance out of a nominated file.

Measured against the sixteen missed files, import edges reach **1**. Files in
the same package as a nominated file reach **10**. The union is also 10 —
import following contributes nothing package proximity does not already cover.

Anchor depth was measured separately. Expanding from the top-ranked nominated
file reaches 8 of 16 and adds about 10 files. Two anchors reach 9 and add 13.
Three anchors reach 9 and add 23.

## Decision

Expand nomination by package proximity: admit files sharing the immediate parent
directory of the single highest-ranked directly nominated file, bounded by a
closed reach ceiling, ordered strictly after every directly nominated file.

Do not build import, inheritance, or call-graph following.

Define a package as the immediate parent directory. Keep the anchor rule, the
package rule, and the reach ceiling as compile-time constants. Mark reach files
with their own reason code and record an explicit unknown whenever a nomination
contains one.

Claim no improvement until map file recall is measured offline over the whole
corpus and gains at least two reference files without losing any currently
recalled.

## Consequences

The reachable ceiling rises from 11 of 27 to at most 19 of 27. That is a
ceiling, not a result: admitting a file to the scope makes its declarations
available to the map without putting them in it.

Every added file thins the others, because `MAX_SCOPED_FACTS` is a whole-scope
allowance divided across the scope. Ordering reach last is what keeps that
dilution from reaching the files the task named outright — a scope that runs out
of allowance loses guesses before it loses evidence.

Rejecting import following saves an import resolver per admitted language, which
is a substantial and recurring cost, and the measurement says it would have
bought one file out of sixteen. This record exists mainly so the idea is not
re-proposed from intuition later; it is a plausible design that the corpus does
not support.

The package rule is deliberately crude. A language-aware definition may reach
further, but nothing measured says so yet, and the crude rule needs no
per-language resolver.

No security invariant changes. Reach performs no additional repository read and
this record grants no execution, network, publication, or submission authority.
