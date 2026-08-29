# Phase 5 Scala Structural Admission PRD

- Status: Accepted

## Outcome

Add snapshot-bound structural evidence for Scala `.scala` files through the
isolated worker.

## Requirements

- Pin `tree-sitter-scala 0.26.2`; emit tested declarations, direct imports,
  direct calls, references, containment, and explicit syntax recovery.
- Reject grammar identity mismatch and preserve all existing resource limits.
- Do not run Scala, SBT, Mill, classpath/dependency, macro, implicit, or
  runtime machinery.

## Acceptance

Grammar, resolver, extension routing, compatibility manifest/matrix,
dependency policy, SBOM, negative import cases, and full gate are verified.
