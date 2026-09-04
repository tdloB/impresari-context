# Task Identifier Index PRD

## Document Control

- PRD ID/version: IC-TII-129 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-03.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Task Identifier Index ARD](../architecture/task-identifier-index-ard.md).
- Governing decision:
  [ADR-0129](../decisions/0129-build-a-bounded-task-identifier-index-at-preparation.md).
- Governing objective: [CLAUDE.md](../../CLAUDE.md) — 98% quality at 78% compression.

## Problem

Seed-scoped extraction needs to know which files contain the identifiers a task
names. [ADR-0128](../decisions/0128-extract-structure-for-nominated-files-not-whole-repositories.md)
assumed a caller could supply that from "whatever index it already holds." No
such index exists, and building the wiring proved the omission expensive.

Answering the question by searching per identifier costs roughly 3,900
repository reads each. Nine identifiers is 36,286 reads against a 10,000
ceiling, so every task exhausted its budget and returned nothing. Reducing the
identifier count does not rescue it: six identifiers preserve nomination recall
exactly (16 of 27 reference files, unchanged from sixteen identifiers) and still
cost roughly 19,700 reads.

Nomination is a planning step, not context. Paying for it out of the request's
context read budget makes the product add work rather than replace it, which is
the failure the governing objective names.

## Product Outcome

A bounded index built once during preparation that maps each admitted file to
the code-shaped identifiers it contains, and answers "which files contain this
identifier" from memory with no repository read.

Index terms use exactly the rule task signals use, so the two sides agree by
construction. A term the task can name is a term the index can hold, and
neither side can drift from the other.

## Functional Requirements

1. Admit an identifier into the index only when it satisfies the same code-shape
   rule task signals apply. The rule has one definition; the index and the task
   planner both call it.
2. Build the index during preparation, reading each admitted file at most once.
   No lookup performs a repository read.
3. Bound the index explicitly: a maximum indexed file count, a maximum distinct
   identifiers retained per file, and a maximum identifier length. Exceeding any
   bound is recorded, never silent.
4. Bind the index to the exact workspace snapshot that produced it. An index
   must never answer for a different snapshot.
5. Answer a lookup deterministically. Identical snapshot and identifier yield an
   identical file set, in a stable order.
6. Produce the exact shape nomination consumes, so no adapter sits between the
   index and [IC-SSSE-128](seed-scoped-structural-extraction-prd.md).
7. Retain no source bytes, spans, or excerpts. The index holds identifiers and
   portable paths only.
8. Never consult a reference change, accepted patch, or test outcome.

## Acceptance Criteria

- A lookup performs zero repository reads, proven by construction: the lookup
  path takes no workspace handle.
- An identifier the task planner would admit is found by the index when present
  in a file; one the planner would reject is never indexed.
- Every bound is enforced and its breach recorded as an explicit unknown.
- An index bound to one snapshot refuses to answer for another.
- Repeating a build over identical inputs yields an identical index.
- A static check proves the module reaches no oracle, execution, or network
  surface, and retains no source bytes.
- The full repository gate passes.

## Non-Goals

- Ranking. The index answers membership; [IC-SSSE-128](seed-scoped-structural-extraction-prd.md)
  ranks and bounds nomination.
- Substring, fuzzy, or semantic matching. Exact identifier membership only.
- Replacing lexical retrieval. This index serves nomination, not evidence.
- Persisting the index across processes. Preparation rebuilds it.
