# Impresari Context — Phase 3: Rust Structural Admission PRD

- Status: Accepted
- Date: 2026-08-23
- Approval basis: Founder’s autonomous roadmap directive

## Objective

Add trustworthy, syntax-only Rust structural evidence without introducing
Cargo, compiler, crate-resolution, execution, network, or source-write
authority.

## Scope

- Support `.rs` through a pinned isolated Tree-sitter grammar.
- Emit bounded facts for structs, enums, unions, traits, named functions, `use`
  declarations, direct identifier calls, and identifier references.
- Publish compatibility, ADR, dependency, and SBOM evidence.

## Non-goals

- Cargo, compiler, formatter, test, registry, or package commands.
- Crate, module, edition, feature, target, build-script, generated-source, or
  package resolution.
- Macro expansion, trait selection, selector calls, dynamic dispatch, `cfg`
  evaluation, or runtime behavior claims.

## Acceptance criteria

- The Rust grammar and resolver identities are pinned and validated by the worker.
- Tests prove bounded Rust extraction and preserve fail-closed worker behavior.
- The full policy, contract, evaluation, SBOM, format, test, and lint gate passes.
- Hosted macOS, Linux, Windows, fuzzing, static-analysis, and dependency checks pass.
