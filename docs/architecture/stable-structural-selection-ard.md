# Stable Structural Selection — Architecture Requirements and Design

- ARD ID/version: IC-SES-ARD-151 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Governing PRD: [IC-SES-151](../product/stable-structural-selection-prd.md).
- Decision: [ADR-0151](../decisions/0151-select-structural-context-by-what-it-is.md).

## Where the map chooses

```text
seed candidates -> sort -> admit (8 total, 3 per file)
each seed       -> query_graph (depth 1, 16 nodes, 16 edges)
traversals      -> merge -> output ceiling (1 MiB) -> map items
```

## Where identity decided

| choice | decided by | now decided by |
| --- | --- | --- |
| seed tie within one file | node identity | declaration offset |
| a node's edges, and which sixteen survive | edge identity | resolution, then source position, kind, target path, name and span, module |
| merged edge order, and what the ceiling keeps | edge identity | seed order, then traversal order |

A node identity hashes the workspace snapshot; an edge identity hashes the
snapshot and the fact provenance. The snapshot identity hashes the workspace
identity, which hashes the checkout's resolved path, and the provenance includes
the recorded parser version. None of those is a property of the code.

## The orders

`context_structural::query_graph` sorts each node's outgoing edges with
`traversal_order`: `resolution_rank` (confirmed and `confirmed_manifest` 0,
heuristic 1, unresolved 2, anything else 3), then span start and end, kind,
target path, name and span, module, and edge identity last.

`merge_structural_traversals` keeps each traversal's edges in order and drops a
repeated edge identity after its first occurrence. Nodes remain the identity-
ordered union; no limit selects nodes after the merge.

`sort_seed_candidates` orders `(rank, nomination position, path, declaration
offset, node identity)`; `admit_seeds` is unchanged.

## What identity still does

It deduplicates, it breaks ties between entries identical in content, and it
orders the canonical graph for storage and identity. It no longer decides what
is delivered.

## Measurement

On the twenty-two-task astropy corpus, with the product's recall scorer:

- Label test: the fix built with each parser label, run in one folder.
- Folder test: one build run in two folders.
- Source order alone against resolved-first, run in one folder.

| comparison | map items | difference | map symbol recall |
| --- | --- | --- | --- |
| `main`, two directories | 1,440 / 1,439 | 385 items delivered differently | 15 / 12 |
| this change, two parser labels, one directory | 1,371 / 1,371 | none, same order | 15 / 15 |
| this change, two directories, one build | 1,371 / 1,371 | none, same order | 15 / 15 |

Run from one directory with the rest of this change in place, source order
alone names 12 of 34 reference symbols against 15 for resolved-first, delivers
668 distinct items against 763, and keeps 795 unresolved relationships against
618: `astropy-13579` loses `pixel_to_world_values` and `world_to_pixel_values`,
and `astropy-14369` loses `_make_parser`.

## Verification

- `a_traversal_keeps_the_same_edges_whatever_their_identities`
- `a_traversal_keeps_resolved_relationships_before_unresolved_ones`
- `a_traversal_orders_same_named_targets_by_where_they_are_declared`
- `seed_ties_in_one_file_go_to_the_earliest_declarations_under_any_snapshot`
- `merged_traversals_keep_seed_order_rather_than_identity_order`

Each fails with its mechanism disabled.

## Preserved invariants

- Every limit is unchanged.
- Selection is deterministic.
- Graph construction, identities and graph verification are unchanged.
- No new authority.
