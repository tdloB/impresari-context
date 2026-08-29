# Impresari Context — Phase 5: Demand-Led Language Expansion PRD

- Status: Complete for the accepted scope
- Date: 2026-08-23
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)

## Objective

Expand language support only where adopter demand and measured evaluation
evidence justify the parser, resolver, and maintenance cost.

## Admitted languages

Scala, Elixir, Clojure, and Haskell have passed their bounded structural
admission gates. Their individual PRDs and ADRs remain the claim boundaries.

## Approved expansion program

C, C++, Ruby, PHP, and Swift completed independently gated implementation under
the [five-language expansion PRD](phase-5-five-language-expansion-prd.md).

## Remaining candidate languages

F#, Elm, Dart, and SQL under a deliberately constrained structural model. No
remaining candidate is selected without new attributable demand evidence.

## Admission requirements

- Demonstrated adopter demand and a defined evidence use case.
- A pinned, isolated grammar and bounded resolver with explicit limitations.
- Evaluation corpus, compatibility evidence, dependency/SBOM review, and full
  quality gates before any structural-support claim.
- SQL remains syntax- and policy-constrained; it does not receive database
  execution, connection, schema-discovery, or runtime-query authority.
