# ADR-0040: Demand-led language admission

- Status: Accepted
- Date: 2026-08-25
- Scope: Phase 5 selection gate

## Decision

Phase 5 languages are selected only from documented adopter or evaluation
evidence. A candidate does not become implemented merely because it appears in
the roadmap, is popular, or has an available Tree-sitter grammar.

Every admitted language requires a separate PRD and ADR defining its facts,
parser versions, resolver boundary, unsupported states, corpus, security
tests, and maintenance rationale.

## Consequences

Scala, Elixir, Clojure, and Haskell have since passed separate PRD/ADR and
admission gates. The roadmap retains Swift, PHP, Ruby, C/C++, Dart, and
constrained SQL as candidate directions without creating a promise, authority
expansion, or implementation commitment. Another Phase 5 slice begins with
new attributable demand evidence and a decision record, not parser code.

ADR-0064 records a founder override and attributable demand for the named C,
C++, Ruby, PHP, and Swift batch. This gate continues unchanged for F#, Elm,
SQL, Dart, and any other language outside that decision.
