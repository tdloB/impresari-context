# Impresari Context — Phase 5 Demand-Led Language Evaluation PRD

- Status: Active gate; four languages admitted, no next language selected
- Date: 2026-08-25
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)
- Related ADR: [ADR-0040](../decisions/0040-demand-led-language-admission.md)

## Objective

Make Phase 5 language expansion a repeatable evidence-led decision rather
than a feature list. Scala, Elixir, Clojure, and Haskell are admitted.
C, C++, Ruby, PHP, and Swift have since received founder demand and are approved
as independently gated implementation slices under ADR-0064. Remaining
candidates include F#, Elm, Dart, and carefully constrained SQL; no remaining
candidate is admitted automatically.

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
