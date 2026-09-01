# Provider-Free Structural Utility Gate — Architecture Requirements and Design

- Status: Implemented; local provider-free gate passed, hosted CI required for merge.
- Date: 2026-09-01.
- Governing PRD:
  [Provider-Free Structural Utility Gate PRD](../product/provider-free-structural-utility-gate-prd.md).
- Governing decision:
  [ADR-0119](../decisions/0119-require-provider-free-utility-before-external-structural-delivery.md).

## Architecture outcome

```text
frozen fixture
  ├─ fresh ordinary engine ── profiled packet ─┐
  └─ fresh seeded engine ─ validated graph ─ packet
                                               │
                                               ▼
 model-neutral comparison: anchors, order, structural delta,
 packet bytes, product reads, repeats, determinism, mutation
```

The gate is an integration test in `context-evaluation`. It calls public engine
contracts and the same deterministic structural worker/graph builder used by
the product. It does not add a production API or make evaluator code part of
the runtime trust boundary.

## Fixture contract

Each closed fixture contains an ID, split, structural language, portable path,
exact task, source text, selected symbol, requested edge kinds, and minimum
new structural evidence. The manifest is tracked source and therefore frozen
by review and commit identity. No expected result is derived from an agent
answer.

The matrix includes TypeScript, Rust, and Ruby. Identifiers intentionally use
the admitted version-1 code-signal form so the fixture tests the ADR-0118
boundary rather than semantic inference.

## Comparison algorithm

1. Materialize the fixture under one private disposable source root and record
   the exact pre-run source digest.
2. Open baseline, seeded, and repeated seeded engines with distinct caches and
   equal policies; build complete snapshots and assert equal source
   fingerprints.
3. Build a validated graph from the seeded snapshot using a fixed worker
   request and closed fact classes.
4. Build one ordinary profiled packet and one seeded structural packet.
5. Treat the ordinary packet's ordered evidence IDs as the anchor inventory.
   Require that inventory to be an exact ordered prefix of the seeded packet.
6. Treat trailing `structural_graph_edge` evidence as the structural delta.
   Require the delta to be non-empty, unique, current-source verified, and
   bounded by the graph query result.
7. Compare complete product read telemetry. Snapshot work is included equally;
   the seeded delta may contain only exact relationship recovery reads.
8. Repeat through a fresh engine and cache and require byte-stable results.
9. Re-hash source bytes and require no mutation.

## Metrics and interpretation

- **Anchor retention:** retained baseline evidence / baseline evidence.
- **Structural novelty:** new structural evidence identities / delivered
  structural evidence identities.
- **Packet growth:** seeded serialized packet bytes minus baseline bytes.
- **Added reads/repeats/bytes:** seeded telemetry minus baseline telemetry.
- **Determinism:** equality of selection reason, normalized traversal, evidence
  ordering, and model-neutral deltas across fresh runs.

These values establish mechanical utility and bounded overhead only. They do
not estimate model tokens, agent behavior, correctness, latency, or cost.

The local frozen result is six novel structural records for six fixtures, with
six added reads, six added repeats, 485 added source bytes, and maximum
profiled-packet growth of 3,130 bytes. Graph construction is excluded and must
be accounted separately by any future external lifecycle.

## Security and failure behavior

- Disposable roots remain capability-relative and symlinks are rejected by
  the existing workspace layer.
- Portable fixture paths are converted through the workspace layer into the
  host's authoritative Unix-byte or Windows-UTF-16LE path identity before
  graph construction.
- Graphs must validate and match the current snapshot.
- Missing anchors, reordered anchors, no structural delta, incomplete
  telemetry, excess reads, packet growth above the ceiling, mutation, or
  nondeterminism fails the gate.
- Fixtures, failures, and output contain no credentials, prompts, provider
  responses, private benchmark instances, or publication authority.

## Next decision

If the gate passes, reassess a product-owned graph lifecycle for the external
MCP boundary. Progressive `map`, `lookup`, and `expand` remains a separate
LeanCTX-informed decision after static structural delivery is measurable.
