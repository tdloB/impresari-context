# ADR-0117: Decompose Task Text Before Repository Retrieval

- Status: Accepted and implemented
- Date: 2026-09-01
- Decider: Aaron Boldt through the active evaluation roadmap continuation
- Related PRD: [Deterministic Task-Signal Selection PRD](../product/deterministic-task-signal-selection-prd.md)
- Architecture: [Deterministic Task-Signal Selection ARD](../architecture/deterministic-task-signal-selection-ard.md)

## Context

The profiled planner currently submits an entire natural-language task to
lexical retrieval. The lexical store safely compiles normalized terms with
`AND`, which is appropriate for an explicit lexical query but brittle for task
prose. Provider-free compatibility proved that `hello-rust` is recovered while
descriptive text containing the same identifier is not.

Impresari already owns deterministic exact search and a snapshot-bound
Tree-sitter structural graph. Replacing those systems, invoking a model to
rewrite the task, or loosening FTS parsing would increase authority and
variability without fixing the planning boundary.

## Decision

Add a bounded deterministic task-signal extractor to the profiled planner.
Keep the original profile operation, then add explicit quoted, path-like,
code-like, and filtered single-term lexical operations under a closed eight-step
budget. Send every operation through the existing exact-source retrieval and
packet path.

The version-1 extractor is ASCII-signal-only, process-local, non-persistent,
source-independent, and fully represented by existing plan steps and reason
codes.

## Consequences

- Ordinary prose can no longer suppress an explicit repository identifier by
  forcing unrelated words into one `AND` query.
- Exact, path, and identifier signals remain inspectable and reproducible.
- More retrieval steps can increase product reads, so the product read observer
  and provider-free utility gates remain mandatory.
- Natural-language-only semantic intent remains limited; this decision does
  not claim semantic understanding.
- Structural graph traversal and progressive disclosure stay separately
  versioned decisions.

## Alternatives

- Change all lexical searches from `AND` to `OR`: rejected because it broadens
  explicit lexical-query semantics and can flood packets with generic prose.
- Use an LLM or embedding model to rewrite tasks: rejected because it adds
  provider cost, nondeterminism, external-data handling, and a new correctness
  dependency before provider-free selection works.
- Copy Graft or LeanCTX wholesale: rejected because Impresari already has
  capability-relative exact reads, structural facts, source verification, and
  bounded packets that must remain authoritative.
- Increase time or packet budgets: rejected because the missing evidence is a
  query-planning failure, not an exhausted-resource result.

## Revisit triggers

Revisit before adding Unicode signals, stemming, learned ranking, embeddings,
model-authored rewriting, dynamic stop words, repository-frequency scoring,
automatic graph start-node selection, more than eight steps, persistent task
signals, or a new public plan schema.
