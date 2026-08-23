# ADR-0020: Strict-JSON configuration evidence

- Status: Accepted
- Date: 2026-08-23
- Scope: Phase 1 static configuration evidence in the existing isolated structural worker

## Decision

Admit only these strict-JSON configuration filenames to the structural worker:
`package.json`, `deno.json`, `composer.json`, and `manifest.json`. Use the
pinned `tree-sitter-json 0.24.8` grammar with the existing `tree-sitter
0.26.12` runtime. The worker emits a bounded, source-provenanced fact for each
JSON object key and uses the existing containment relation for nested object
keys.

The adapter uses the canonical decoded JSON key string, not a guessed or
runtime-expanded value. Its public fact class remains the graph's generic named
source binding class; the fact's `syntax_kind` is `pair`, so consumers can
distinguish a configuration key from a code declaration.

## Constraints

- This is strict JSON syntax only. JSONC, JSON5, TOML, YAML, arbitrary JSON
  data, and unrecognized `.json` filenames remain outside structural support.
- No JSON Schema validation, configuration loading, inheritance, interpolation,
  environment access, package resolution, subprocess, network, or source-write
  authority is added.
- Key presence and syntactic nesting are facts about source text, not a claim
  that a consumer, package manager, browser, or runtime honors the setting.
- Parsing remains in the existing short-lived, capability-reduced worker and
  continues to expose syntax recovery instead of treating malformed input as
  valid configuration.

## Verification

- Unit coverage verifies recognized-filename admission, arbitrary JSON-data
  exclusion, decoded key extraction, and nested-key containment.
- Existing worker framing, hash, parser/grammar/resolver identity, cache,
  source-immutability, dependency, SBOM, and cross-platform checks remain
  mandatory.
- The added grammar is pinned and MIT licensed.

## Consequences

Phase 1 can surface small, durable configuration anchors without turning the
core into a configuration evaluator. Future non-JSON or JSONC support requires
its own grammar, resolver, evidence, and authority review.

## References

- [ADR-0004: Source-language and parser strategy](0004-source-language-and-parser-strategy.md)
- [ADR-0010: Structural worker protocol and isolation](0010-structural-worker-protocol-and-isolation.md)
- [ADR-0019: Python structural-language admission](0019-python-structural-language-admission.md)
