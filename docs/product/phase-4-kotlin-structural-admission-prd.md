# Impresari Context — Phase 4 Delivery Record: Kotlin Structural Admission

- Status: Accepted
- Date: 2026-08-24
- Approved by: Founder (via the approved Phase 4 roadmap)
- Roadmap role: Second bounded language-admission slice within Phase 4; see the
  [Phase 4 enterprise-language and impact-evidence PRD](phase-4-enterprise-language-and-impact-evidence-prd.md)
  for the complete phase.

## Objective

Add trustworthy, syntax-only Kotlin structural evidence without introducing a
Kotlin compiler, Gradle, classpath, execution, network, or source-write
authority.

## Scope

- Support `.kt` and `.kts` through a pinned isolated Tree-sitter grammar.
- Emit only tested facts for named classes, objects, functions, and type aliases;
  non-wildcard/non-aliased imports; direct identifier calls; and references.
- Publish compatibility, ADR, dependency, SBOM, and verification evidence.

## Non-goals

- Kotlin compiler, formatter, test, Gradle, dependency, or language-server calls.
- Classpath, package, Gradle model, script evaluation, dependency, generated
  source, coroutine, extension-dispatch, or runtime resolution.
- Claims about qualified calls, overload selection, inheritance, annotations,
  reflection, or runtime behavior.

## Acceptance criteria

- The Kotlin grammar and resolver identities are pinned and validated by the
  worker.
- Tests prove only the bounded Kotlin subset and preserve fail-closed behavior.
- The full policy, contract, evaluation, SBOM, format, test, and lint gate passes.
- Hosted macOS, Linux, Windows, fuzzing, static-analysis, and dependency checks
  passed before admission on 2026-08-24.
