# ADR-0022: Rust structural-language admission

- Status: Accepted
- Date: 2026-08-23
- Scope: Phase 3 admission of Rust source to the existing isolated structural worker

## Decision

Admit `.rs` source using the pinned `tree-sitter-rust 0.24.2` grammar and the
existing `tree-sitter 0.26.12` runtime. The worker emits bounded,
syntax-derived facts for structs, enums, unions, traits, named functions, `use`
declarations, direct identifier calls, and identifier references.

## Constraints

- Parsing remains in the existing short-lived, capability-reduced worker.
- No Cargo command, compiler, formatter, test runner, crate graph, registry,
  package cache, environment, subprocess, network, or source-write authority
  is added.
- `use` declarations are syntactic facts only. The resolver does not resolve
  crates, modules, edition behavior, features, target-specific source, build
  scripts, generated source, or package dependencies.
- Macro expansion, procedural macros, trait selection, method/selector calls,
  dynamic dispatch, `cfg` evaluation, build scripts, and runtime behavior
  remain explicit syntax-only limitations.

## Verification

- Unit coverage exercises declarations, `use` declarations, direct calls, and
  references, including exclusion of declaration names from reference facts.
- Existing framing, source hash, grammar/toolchain identity, cache validation,
  source-immutability, dependency, SBOM, and cross-platform checks remain
  mandatory.

## Consequences

Rust receives structural evidence without expanding the authority boundary or
claiming Cargo, compiler, crate, macro, package, or runtime semantics.

## References

- [ADR-0004: Source-language and parser strategy](0004-source-language-and-parser-strategy.md)
- [ADR-0010: Structural worker protocol and isolation](0010-structural-worker-protocol-and-isolation.md)
