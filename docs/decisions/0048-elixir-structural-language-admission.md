# ADR-0048: Elixir structural-language admission

- Status: Accepted for implementation
- Date: 2026-08-25

## Decision

Admit Elixir `.ex` and `.exs` files through the isolated worker using pinned
`tree-sitter-elixir 0.3.5` (Apache-2.0). Emit only literal module/function
forms, direct alias/import/require forms, direct identifier calls, references,
and containment.

## Boundary

Do not invoke Mix, Hex, BEAM, macro expansion, protocol dispatch, compile-time
code, package resolution, or runtime behavior.
