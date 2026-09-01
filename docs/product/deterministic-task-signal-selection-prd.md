# Deterministic Task-Signal Selection PRD

- PRD ID/version: IC-DTSS-117 / 1.0.
- Status: Implemented and provider-free verified.
- Date: 2026-09-01.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Deterministic Task-Signal Selection ARD](../architecture/deterministic-task-signal-selection-ard.md).
- Governing decision:
  [ADR-0117](../decisions/0117-decompose-task-text-before-repository-retrieval.md).

## Problem

The current profiled planner sends the complete task sentence to literal and
lexical retrieval. Lexical retrieval requires every normalized term to occur in
one candidate document. As a result, an exact identifier such as `hello-rust`
is found while a normal task such as “Find the Rust greeting hello-rust in
rust.rs” can return no evidence.

This is a deterministic product-selection defect. It is independent of the
model provider, adapter deadline, SWE-bench grader, and packet renderer.

## Outcome

Profiled context planning converts bounded natural-language task text into a
small, ordered set of exact retrieval signals. Explicit quoted text, portable
path candidates, and code-like identifiers remain lossless. Ordinary lexical
terms are filtered and searched independently so surrounding prose cannot make
an exact repository anchor undiscoverable.

## Requirements

1. Decompose task text locally and deterministically; do not call a model,
   embedding service, analyzer, shell, network, or provider.
2. Preserve the complete original query as the first profile-defined operation
   when it satisfies that retrieval kind's closed input contract and, for a
   literal operation, the request's actual per-item excerpt limit. Otherwise
   record an explicit omission and execute only bounded derived signals.
3. Extract closed classes of high-signal candidates: quoted/backticked spans,
   portable path-like tokens, and code-like identifiers containing syntax
   separators such as `-`, `_`, `::`, or `.`.
4. Extract bounded lexical fallbacks from ASCII alphanumeric terms after a
   frozen, versioned stop-word filter. Search fallbacks independently rather
   than joining unrelated task words with `AND`.
5. Produce no more than eight total retrieval steps, retain no more than 16
   path/code token candidates, accept no signal longer than 256 bytes, and
   admit no literal signal larger than the request's actual per-item excerpt
   limit. Perform no unbounded allocation or recursion under the 4,096-byte
   task cap.
6. Deduplicate signals by retrieval kind plus exact query bytes. Preserve a
   stable priority and first-occurrence tie-break.
7. Keep path matching, literal matching, lexical matching, current-source
   verification, packet limits, policy, and authority boundaries unchanged.
8. Expose every derived operation and a stable reason code in the existing
   deterministic plan so selection remains inspectable and reproducible.
9. Reject NUL, unsupported-control-bearing, empty, or oversized task text;
   treat tabs and line breaks only as separators. Never interpret task content
   as a command, glob, regular expression, FTS syntax, path authority, or
   structural-graph start node.
10. Demonstrate provider-free stability across exact, descriptive, noisy,
    reordered, quoted, path-bearing, and adversarial task variants.

## Acceptance

- The exact query `hello-rust` and a descriptive query containing that anchor
  both recover the same exact fixture evidence.
- Adding ordinary task prose does not remove an explicit identifier or path
  signal from the plan.
- Reordered prose produces the same high-signal set, with deterministic plan
  ordering under the documented priority rules.
- Control bytes, overlong spans, FTS operators, shell-looking text, traversal
  strings, and signal floods cannot add authority or escape hard limits.
- Existing exact-query, packet, structural, security, and interface tests stay
  green.
- Formatting, warnings-denied Clippy, all-target tests, docs, and hosted CI
  pass.

## Verification Result

Implementation verification passed locally on 2026-09-01. The complete
all-target/all-feature suite, warnings-denied Clippy, documentation build,
security vectors, and exact-versus-descriptive engine fixture are green. The
independent `repository-context-eval` v3 compatibility gate also confirmed
that exact and descriptive task forms recover a shared content-addressed Rust
evidence item while complete product-read telemetry remains valid. Hosted CI
remains required before merge.

## Non-Goals

- Semantic embeddings, model-authored query rewriting, or probabilistic intent
  classification.
- Building a second structural graph; Impresari's existing snapshot-bound
  Tree-sitter graph remains authoritative.
- Automatically choosing a graph start node or traversing structural edges.
- LeanCTX-style progressive delivery, session memory, or cross-session cache.
- Claiming token, cost, latency, correctness, or repository-read improvement.
- Running a paid provider study or official grader.

## Stop Conditions

Paid evaluation remains prohibited. Passing this increment authorizes the next
provider-free structural-seed and progressive-disclosure assessment only.
