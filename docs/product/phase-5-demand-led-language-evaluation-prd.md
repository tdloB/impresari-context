# Impresari Context — Phase 5 Demand-Led Language Evaluation PRD

- Status: Approved evaluation gate; no language implementation selected
- Date: 2026-08-25
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)
- Related ADR: [ADR-0040](../decisions/0040-demand-led-language-admission.md)

## Objective

Make Phase 5 language expansion a repeatable evidence-led decision rather
than a feature list. Candidate languages are Swift, PHP, Ruby, C/C++, Scala,
Dart, and carefully constrained SQL; no candidate is admitted automatically.

## Required admission evidence

- At least one attributable adopter request or evaluated representative
  repository, recorded without source-content retention.
- Structural grammar availability and version pinning feasibility.
- A language-specific fact contract and explicit unsupported semantics.
- Isolated-worker, budget, source-integrity, adversarial, and deterministic
  corpus evaluation evidence.
- Expected decision value over current supported languages and maintenance cost.

## Decision outcomes

- `admit`: create a language PRD and ADR, then implement only that language.
- `defer`: retain the evidence and revisit without code changes.
- `reject`: document why the evidence or authority boundary is insufficient.

## Non-goals

- Popularity rankings, telemetry collection, background repository scanning,
  networked analytics, or bulk language implementation.
