# Structural Query Output Bound — Architecture Requirements and Design

- ARD ID/version: IC-SQOB-ARD-144 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-08.
- Governing PRD: [IC-SQOB-144](../product/structural-query-output-bound-prd.md).
- Decision: [ADR-0144](../decisions/0144-bound-a-structural-query-by-its-own-ceiling.md).

## Two different quantities, one number

```text
consumer's budget.requested = 16 KiB
        │
        ├──▶ the packet delivered to the consumer      ← what the number is for
        │
        └──▶ the traversal's nodes and edges, in memory ← what it was also used for
```

The second is an intermediate value. It is never delivered; the packet builder
bounds delivery separately and far lower. Measuring it against the delivery
ceiling compared a working set to a shipping limit.

The distribution shows how close that ran. Across twenty-two astropy tasks the
per-query maximum ranged 2,493 to 17,562 bytes against a 16,384 ceiling, with a
median of 13,971. Success and failure were separated by under 1.2 KiB.

## The bound belongs to the engine

`MAX_STRUCTURAL_QUERY_OUTPUT_BYTES` is a closed constant beside the traversal's
other closed constants, not a caller's choice. A caller able to widen an
internal bound could steer selection, which is the reasoning already applied to
`MAX_STRUCTURAL_SEEDS` and `MAX_SEED_TRAVERSAL_MATCHES`.

One mebibyte is roughly sixty times the observed maximum. It is deliberately
not tight: the real limiter is the traversal's own node and edge counts, and
this bound exists only so a pathological result is never held whole.

## Narrowing drops edges, never nodes

Not a preference — a constraint the consumer imposes. `query_structure`'s
caller builds a path map from `result.nodes` and resolves every edge's
`source_node` against it, failing hard on a miss:

```rust
let paths = result.nodes.iter().map(|node| (node.node_id.as_str(), &node.path)).collect();
...
let worker_path = paths.get(edge.source_node.as_str()).ok_or_else(|| failure(..))?;
```

Dropping an edge can only shrink the set of sources that must be present.
Dropping a node could break that closure. So nodes are retained whole and edges
are narrowed from the tail, where `edge_id` ordering makes the retained prefix
deterministic for a snapshot.

## Narrowing is logarithmic, not quadratic

Every probe re-serializes the result to measure it. Removing edges one at a
time is therefore quadratic in the size of exactly the oversized input this
function exists to handle — which would replace a clean failure with a hang, a
worse product than the defect being fixed.

The retained prefix is binary-searched instead: `O(log n)` probes. The first
implementation here was the linear one, and its own test exposed the cost
before it reached review.

## Preserved properties

The consumer's `requested` is still parsed and still rejects a malformed
budget; it simply no longer bounds this value. Delivery is unchanged, because
the packet builder's own ceiling is what governs it. `SEC-INV-002`,
`SEC-INV-003`, `SEC-INV-007` and `SEC-INV-011` are untouched: no source is
read, written, executed, or excerpted, and narrowing removes information rather
than adding any.

## Relationship to ADR-0134

[ADR-0134] established that a disclosure exceeding its ceiling should be
narrowed and returned rather than discarded whole, "after the reads that
produced it were already spent". The structural query beneath that disclosure
kept the discarded-whole behaviour. This record applies the same principle one
layer down.

[ADR-0134]: ../decisions/0134-truncate-a-disclosure-at-its-ceiling-rather-than-discarding-it.md
