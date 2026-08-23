# Impresari Context — Phase 2 Delivery Record: Go Structural Admission

- Status: Accepted
- Date: 2026-08-23
- Approved by: Founder
- Roadmap role: Completed language-admission slice within Phase 2; see the
  [Phase 2 infrastructure-language and agent-expansion PRD](phase-2-infrastructure-language-and-agent-expansion-prd.md)
  for the complete phase.

## Objective

Add trustworthy, syntax-only Go structural evidence without introducing Go
toolchain, module-resolution, execution, network, or source-write authority.

## Scope

- Support `.go` through a pinned isolated Tree-sitter grammar.
- Emit bounded facts for named functions, methods, type specifications/aliases,
  import paths, direct identifier calls, and identifier references.
- Publish compatibility, ADR, dependency, and SBOM evidence.

## Non-goals

- Compiler, type-checker, language-server, formatter, test, or package commands.
- Go module, vendor, replacement, build-tag, generated-source, or runtime resolution.
- Claims about selector calls, dynamic dispatch, reflection, or runtime behavior.

## Acceptance criteria

- The Go grammar and resolver identities are pinned and validated by the worker.
- Tests prove bounded Go extraction and preserve fail-closed worker behavior.
- The full policy, contract, evaluation, SBOM, format, test, and lint gate passes.
- Hosted macOS, Linux, Windows, fuzzing, static-analysis, and dependency checks pass.
