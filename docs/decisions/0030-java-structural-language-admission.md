# ADR-0030: Java structural-language admission

- Status: Proposed
- Date: 2026-08-24
- Scope: Phase 4 admission of Java source to the existing isolated structural worker

## Decision

Admit `.java` source using the pinned `tree-sitter-java 0.23.5` grammar and the
existing `tree-sitter 0.26.12` runtime. The worker will emit only bounded,
syntax-derived facts that are covered by direct extraction tests.

## Constraints

- Parsing remains in the existing short-lived, capability-reduced worker.
- No Java compiler, formatter, test runner, build tool, classpath, package
  cache, environment, subprocess, network, or source-write authority is added.
- Import paths are syntactic facts only. The resolver does not resolve Java
  packages, modules, classpaths, dependencies, annotations, generated sources,
  or platform-specific behavior.
- Member dispatch, overload selection, inheritance, reflection, dependency
  injection, annotation processing, and runtime behavior remain explicit
  syntax-only limitations.

## Verification

- Unit coverage exercises the admitted Java constructs and grammar/toolchain
  identity checks.
- Existing framing, source hash, cache validation, source-immutability,
  dependency, SBOM, and cross-platform checks remain mandatory.

## Consequences

Java can contribute evidence-grade syntax facts without expanding the authority
boundary or claiming compiler, package, or runtime semantics.

## References

- [ADR-0004: Source-language and parser strategy](0004-source-language-and-parser-strategy.md)
- [ADR-0010: Structural worker protocol and isolation](0010-structural-worker-protocol-and-isolation.md)
