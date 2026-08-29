# Phase 5 Clojure Structural Admission PRD

- Status: Accepted

## Outcome

Add snapshot-bound structural evidence for Clojure `.clj`, `.cljs`, and
`.cljc` source files.

## Requirements

- Pin `tree-sitter-clojure-orchard 0.2.8`; emit only tested literal direct
  special forms, direct list-head calls, references, and containment.
- Treat EDN as lexical data, not Clojure source.
- Do not evaluate readers, syntax quote, macros, namespace/classpath,
  dependencies, JVM/JS tooling, or runtime behavior.

## Acceptance

Identity, boundaries, fixtures, manifest/matrix, policy, SBOM, and full gate
are verified.
