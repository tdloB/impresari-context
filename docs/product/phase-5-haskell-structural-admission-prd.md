# Phase 5 Haskell Structural Admission PRD

## Outcome

Add snapshot-bound structural evidence for Haskell `.hs` and `.lhs` files.

## Requirements

- Pin `tree-sitter-haskell 0.23.1`; emit tested named bindings, direct imports,
  references, containment, and explicit syntax recovery.
- Do not run GHC, Cabal, Stack, packages, type inference/typeclasses, Template
  Haskell, compiler, or runtime logic.

## Acceptance

Identity, boundaries, fixtures, manifest/matrix, policy, SBOM, and full gate
are verified.
