# ADR-0050: Haskell structural-language admission

- Status: Accepted for implementation
- Date: 2026-08-25

## Decision

Admit Haskell `.hs` and `.lhs` files through the isolated worker using pinned
`tree-sitter-haskell 0.23.1` (MIT). Emit only named bindings, direct imports,
references, and containment.

## Boundary

Do not invoke GHC, Cabal, Stack, package resolution, type inference,
typeclass resolution, Template Haskell, compiler, or runtime behavior.
