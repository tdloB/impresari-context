# ADR-0032: C# structural-language admission

- Status: Proposed
- Date: 2026-08-24
- Scope: Phase 4 C# admission to the isolated structural worker

## Decision

Admit `.cs` source using the pinned `tree-sitter-c-sharp 0.23.5` grammar and
the existing `tree-sitter 0.26.12` runtime. Emit only bounded syntax-derived
facts directly covered by extraction tests.

## Constraints

- Parsing remains in the short-lived capability-reduced worker.
- No .NET compiler, formatter, test runner, MSBuild, project/dependency cache,
  environment, subprocess, network, or source-write authority is added.
- Using directives are syntactic facts only; static and aliased forms are
  omitted. The resolver does not resolve projects, NuGet packages, assemblies,
  dependencies, attributes, generated source, or platform behavior.
- Qualified calls, overload selection, member dispatch, inheritance, reflection,
  attributes, dependency injection, and runtime behavior remain explicit limits.

## Consequences

C# contributes conservative evidence-grade syntax facts without expanding the
authority boundary or implying .NET semantic analysis.

## References

- [ADR-0004: Source-language and parser strategy](0004-source-language-and-parser-strategy.md)
- [ADR-0010: Structural worker protocol and isolation](0010-structural-worker-protocol-and-isolation.md)
