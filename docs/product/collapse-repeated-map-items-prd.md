# Collapse Repeated Map Items PRD

## Document Control

- PRD ID/version: IC-CRM-152 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Collapse Repeated Map Items ARD](../architecture/collapse-repeated-map-items-ard.md).
- Governing decision:
  [ADR-0152](../decisions/0152-collapse-map-items-that-repeat-visible-content.md).
- Follows: [ADR-0151](../decisions/0151-select-structural-context-by-what-it-is.md).

## Problem

The disclosure map sent one item per traversal edge, and many edges read
identically to an agent. Every unresolved reference from a seed carries the
seed's own name, and repeated references to one target differ only in a source
position the map does not show. On current `main`, over the twenty-two-task astropy corpus, 608 of 1,371 map
items (44%) repeated an earlier item's visible content. The consumer paid for each
repeat in tokens and in item-ceiling capacity and learned nothing from it.

## Product Outcome

The map sends each relationship an agent can see once, loses none of them, and
says when it has collapsed repeats.

## Functional Requirements

1. The map keeps the first item for each distinct visible content and drops
   later items with the same content.
2. Visible content is the display path, target path, relationship, symbol label,
   confidence, freshness and unknowns.
3. The kept item retains its handles for lookup and expansion.
4. The collapse runs before the item ceiling, so the ceiling counts distinct
   items.
5. A map from which any item was dropped carries the omission
   `repeated_relationships_collapsed`.
6. Nomination, the packet's evidence, and every item field are unchanged.

## Acceptance Criteria

- On the twenty-two-task astropy corpus every task keeps every distinct visible
  relationship it had on `main`, in first-occurrence order.
- Map items fall from 1,371 to 763 and the map JSON from 439,240 to 265,881
  bytes.
- Map file, map symbol and evidence file recall are identical to `main` on every
  task.
- A test proves repeats are dropped and distinct items kept in order, and fails
  when the collapse is disabled.
- The full repository gate passes.

## Non-Goals

- Changing which relationships the traversal selects.
- Adding an occurrence count to items.
