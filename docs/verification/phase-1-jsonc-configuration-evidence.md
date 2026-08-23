# Phase 1 JSONC configuration-evidence record

- Date: 2026-08-23
- Scope: Bounded JSONC structural evidence and strict-JSON syntax correction.
- Governing records: [Phase 1 PRD](../product/phase-1-language-configuration-and-client-admission-prd.md),
  [ADR-0020](../decisions/0020-strict-json-configuration-evidence.md), and
  [ADR-0025](../decisions/0025-jsonc-configuration-evidence.md).

## Delivered contract

| Surface | Supported evidence | Explicit limit |
| --- | --- | --- |
| Named strict-JSON manifests | Decoded object keys and syntactic containment only | Raw bytes must parse as one complete strict JSON value; comments and other non-strict forms emit no facts. |
| `.jsonc` files | Decoded object keys and syntactic containment only | Syntax is not configuration evaluation or a runtime claim. |
| Named JSONC convention files | `tsconfig.json`, `jsconfig.json`, `devcontainer.json`, and selected `.vscode/*.json` key/containment evidence | No compiler/editor/container/include/interpolation/configuration-to-code semantics. |

## Evidence

- The existing pinned `tree-sitter-json` 0.24.8 grammar is retained; no new
  parser or runtime dependency is introduced.
- Unit coverage proves strict JSON rejects comment-tolerant syntax without
  facts, valid JSONC emits decoded keys and containment, malformed JSONC
  signals syntax recovery, and arbitrary `.json` data remains excluded.
- The worker continues to receive bounded source bytes only; parser identity,
  source hash, provenance, fact limits, response limits, and isolation remain
  unchanged.
- The full local release-assurance gate passes: repository policy, security
  boundaries, contracts, frozen SBOM, linting, all workspace tests, evaluation,
  restart, and documentation checks.
- The updated engine was dogfooded against this repository. Its bounded context
  packet completed against a current partial snapshot and explicitly retained
  the `snapshot_partial` unknown rather than treating excluded or oversized
  files as complete coverage.

## Non-claims

This work does not load configuration, resolve includes or references, validate
schemas, expand interpolation, access an environment, invoke an editor,
compiler, container, package manager, or runtime, execute source, mutate the
workspace, or make configuration-to-code semantic claims.

## Roadmap checkpoint

The Master PRD, Phase 1 PRD, ADR-0018, ADR-0020, ADR-0023, and the new
ADR-0025 were reassessed after this completed slice. The strict-JSON correction
removes an implementation/claim mismatch, and bounded JSONC is already
approved Phase 1 scope. No further roadmap sequencing, authority-boundary, or
admission-criterion change is warranted. The next language/configuration work
remains TOML, then deliberately limited YAML; client admission remains a
separate evidence track.
