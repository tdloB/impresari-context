# Task-Recall-First Context Selection — Architecture Requirements and Design

- ARD ID/version: IC-TRFC-ARD-125 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-03.
- Governing PRD:
  [IC-TRFC-125](../product/task-recall-first-context-selection-prd.md).
- Decision:
  [ADR-0125](../decisions/0125-select-ranked-seed-sets-and-traverse-to-definitions.md).

## Architecture outcome

```text
bounded task text
      │
      ▼
admitted path / identifier signals            (ADR-0124)
      │
      ▼
candidate resolution against the graph
      ├─ exact symbol inside exact task path      rank 0
      ├─ exact task file path                     rank 1
      ├─ globally unique exact symbol             rank 2
      └─ globally ambiguous exact symbol          rank 3, bounded, disclosed
      │
      ▼
ranked seed set (bounded, deterministic)
      │
      ▼
bounded traversal, now including
      ├─ reference  ──names──▶ declaration
      └─ declaration ──extends──▶ supertype
      │
      ▼
existing exact-source recovery and ranked packet builder
```

The selector remains a pure graph projection. It reads no source, opens no
process, and cannot accept a node identity from task text or an evaluator.

## From one seed to a ranked set

The previous algorithm returned `Option<seed>` and treated a tie as failure.
That conflates two different states: *nothing matched* and *several things
matched*. Only the first is a reason to give up.

Selection now returns an ordered, bounded set. Rank classes are total and
evaluated in order, and every rank is deterministic under a fixed tie-break of
portable path then node identity, so an identical snapshot yields an identical
set. Ambiguity is retained as a bounded, rank-ordered slice with an explicit
disclosure reason rather than discarded.

The maximum seed count is a closed constant. It is not caller-supplied, because
a caller able to widen the seed set could steer selection, and steering is
oracle authority.

## Why traversal must reach declarations and supertypes

The measured failure is structural, not lexical. A task names the symbol a user
touches; the defect frequently lives in what that symbol *is* or what it
*inherits from*.

`TimeSeries` appears in `sampled.py` as a reference. Its declaration and the
supertype `BaseTimeSeries` in `core.py` carry the behaviour the task describes.
Without a `names` and an `extends` step, a correct seed still yields a map of
the wrong file.

Both steps are ordinary bounded graph traversal over edge kinds the graph
already records. Neither adds a resource ceiling, reads source, or introduces a
second graph.

## Measurement boundary and oracle isolation

Quality is task-relative recall, not text similarity. Scoring therefore requires
a reference change, and a reference change is an oracle.

The scoring tool lives outside the engine, in the evaluation crate, and drives
the product as a black box:

```text
corpus { task_text, reference_change }
      │
      ├──── task_text only ────▶  product  ────▶ delivered context
      │                                                │
      └──── reference_change ──────▶ scorer ◀──────────┘
```

The engine never receives, reads, or can reach a reference change. A static
check enforces the absence of that path. This preserves the existing rule that
the product is not a retrieval oracle and keeps the score honest: a map cannot
be tuned to an answer it cannot see.

The scorer performs no model call and opens no network socket, so a full corpus
run costs nothing and can gate every commit.

## Preserved invariants

Nothing here relaxes a security invariant. `SEC-INV-003` (repository content is
data), `SEC-INV-007` (no execution or network), `SEC-INV-009` (budgets on every
operation), and `SEC-INV-011` (no exact-source authority without a verified hash
and span) all hold unchanged. The seed set is larger; the authority is not.

## Deferred work

Ranking remains lexical and structural. Model-assisted ranking, cross-file
dataflow, and test-to-implementation association are out of scope and would each
require their own record.
