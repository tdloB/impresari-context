# ADR-0025: Bounded JSONC configuration evidence

- Status: Accepted
- Date: 2026-08-23
- Scope: Phase 1 comment-tolerant configuration evidence and strict-JSON
  syntax enforcement in the isolated structural worker

## Context

ADR-0020 admitted a narrow strict-JSON manifest set and deliberately deferred
JSONC. The pinned `tree-sitter-json` 0.24.8 grammar accepts line and block
comments as extras. Without a separate strict validation gate, using that
grammar for the existing `json` language could silently make comments appear
valid under a strict-JSON claim.

Modern TypeScript, editor, and development-container configuration commonly
uses JSONC. It is useful as a source of exact configuration-key and containment
anchors, but it must not be treated as a configuration evaluator or as evidence
that an editor, compiler, or runtime honors a setting.

## Decision

Retain `tree-sitter-json` 0.24.8 as the only JSON-family grammar and admit a
distinct `jsonc` worker language. It emits only decoded object-key declarations
and syntax-derived containment facts with the existing pinned parser, worker,
provenance, and graph contract.

`json` remains the narrow existing strict-JSON manifest set. Before it emits
facts, the worker requires the raw source bytes to parse as one complete
`serde_json::Value`. A strict-validation failure yields no facts, marks syntax
recovery, and emits a stable source-free `strict_json_validation_failed`
warning.

`jsonc` is admitted only for files with a `.jsonc` extension and these named
configuration surfaces:

- `tsconfig.json`, `jsconfig.json`, and `devcontainer.json`;
- `.vscode/settings.json`, `.vscode/tasks.json`, `.vscode/launch.json`, and
  `.vscode/extensions.json`.

No configuration evaluation, schema validation, inheritance, include
resolution, interpolation, environment access, process execution, package or
toolchain invocation, network access, source mutation, or
configuration-to-code semantic claim is added.

## Consequences

- Public language claims distinguish strict JSON from comment-tolerant JSONC.
- Existing strict-JSON facts now fail closed when the raw source is not strict
  JSON, even when Tree-sitter can recover a tolerant syntax tree.
- JSONC and strict-JSON requests have distinct language discriminators and
  worker identities while sharing the pinned grammar implementation.
- Arbitrary `.json` data remains outside structural configuration support;
  `.jsonc` support remains syntax-only and must not be read as runtime support.

## Verification

- Valid strict JSON preserves decoded-key and containment evidence.
- A comment in the strict-JSON set produces no facts and the stable strict
  validation warning.
- Valid admitted JSONC produces only decoded-key and containment facts.
- Malformed JSONC exposes explicit syntax recovery.
- Non-admitted `.json` paths remain excluded.
- Existing worker protocol, source-hash, cache, hostile-workspace,
  dependency/SBOM, evaluation, and cross-platform gates remain required.

## Alternatives considered

### Treat the existing tolerant grammar as strict JSON

Rejected. The pinned grammar accepts comments, so this would overstate the
syntax contract.

### Add a second JSONC parser dependency

Rejected. The existing pinned grammar provides the syntax required for this
bounded scope; another parser would add supply-chain and behavior-divergence
risk without evidence benefit.

### Admit all `.json` data

Rejected. A file extension alone does not establish that generic data is
configuration supported by a tool or runtime.

## References

- [ADR-0004: Source-language and parser strategy](0004-source-language-and-parser-strategy.md)
- [ADR-0010: Structural worker protocol and isolation](0010-structural-worker-protocol-and-isolation.md)
- [ADR-0020: Strict-JSON configuration evidence](0020-strict-json-configuration-evidence.md)
- [Phase 1 PRD](../product/phase-1-language-configuration-and-client-admission-prd.md)
