# Target File Disclosure — Architecture Requirements and Design

- ARD ID/version: IC-TFD-ARD-146 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Governing PRD: [IC-TFD-146](../product/target-file-disclosure-prd.md).
- Decision: [ADR-0146](../decisions/0146-name-the-file-a-relationship-points-into.md).

## An entry already mixes two files

```text
edge:  source_node ───references───▶ target_node
        │                             │
        ├── display_path              ├── symbol_label   ← taken from here
        │   (the entry's file)        └── path           ← discarded
        ▼
   "sampled.py — BaseTimeSeries"
```

The name comes from one end of the relationship and the path from the other.
That is not a rendering quirk; it is what makes the entry useful — the symbol a
file *reaches* is more informative than the symbol it *contains*. But naming
the reached symbol while naming only the reaching file points a reader at the
wrong place.

The fix is to say both. `disclosure_item` already resolves the target node to
read its name; it now keeps the path as well.

## Only a different file is worth naming

Most relationships stay inside one file. Emitting the source path again on
every such entry would grow the map without telling a consumer anything, and
the map is the scarcest thing in the packet. A target path is therefore emitted
only when it differs from the entry's own, and the field is omitted otherwise.

## This discloses; it does not search

No traversal is extended, no repository read is added, no node is looked up
that was not already looked up. The target node is in `query.result.nodes`
because the traversal put it there, and the entry already finds it. The change
is one field kept instead of dropped.

That is why it can raise recall at essentially no cost, and equally why it
cannot reach a file the traversal never resolved. It moves nothing; it stops
withholding.

## Identity

`item_handle` hashes the edge, which carries `target_node`, so the new field
adds no degree of freedom and item identities are unchanged.

`map_id` hashes the rendered items, so map identities **do** change. That is
correct — the map's content changed — but any pinned or cached `map_id` from an
earlier build is stale and must be re-derived.

## Evaluator boundary

A disclosed target path is admitted only when the frozen source allowlist
already admits it, the same rule the entry's own path obeys. The evaluator
froze what an arm may see, and the product's answer does not widen it.

## Measured supply

Across the twenty-two-task corpus, entries hold **23 distinct files** that no
entry currently names — 72 named against 95 held, a 32% increase in breadth.
Three of those 23 are reference files the map currently misses.

That ratio is the honest cost: roughly one added file in eight is a file the
change actually touched. The rest widen the map without helping, and whether a
wider map assists or distracts a consumer is not measurable without a graded
run.

## Preserved invariants

`SEC-INV-002`, `SEC-INV-003`, `SEC-INV-007`, `SEC-INV-011` and `SEC-INV-012`
hold unchanged. No source is read, written, executed, or excerpted; the value
disclosed is derived from a traversal already performed over an already
authorized snapshot; and no consumer input reaches it.
