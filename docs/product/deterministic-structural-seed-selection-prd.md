# Deterministic Structural-Seed Selection PRD

- PRD ID/version: IC-DSSS-118 / 1.0.
- Status: Core selector implemented and locally verified; provider-free utility
  comparison and external protocol integration remain gated.
- Date: 2026-09-01.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Deterministic Structural-Seed Selection ARD](../architecture/deterministic-structural-seed-selection-ard.md).
- Governing decision:
  [ADR-0118](../decisions/0118-select-structural-seeds-from-admitted-task-signals.md).

## Problem

Impresari has a current-snapshot Tree-sitter graph and bounded typed traversal,
but profiled structural context requires a caller to provide the exact start
node. The independent evaluator cannot supply that node without becoming a
retrieval oracle. The newly implemented task-signal planner can recover exact
paths and identifiers, but it does not yet connect those anchors to structural
relationships.

## Outcome

The product deterministically selects at most one structural start node from
already-admitted exact path and code-identifier signals. It then reuses the
existing graph validator, traversal, exact-source recovery, packet ordering,
and resource limits. Ambiguous or absent seeds remain explicit and ordinary
exact retrieval continues without structural evidence.

## Requirements

1. Accept only a validated structural graph bound to the current workspace
   snapshot; never build, discover, download, or infer a second graph.
2. Reuse version-1 task signals. Lexical fallback terms, arbitrary prose,
   quoted sentences, shell-looking text, and caller-supplied node identifiers
   cannot become graph authority.
3. Prefer an exact symbol-name match inside an exact task path, then an exact
   file-path match, then one globally unique exact symbol-name match.
4. Select no seed when the highest applicable class is ambiguous. Report a
   stable omission/unknown reason and retain ordinary retrieval.
5. Select at most one seed, traverse only the existing closed edge kinds, and
   default to depth one with independent node, edge, byte, time, and memory
   ceilings no broader than the admitted request budget.
6. Keep exact/path/identifier evidence ahead of structural neighbors in packet
   priority. Deduplicate by evidence identity.
7. Bind graph identity, seed node, edge kinds, traversal result, truncation,
   reason codes, plan identity, and source evidence to the returned packet.
8. Perform no provider, model, embedding, network, Git, language-server,
   compiler, package-manager, test, or source-write operation.
9. Prove deterministic equality for repeated inputs, reordered non-signal
   prose, and exact/descriptive variants that preserve admitted candidate
   priority.
10. Measure provider-free evidence density, overlap, added reads, and packet
    growth before exposing automatic structural seeding through the external
    MCP evaluation protocol.

## Acceptance

- Exact `review.rs` plus `reviewed_change` task signals select the unique symbol
  in that file and recover its bounded outgoing structural evidence.
- A unique path without a matching symbol selects the file node.
- Duplicate symbol names without a path select no node and do not abort exact
  retrieval.
- Traversal, ambiguity, stale graph, hostile text, and resource-limit vectors
  remain deterministic and fail closed.
- Existing structural, retrieval, packet, security, interface, and task-noise
  suites remain green.
- Formatting, warnings-denied Clippy, all-target tests, docs, and hosted CI
  pass.

## Non-Goals

- Building the graph automatically during MCP startup.
- Adding structural-worker identity to the external evaluation protocol.
- Progressive pull, hybrid delivery, durable sessions, cross-session cache, or
  LeanCTX-style content replacement.
- Learned ranking, embeddings, semantic intent, reverse-edge traversal, or
  multi-seed graph search.
- Paid provider tests, official grading, publication, or performance claims.

## Stop Condition

External protocol integration and progressive delivery remain separate
increments. They require this provider-free structural utility gate to pass.
