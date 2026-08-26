# ADR-0049: Clojure structural-language admission

- Status: Accepted for implementation
- Date: 2026-08-25

## Decision

Admit Clojure `.clj`, `.cljs`, and `.cljc` files through the isolated worker
using pinned `tree-sitter-clojure-orchard 0.2.8` (CC0-1.0). Emit only direct
literal namespace/declaration special forms, direct list-head calls,
references, and containment.

## Boundary

Do not evaluate readers, syntax quotes, macros, namespaces, classpaths,
dependencies, JVM/JavaScript tooling, or runtime behavior.
