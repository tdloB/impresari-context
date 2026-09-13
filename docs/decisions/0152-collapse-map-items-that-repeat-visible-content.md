# ADR-0152: Collapse Map Items That Repeat Another's Visible Content

- Status: Accepted
- Date: 2026-09-10
- Related PRD: [Collapse Repeated Map Items](../product/collapse-repeated-map-items-prd.md)
- Architecture: [Collapse Repeated Map Items](../architecture/collapse-repeated-map-items-ard.md)
- Follows: [ADR-0151](0151-select-structural-context-by-what-it-is.md)

## Context

A disclosure map item is built from one traversal edge, and an agent reads it as
its path, target path, relationship, symbol label and confidence. Several edges
often read identically:

- An unresolved edge has no target, so its label falls back to the name of the
  node it leaves, and every unresolved reference from one seed reads as the
  same line.
- A name referenced or called several times yields several edges to one target
  that differ only in where they occur, which the map does not show.

On current `main`, over the twenty-two-task astropy corpus, 608 of 1,371 map
items (44%) repeated an earlier item's visible content. Each repeat costs the consumer tokens, counts against the
item ceiling, and tells an agent nothing its first occurrence did not.

## Decision

Before the item ceiling is applied, the map keeps the first item for each
distinct visible content (display path, target path, relationship, symbol label,
confidence, freshness and unknowns) and drops the rest. ADR-0151 orders edges by
what they are, so the first occurrence is the one the traversal valued most. It
keeps its handles, so every relationship shown remains resolvable by lookup and
expansion.

When any item is dropped, the map's omissions include
`repeated_relationships_collapsed`.

Item fields and the progressive contract version are unchanged. Every field keeps
its meaning; the set no longer carries exact visual repeats, and the omission
says so on every map it affects. Nothing pins the contract version, and none of
its consumers depend on one item per edge.

## Consequences

Measured on the twenty-two-task astropy corpus against current `main`, run from
the same directory:

| | `main` | this decision |
| --- | --- | --- |
| map items | 1,371 | 763 |
| map JSON | 439,240 bytes | 265,881 bytes |
| map in the evaluation harness's item shape | 123,042 bytes | 69,236 bytes |
| whole build result | 1,747,867 bytes | 1,375,810 bytes |
| map file recall | 22/27 | 22/27 |
| map symbol recall | 15/34 | 15/34 |
| evidence file recall | 16/27 | 16/27 |

Every distinct visible relationship survives on all twenty-two tasks, in the
order it first appeared, and recall is identical task by task. Twenty-one of the
twenty-two maps carried a repeat and say so; nomination and the packet's
evidence are unchanged. At about four bytes a token, the map in the harness's
item shape falls from roughly 1,400 tokens a task to 790.

A seed's traversal still spends its sixteen-edge limit on repeats before they
are collapsed. Letting repeats not count against that limit would admit more
distinct relationships, which is a change to selection and needs its own
measurement.

## Alternatives considered

**Keep one item per edge and show an occurrence count.** Deferred: it adds a
field to every item for a number no consumer uses yet.

**Collapse in the traversal, so repeats never consume the edge limit.** Deferred:
it changes which relationships are selected and what a structural query returns.
This decision changes delivery only.

**Leave it to each client's renderer.** Rejected: every client would still receive
and pay for the repeats.
