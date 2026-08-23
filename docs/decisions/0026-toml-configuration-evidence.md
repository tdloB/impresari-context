# ADR-0026: Bounded TOML configuration evidence

- Status: Accepted
- Date: 2026-08-23
- Scope: Phase 1 TOML configuration evidence in the existing isolated
  structural worker

## Context

TOML is a common repository configuration format, including build, packaging,
tool, editor, and deployment surfaces. It is useful for exact configuration-key
and containment anchors, but a TOML document is not proof that a tool loaded it
or that its values describe runtime behavior.

The worker contract requires every structural fact to carry an exact raw-source
span and pinned parser provenance. Parsing into a semantic TOML value alone
would discard those byte locations and move parsing into the higher-authority
control process. That would violate the project evidence and authority
boundaries.

## Decision

Admit `.toml` artifacts through the existing short-lived structural worker
using the pinned MIT-licensed `tree-sitter-toml-ng` 0.7.0 grammar with the
existing `tree-sitter` 0.26.12 runtime. The worker accepts the exact grammar
identity `tree-sitter-toml-ng-0.7.0`, records it in every fact's provenance,
and emits only syntax-derived declarations and containment facts for:

- key/value pairs;
- tables; and
- table-array elements.

The emitted name is the raw TOML key or table text as it appears in the parsed
source. This preserves a directly recoverable syntax anchor; it does not claim
canonical key normalization or value interpretation.

Any parser recovery/error for a TOML request yields no facts and the existing
source-free `syntax_recovery_present` warning. All existing source-hash,
request/response, fact-count, nesting-depth, response-size, worker-identity,
and graph-validation limits remain required.

No include resolution, interpolation, environment access, value evaluation,
schema validation, toolchain or package resolution, build-script loading,
process execution, network access, source mutation, or configuration-to-code
semantic claim is added.

## Consequences

- TOML becomes a supported Phase 1 configuration family with exact
  syntax-backed evidence.
- The additional native grammar is limited to the existing lower-authority
  worker process and is subject to lockfile, SBOM, license, audit, and
  cross-platform gates.
- Valid TOML may contribute only key/table/containment anchors, never an
  inference about how Cargo, a package manager, a compiler, editor, runtime,
  or deployment system behaves.
- Syntax-malformed TOML fails closed rather than yielding partial recovered
  facts.

## Verification

- The worker rejects an incorrect TOML grammar identity.
- Valid TOML emits key, table, table-array, and containment facts with
  `tree-sitter-toml-ng-0.7.0` provenance.
- Malformed TOML emits no facts and an explicit recovery warning.
- Engine admission recognizes `.toml` paths and publishes matching
  compatibility metadata.
- Dependency, license, SBOM, evaluation, worker-isolation, source-immutability,
  lint, and Tier A hosted platform checks remain required.

## Alternatives considered

### Parse TOML semantic values in the control process

Rejected. Semantic values do not preserve the byte-accurate locations required
for recoverable evidence, and this would broaden the control-process parsing
surface.

### Treat TOML as lexical-only

Rejected. It does not meet the approved Phase 1 configuration-evidence outcome
when exact concrete syntax can be bounded in the existing worker.

### Evaluate TOML per consuming tool

Rejected. Tool-specific configuration behavior introduces runtime, environment,
and dependency semantics outside the read-only evidence-engine boundary.

## References

- [ADR-0004: Source-language and parser strategy](0004-source-language-and-parser-strategy.md)
- [ADR-0010: Structural worker protocol and isolation](0010-structural-worker-protocol-and-isolation.md)
- [Phase 1 PRD](../product/phase-1-language-configuration-and-client-admission-prd.md)
- [Dependency policy](../dependency-policy.md)
