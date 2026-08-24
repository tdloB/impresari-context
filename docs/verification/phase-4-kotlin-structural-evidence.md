# Phase 4 Kotlin structural-evidence verification

- Status: Local release gate passed; hosted admission pending
- Governing records: [Kotlin delivery record](../product/phase-4-kotlin-structural-admission-prd.md) and [ADR-0031](../decisions/0031-kotlin-structural-language-admission.md)

## Admitted local behavior

The isolated worker accepts `.kt` and `.kts` source with the pinned
`tree-sitter-kotlin-ng 1.1.0` grammar. It emits syntax-confirmed facts for
named classes, objects, functions, and type aliases; non-wildcard,
non-aliased imports; direct identifier calls; and references. Every fact retains
byte spans and pinned parser, grammar, resolver, and graph identities.

Wildcard imports, aliased imports, qualified calls, Kotlin compiler/Gradle
behavior, classpaths, package/dependency resolution, scripts, coroutines,
extension dispatch, annotations, generated source, and runtime behavior are
intentionally not claimed.

## Local verification

- Structural unit coverage exercises each admitted declaration class, an exact
  import, wildcard and aliased import omission, direct-call admission,
  qualified-call omission, references, and mismatched grammar rejection.
- Engine tests prove `.kt` and `.kts` admission is explicit and pinned.
- Targeted `context-structural` and `context-engine` tests passed on 2026-08-24.
- `./scripts/check.sh` passed on 2026-08-24: policy, security boundary,
  tracked-source immutability, 21 schemas, identity/path/JCS/semantic vectors,
  SBOM (191 packages), evaluation and scale checks, cache restart, formatting,
  clippy, all unit/integration tests, and documentation tests.

## Admission requirement

Run the full release gate, then require hosted macOS, Linux, Windows, fuzzing,
static-analysis, and dependency-security checks before marking Kotlin accepted.
