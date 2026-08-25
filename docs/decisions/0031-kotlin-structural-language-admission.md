# ADR-0031: Kotlin structural-language admission

- Status: Accepted
- Date: 2026-08-24
- Scope: Phase 4 admission of Kotlin source to the existing isolated structural worker

## Decision

Admit `.kt` and `.kts` source using the pinned `tree-sitter-kotlin-ng 1.1.0`
grammar and the existing `tree-sitter 0.26.12` runtime. The worker will emit
only bounded syntax-derived facts directly covered by extraction tests.

## Constraints

- Parsing remains in the existing short-lived, capability-reduced worker.
- No Kotlin compiler, formatter, test runner, Gradle, classpath, package cache,
  environment, subprocess, network, or source-write authority is added.
- Imports are syntactic facts only. Wildcard and aliased imports are omitted;
  the resolver does not resolve packages, dependencies, Gradle models, scripts,
  generated source, or platform-specific behavior.
- Qualified calls, overload selection, extension dispatch, inheritance,
  annotations, reflection, coroutines, and runtime behavior remain explicit
  syntax-only limitations.

## Verification

- Unit coverage exercises the admitted Kotlin constructs and grammar/toolchain
  identity checks, including the intentional import and qualified-call limits.
- Existing framing, source hash, cache validation, source-immutability,
  dependency, SBOM, and cross-platform checks remain mandatory.

## Consequences

Kotlin contributes a small, evidence-grade syntax subset without expanding the
authority boundary or claiming compiler, Gradle, dependency, or runtime semantics.

## References

- [ADR-0004: Source-language and parser strategy](0004-source-language-and-parser-strategy.md)
- [ADR-0010: Structural worker protocol and isolation](0010-structural-worker-protocol-and-isolation.md)
