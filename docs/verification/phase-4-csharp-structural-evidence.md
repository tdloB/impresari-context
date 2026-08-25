# Phase 4 C# structural-evidence verification

- Status: Accepted
- Governing records: [C# delivery record](../product/phase-4-csharp-structural-admission-prd.md) and [ADR-0032](../decisions/0032-csharp-structural-language-admission.md)

## Admitted local behavior

The isolated worker accepts `.cs` source with the pinned
`tree-sitter-c-sharp 0.23.5` grammar. It emits syntax-confirmed facts for
classes, records, structs, delegates, constructors, methods, non-static and
non-aliased using directives, direct identifier calls, and references. Every
fact retains byte spans and pinned parser, grammar, resolver, and graph
identities.

Static and aliased using directives, qualified calls, .NET compiler/MSBuild
behavior, project and NuGet resolution, assemblies, generated source,
attributes, overload selection, member dispatch, reflection, dependency
injection, and runtime behavior are deliberately not claimed.

## Local verification

- Structural unit coverage exercises each admitted declaration class, an exact
  using directive, static and aliased using omission, direct-call admission,
  qualified-call omission, references, and mismatched grammar rejection.
- Engine tests prove `.cs` admission is explicit and pinned.
- `./scripts/check.sh` passed on 2026-08-24: policy, security boundary,
  tracked-source immutability, 21 schemas, identity/path/JCS/semantic vectors,
  SBOM (192 packages), evaluation and scale checks, cache restart, formatting,
  clippy, all unit/integration tests, and documentation tests.

## Hosted admission

PR #42 passed the required hosted macOS, Linux (Rust 1.96, 1.97, and 1.98),
Windows, fuzzing, CodeQL static-analysis, and dependency-security/license checks
on 2026-08-24. It was squash-merged as
`1f8b236b23f993ef9c195a7bd65dfe2c18ea1256`. No authority-boundary change was
required by the hosted evidence.
