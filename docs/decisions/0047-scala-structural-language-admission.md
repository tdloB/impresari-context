# ADR-0047: Scala structural-language admission

- Status: Accepted for implementation
- Date: 2026-08-25

## Decision

Admit Scala `.scala` files through the isolated worker using pinned
`tree-sitter-scala 0.26.2` (MIT). Emit only syntax-confirmed classes, objects,
traits, enums, named functions, direct non-wildcard/non-selector imports,
direct identifier calls, references, and containment.

## Boundary

Do not invoke Scala compilers, SBT, Mill, classpath or dependency resolution,
implicit or macro expansion, extension dispatch, or runtime behavior.
