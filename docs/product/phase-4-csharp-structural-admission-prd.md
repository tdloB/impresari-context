# Impresari Context — Phase 4 Delivery Record: C# Structural Admission

- Status: Accepted
- Date: 2026-08-24
- Approved by: Founder (via the approved Phase 4 roadmap)
- Roadmap role: Third bounded language-admission slice within Phase 4.

## Objective

Add trustworthy, syntax-only C# structural evidence without introducing a .NET
compiler, MSBuild, project/dependency resolution, execution, network, or
source-write authority.

## Scope

- Support `.cs` through a pinned isolated Tree-sitter grammar.
- Emit tested facts for classes, records, structs, delegates, constructors,
  methods, non-static/non-aliased using directives, direct identifier calls,
  and references.
- Publish compatibility, ADR, dependency, SBOM, and verification evidence.

## Non-goals

- Compiler, formatter, test, MSBuild, NuGet, project, or language-server calls.
- Project, package, dependency, generated-source, attribute, overload, member
  dispatch, reflection, or runtime resolution.
- Claims about static or aliased using directives, qualified calls, or runtime behavior.

## Acceptance criteria

- Grammar and resolver identities are pinned and worker-validated.
- Tests prove the bounded C# subset and fail-closed behavior.
- The full policy, contract, evaluation, SBOM, format, test, and lint gate passes.
- Hosted macOS, Linux, Windows, fuzzing, static-analysis, and dependency checks
  passed before admission on 2026-08-24.
