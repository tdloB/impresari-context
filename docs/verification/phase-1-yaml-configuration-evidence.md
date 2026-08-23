# Phase 1 YAML configuration-evidence record

- Date: 2026-08-23
- Scope: Deliberately bounded YAML mapping-key evidence in the existing
  isolated worker.
- Governing records: [Phase 1 PRD](../product/phase-1-language-configuration-and-client-admission-prd.md),
  [ADR-0027](../decisions/0027-yaml-configuration-evidence.md), and the
  [dependency policy](../dependency-policy.md).

## Delivered contract

| Surface | Supported evidence | Explicit limit |
| --- | --- | --- |
| `.yaml` and `.yml` artifacts | Raw direct-scalar block/flow mapping keys and syntactic containment with exact snapshot spans | No scalar decoding, alias/anchor/tag expansion, merge behavior, directives, multi-document interpretation, sequences, schemas, or consumer/runtime semantics. |
| Syntax-malformed YAML | Explicit syntax-recovery state and no structural facts | Recovery must never become partial YAML configuration evidence. |

## Evidence

- `tree-sitter-yaml` 0.7.2 is pinned as a worker-only MIT grammar with the
  current `tree-sitter` 0.26.12 runtime. The lockfile and frozen SPDX SBOM now
  contain 189 packages.
- Unit coverage proves direct block and flow mapping-key extraction, nested
  containment, grammar identity rejection, unquoted merge-key omission,
  unexpanded alias values, syntax-malformed no-fact behavior, and both YAML
  extension admissions.
- The complete local release-assurance gate passes: repository policy, security
  boundaries, contracts, frozen SBOM, linting, all workspace tests, evaluation,
  restart, and documentation checks.
- Dependency advisory, license, source, and duplicate review passes for the
  189-package lockfile. Existing duplicate transitive packages remain visible
  in the audit output; no advisory, license, or source exception was accepted.
- An updated engine dogfood run against this checkout, using a separate fresh
  cache, produced packet
  `sha256:856dbb138f028ed8d00ee82f085a314083bfa0197ca9cedba7ac460a9c0f7b72`
  for the literal query `tree-sitter-yaml`. The packet retains the explicit
  `snapshot_partial` uncertainty caused by two policy-excluded and fifteen
  oversized files.
- The worker receives bounded snapshot bytes only. Existing hash binding,
  provenance, request/response validation, fact limits, nesting limits,
  source-immutability, and lower-authority process isolation are unchanged.

## Non-claims

YAML evidence does not decode scalar values, resolve or follow aliases,
anchors, tags, or merge keys, interpret sequences, directives, documents, or
schemas, load a YAML consumer, access an environment, invoke a package manager,
compiler, toolchain, runtime, CI, deployment system, or network service,
execute repository code, mutate the workspace, or infer configuration-to-code
behavior.

## Roadmap checkpoint

The Master PRD, Phase 1 PRD, revised roadmap, ADR-0004, ADR-0010, ADR-0018,
ADR-0023, ADR-0026, and ADR-0027 were reassessed after this slice. YAML's
complex semantics are contained by the direct-scalar mapping-key rule; no new
authority, evaluation, or roadmap sequence is introduced. All Phase 1 language
and configuration evidence is now implemented pending hosted verification. The
remaining Phase 1 outcome is first-class client admission for Codex, Claude
Code, and Cursor; it remains a separate evidence track.
