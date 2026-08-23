# ADR-0027: Deliberately bounded YAML configuration evidence

- Status: Accepted
- Date: 2026-08-23
- Scope: Phase 1 YAML configuration syntax evidence in the existing isolated
  structural worker

## Context

YAML is common in deployment, continuous-integration, infrastructure, and
application configuration. Its concrete syntax can anchor a user in source,
but YAML aliases, anchors, tags, merge behavior, flow/block styles, directives,
multiple documents, and consumer-specific schemas make generic configuration
interpretation unsafe and misleading.

The worker contract requires exact source spans and pinned parser provenance.
The accepted Phase 1 outcome therefore needs an isolated syntax parser, not a
configuration evaluator or a consumer-specific YAML runtime.

## Decision

Admit `.yaml` and `.yml` artifacts through the existing short-lived structural
worker using the pinned MIT-licensed `tree-sitter-yaml` 0.7.2 grammar and the
existing `tree-sitter` 0.26.12 runtime. The worker accepts only the exact
grammar identity `tree-sitter-yaml-0.7.2` and records it in every fact's
provenance.

The worker emits only raw direct-scalar mapping-key declarations and syntactic
containment from block and flow mapping pairs. A key is eligible only when its
syntax node contains exactly one plain, single-quoted, or double-quoted scalar.
The unquoted merge key `<<` is excluded. The emitted name is raw source text;
it is not a decoded, normalized, resolved, or evaluated YAML value.

When the parser reports syntax recovery/error for a YAML request, the worker
emits no YAML facts and returns the existing source-free
`syntax_recovery_present` warning. Existing source-hash, response validation,
fact-count, depth, response-size, worker-identity, and graph limits remain
required.

No alias or anchor expansion, tag handling, merge resolution, directive or
multi-document interpretation, scalar-value decoding, schema validation,
include resolution, environment access, toolchain/runtime invocation, process
execution, network access, source mutation, or configuration-to-code semantic
claim is added.

## Consequences

- YAML is a supported Phase 1 configuration family only for exact mapping-key
  and nesting anchors.
- A deployment, CI, or application YAML file does not establish that any
  platform loads it, that aliases or merges resolve as shown, or that a setting
  has runtime effect.
- The worker-only native grammar is subject to lockfile, SBOM, license,
  advisory, and cross-platform verification.
- Complex keys, aliases, anchors, tags, merge behavior, sequences, scalar
  values, and semantic references remain explicit non-claims.

## Verification

- A wrong YAML grammar identity is rejected.
- Valid block and flow mappings emit direct scalar mapping keys and containment
  facts with `tree-sitter-yaml-0.7.2` provenance.
- Alias values are not resolved, and the unquoted merge key emits no fact.
- Syntax-malformed YAML emits no facts and exposes recovery.
- Engine admission recognizes both YAML extensions and compatibility metadata
  cannot overclaim the shipped inventory.
- Dependency, license, SBOM, evaluation, worker-isolation, source-immutability,
  lint, and Tier A hosted platform gates remain required.

## Alternatives considered

### Evaluate YAML according to each consumer

Rejected. Consumer behavior requires schemas, runtime context, tooling, and
environment semantics that violate the evidence-only boundary.

### Decode or resolve aliases, anchors, tags, and merges

Rejected. It would turn local syntax evidence into a partially implemented YAML
evaluator and risk materially misleading configuration claims.

### Treat YAML as lexical-only

Rejected. Exact bounded mapping-key syntax evidence meets the approved Phase 1
outcome without claiming broader YAML semantics.

## References

- [ADR-0004: Source-language and parser strategy](0004-source-language-and-parser-strategy.md)
- [ADR-0010: Structural worker protocol and isolation](0010-structural-worker-protocol-and-isolation.md)
- [Phase 1 PRD](../product/phase-1-language-configuration-and-client-admission-prd.md)
- [Dependency policy](../dependency-policy.md)
