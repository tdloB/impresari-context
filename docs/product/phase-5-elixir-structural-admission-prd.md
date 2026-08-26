# Phase 5 Elixir Structural Admission PRD

## Outcome

Add snapshot-bound structural evidence for Elixir `.ex` and `.exs` files.

## Requirements

- Pin `tree-sitter-elixir 0.3.5`; emit tested direct modules, definitions,
  alias/import/require forms, identifier calls, references, and containment.
- Do not execute Mix, Hex, BEAM, macros, compile-time code, or runtime logic.

## Acceptance

Identity, boundaries, fixtures, manifest/matrix, policy, SBOM, and full gate
are verified.
