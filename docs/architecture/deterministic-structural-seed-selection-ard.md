# Deterministic Structural-Seed Selection — Architecture Requirements and Design

- Status: Core engine boundary and provider-free utility comparison implemented
  and locally verified; external protocol integration remains gated.
- Date: 2026-09-01.
- Governing PRD:
  [Deterministic Structural-Seed Selection PRD](../product/deterministic-structural-seed-selection-prd.md).
- Governing decisions:
  [ADR-0118](../decisions/0118-select-structural-seeds-from-admitted-task-signals.md),
  [ADR-0124](../decisions/0124-classify-task-signals-by-code-shape-not-token-position.md).

## Architecture outcome

```text
bounded task text
      │
      ▼
version-1 admitted path / identifier signals
      │
      ▼
pure structural seed selector
      ├─ unique symbol inside exact path
      ├─ exact file path
      ├─ globally unique symbol
      └─ explicit no-seed reason
      │
      ▼
existing snapshot-bound graph query (depth 1, hard limits)
      │
      ▼
existing exact-source recovery and ranked packet builder
```

The selector is a pure graph projection. It reads no source, opens no process,
and cannot accept a raw node id from task text or an external evaluator.

## Closed seed algorithm

Inputs are one validated graph and the already bounded `TaskSignals` produced
from the exact task. Version 1 considers only portable path candidates and
code-like identifiers.

Signal admission decides what the selector can ever see, so its shape rules are
part of this boundary. A bounded number of raw tokens is scanned, and each of
the path and identifier lists fills to its own ceiling from classified signals
rather than from token position; leading prose therefore cannot exhaust the
allowance before the first code signal. A token is code-shaped when it begins
with a letter or underscore and either carries a separator (`snake_case`,
`kebab-case`, `path::qualified`) or contains interior capitalization
(`CamelCase`, `ValueError`). Markup runs such as `--` are excluded by the
leading-character rule. A path's final component must contain a letter, so a
version such as `1.22.3` cannot displace a real path candidate.

Widening code shape does not widen authority. Lexical terms, quoted sentences,
and caller-supplied node identifiers still cannot seed, and a leading capital
alone remains prose.

1. Resolve task paths against graph file-node `display_path` using exact
   portable bytes. Zero or multiple matches do not produce a path scope.
2. Within the first exact unique path, resolve identifiers against confirmed
   symbol-node names. One match selects that symbol. Multiple matches are
   ambiguous and select no symbol.
3. If no symbol was selected, the exact unique path selects its file node.
4. Without a usable path, a code identifier selects a symbol only when exactly
   one confirmed graph symbol has that exact case-sensitive name.
5. Candidate order follows the existing task-signal priority and occurrence
   rules. At most one node is emitted.

The selection receipt records a stable reason: `unique_symbol_in_exact_path`,
`unique_exact_file_path`, `globally_unique_symbol`,
`structural_seed_unavailable`, or `structural_seed_ambiguous`.

## Traversal and packet rules

- The existing `query_graph` implementation remains the only traversal path.
- Version 1 uses depth one. Requested edge kinds remain within the graph
  contract's closed set and the request's existing limits are narrowed to the
  increment's fixed node/edge ceilings.
- Original exact/path/identifier evidence is inserted before structural edge
  evidence. Structural neighbors cannot evict the anchor that selected them.
- Every structural span is re-read and reverified through the authorized
  workspace before becoming packet evidence.
- No-seed is a valid fallback result; stale or malformed graph identity remains
  an error.

## Integration boundary

The first increment extends the engine API that already accepts a caller-owned
validated graph, replacing caller-owned start-node selection with product-owned
seed selection. It does not change MCP startup or the independent evaluator.
That allows provider-free utility measurement before adding a worker identity
and graph lifecycle to the external protocol.

## Verification

1. Pure selection vectors cover unique, scoped, ambiguous, reordered, hostile,
   and flooded signals.
2. Engine fixtures prove the anchor remains first and structural evidence is
   current-source verified.
3. Provider-free analysis compares evidence density, overlap, reads, and bytes
   with and without seeded traversal.
4. Existing exact/descriptive external compatibility remains green because the
   public static adapter is unchanged in this increment.

## Deferred LeanCTX increment

After static structural selection is useful, expose progressive `map`,
`lookup`, and `expand` through a fresh credential-free session. Reuse current
opaque source-bound evidence handles, process-local session ownership, exact
expansion hashes, and cumulative budgets. Do not introduce durable conversation
memory or destructive context replacement without a separate decision.
