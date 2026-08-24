# Impresari Context — Phase 4 Delivery Record: Java Structural Admission

- Status: In progress
- Date: 2026-08-24
- Approved by: Founder (via the approved Phase 4 roadmap)
- Roadmap role: First bounded language-admission slice within Phase 4; see the
  [Phase 4 enterprise-language and impact-evidence PRD](phase-4-enterprise-language-and-impact-evidence-prd.md)
  for the complete phase.

## Objective

Add trustworthy, syntax-only Java structural evidence without introducing Java
toolchain, classpath, build-system, execution, network, or source-write
authority.

## Scope

- Support `.java` through a pinned isolated Tree-sitter grammar.
- Emit bounded facts for named type and method declarations, import paths,
  direct identifier calls, and identifier references where the grammar confirms
  each fact.
- Publish compatibility, ADR, dependency, SBOM, and verification evidence.

## Non-goals

- `javac`, formatter, test, build-tool, dependency, or language-server calls.
- Classpath, package, module, annotation-processing, generated-source, or
  runtime resolution.
- Claims about member dispatch, overload selection, inheritance, reflection,
  dependency injection, or runtime behavior.

## Acceptance criteria

- The Java grammar and resolver identities are pinned and validated by the
  worker.
- Tests prove bounded Java extraction and preserve fail-closed worker behavior.
- The full policy, contract, evaluation, SBOM, format, test, and lint gate
  passes.
- Hosted macOS, Linux, Windows, fuzzing, static-analysis, and dependency checks
  pass before the slice is admitted.
