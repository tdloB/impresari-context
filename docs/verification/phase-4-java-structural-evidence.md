# Phase 4 Java structural-evidence verification

- Status: Local release gate passed; hosted admission pending
- Governing records: [Java delivery record](../product/phase-4-java-structural-admission-prd.md) and [ADR-0030](../decisions/0030-java-structural-language-admission.md)

## Admitted local behavior

The isolated worker accepts `.java` source with the pinned
`tree-sitter-java 0.23.5` grammar. It emits syntax-confirmed facts for Java
type declarations (class, interface, enum, record, and annotation type), named
methods and constructors, non-static non-wildcard imports, direct unqualified
method calls, and identifier references. Every fact retains byte spans and the
pinned parser, grammar, resolver, and graph identities.

Static imports, wildcard imports, qualified/member calls, classpath/package
resolution, overload selection, inheritance, annotations, generated source,
build tools, compiler behavior, and runtime behavior are intentionally not
claimed.

## Local verification

- Structural unit coverage exercises records, methods, constructors, direct
  calls, qualified-call omission, import admission, static/wildcard omission,
  references, and a mismatched Java grammar identity.
- Engine and CLI checks prove `.java` admission is reflected in the shipped
  compatibility contract.
- `./scripts/check.sh` passed on 2026-08-24: policy, security boundary,
  tracked-source immutability, 21 schemas, identity/path/JCS/semantic vectors,
  SBOM (190 packages), evaluation and scale checks, cache restart, formatting,
  clippy, all unit/integration tests, and documentation tests.

## Hosted admission requirement

This slice is not marked accepted until the pull request passes the required
hosted macOS, Linux, Windows, fuzzing, static-analysis, and dependency-security
checks. That final result updates the delivery record and ADR without changing
the authority boundary.
