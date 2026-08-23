# ADR-0021: Go structural-language admission

- Status: Accepted
- Date: 2026-08-23
- Scope: Phase 2 admission of Go source to the existing isolated structural worker

## Decision

Admit `.go` source using the pinned `tree-sitter-go 0.25.0` grammar and the
existing `tree-sitter 0.26.12` runtime. The worker emits bounded,
syntax-derived facts for named functions, methods, type specifications and
aliases, import specifications, direct identifier calls, and identifier
references.

## Constraints

- Parsing remains in the existing short-lived, capability-reduced worker.
- No Go compiler, formatter, test runner, `go list`, package/module cache,
  environment, subprocess, network, or source-write authority is added.
- Import paths are syntactic facts only. The resolver does not resolve Go
  packages, module replacements, vendoring, build tags, generated source, or
  platform-specific files.
- Selector calls, dynamic dispatch, reflection, build constraints, and runtime
  behavior remain explicit syntax-only limitations.

## Verification

- Unit coverage exercises functions, methods, type specifications, imports,
  direct calls, and references.
- Existing framing, source hash, grammar/toolchain identity, cache validation,
  source-immutability, dependency, SBOM, and cross-platform checks remain
  mandatory.

## Consequences

Go receives structural evidence without expanding the authority boundary or
claiming compiler, package, or runtime semantics.

## References

- [ADR-0004: Source-language and parser strategy](0004-source-language-and-parser-strategy.md)
- [ADR-0010: Structural worker protocol and isolation](0010-structural-worker-protocol-and-isolation.md)
