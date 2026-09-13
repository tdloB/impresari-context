# Collapse Repeated Map Items — Architecture Requirements and Design

- ARD ID/version: IC-CRM-ARD-152 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Governing PRD: [IC-CRM-152](../product/collapse-repeated-map-items-prd.md).
- Decision: [ADR-0152](../decisions/0152-collapse-map-items-that-repeat-visible-content.md).

## Where repeats come from

`disclosure_item` builds one item from one edge. Its `symbol_label` is the
target node's name, or the source node's name when the edge has no target, and
its `confidence` is the edge's resolution. Two edges therefore read identically
when they leave the same file for the same target, or for no target at all, with
the same kind and resolution. Only their spans and identities differ, and the map
shows neither.

## The collapse

```text
edges -> disclosure_item (one per edge) -> collapse_repeated_items -> item ceiling -> map
```

`collapse_repeated_items` keeps the first item for each key

```text
(display path, target path, relationship, symbol label, confidence, freshness, unknowns)
```

and reports whether it dropped any. Items arrive in traversal order, which
ADR-0151 made an order by what an edge is, so the first occurrence is the one the
traversal valued most. The kept item is stored with its evidence handle and edge
identity exactly as before; dropped items are never stored, so no handle is
issued that the map does not show.

Because the collapse runs before `truncate_to_item_ceiling`, the ceiling counts
distinct items, and the map identity, the session's lookup targets and the
consumption receipt all describe the delivered set. When anything was dropped,
`repeated_relationships_collapsed` joins the map's omissions.

## Compatibility

Every item field keeps its meaning. The progressive contract version is
unchanged: nothing pins it, the evaluation harness reads omissions as free text
and renders only the items it receives, and no consumer maps items to edges one
to one.

## Measurement

Against current `main`, run from the same directory on the twenty-two-task
astropy corpus:

- Every task keeps every distinct visible relationship, in first-occurrence
  order, and recall is identical task by task.
- Twenty-one of twenty-two maps collapsed at least one repeat and carry the
  omission; nomination and the packet's evidence are unchanged.
- Items fall from 1,371 to 763 (44.3%), the map JSON from 439,240 to 265,881
  bytes (39.5%), the map in the evaluation harness's item shape from 123,042 to
  69,236 bytes (43.7%), and the whole build result from 1,747,867 to 1,375,810
  bytes (21.3%).

## Verification

- `a_relationship_that_reads_the_same_is_delivered_once` fails with the collapse
  disabled.
- The existing map item and ceiling tests pass unchanged.
