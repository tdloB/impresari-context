# ADR-0019: Python structural-language admission

- Status: Accepted
- Date: 2026-08-23
- Scope: Phase 1 admission of Python source to the existing isolated structural worker

## Decision

Admit Python files with the `.py` extension to the structural worker through
the pinned `tree-sitter-python 0.25.0` grammar and the existing
`tree-sitter 0.26.12` runtime. The worker accepts the exact grammar identity
`tree-sitter-python-0.25.0`, records it in every fact's provenance, and
includes it in the worker/cache toolchain identity.

The project-owned resolver emits only bounded syntax-derived declarations,
containment, import statements, calls, and identifier references. A Python
assignment is a declaration only when its left-hand side is a direct
identifier. Import facts identify syntactic module text only; they do not
assert that a package, environment, relative import, or import-time side effect
can resolve.

## Constraints

- Parsing remains in the existing short-lived, capability-reduced worker.
- No Python interpreter, virtual environment, package metadata, `PYTHONPATH`,
  import resolution, repository configuration, subprocess, network, or source
  write authority is added.
- Decorators, dynamic dispatch, reflection, import hooks, and non-identifier
  assignment targets remain explicit limits of syntax-only facts.
- Syntax recovery remains visible through the existing `syntax_recovery_present`
  warning and does not imply runtime validity.

## Verification

- Unit coverage exercises classes, functions, direct assignments, absolute and
  relative imports, calls, references, and containment.
- Existing worker framing, source hash, grammar identity, toolchain identity,
  cache validation, and source-immutability checks remain mandatory.
- The dependency and SBOM gates cover the pinned MIT-licensed grammar.

## Consequences

Python now has structural evidence in the public compatibility matrix. This is
not compiler, interpreter, package, or language-server semantics, and it does
not expand the core's authority boundary.

## References

- [ADR-0004: Source-language and parser strategy](0004-source-language-and-parser-strategy.md)
- [Phase 0 language/client compatibility contract](0018-first-class-client-integration-and-compatibility-contract.md)
