# Phase 1 TOML configuration-evidence record

- Date: 2026-08-23
- Scope: Bounded TOML structural evidence in the existing isolated worker.
- Governing records: [Phase 1 PRD](../product/phase-1-language-configuration-and-client-admission-prd.md),
  [ADR-0026](../decisions/0026-toml-configuration-evidence.md), and the
  [dependency policy](../dependency-policy.md).

## Delivered contract

| Surface | Supported evidence | Explicit limit |
| --- | --- | --- |
| `.toml` configuration artifacts | Raw syntax-derived keys, tables, table-array elements, and syntactic containment with exact snapshot spans | No value interpretation, key normalization, include resolution, interpolation, package/toolchain resolution, build-script behavior, or runtime semantics. |
| Malformed TOML | Explicit syntax-recovery state and no structural facts | Recovery must never be treated as partial configuration evidence. |

## Evidence

- `tree-sitter-toml-ng` 0.7.0 is pinned as a worker-only MIT grammar with the
  current `tree-sitter` 0.26.12 runtime. The lockfile and frozen SPDX SBOM now
  contain 188 packages.
- Unit coverage proves valid key/table/table-array/containment extraction,
  rejected incorrect grammar identity, malformed-TOML no-fact behavior, and
  `.toml` engine admission.
- The complete local release-assurance gate passes: repository policy, security
  boundaries, contracts, frozen SBOM, linting, all workspace tests, evaluation,
  restart, and documentation checks.
- The project dependency advisory, license, source, and duplicate review passes
  for the 188-package lockfile; pre-existing duplicate warnings remain visible
  and no advisory, license, or source failure is accepted.
- The updated engine was dogfooded against this repository. A separate fresh
  cache produced packet `sha256:dfb208a6dc894e78e7a1e75bc5c0841f0a1e0b3b182100bef96d9ecebf4a26c3`
  for the literal `tree-sitter-toml-ng` query. The packet explicitly retained
  `snapshot_partial` rather than claiming coverage of 2 policy-excluded and 15
  oversized artifacts.
- The worker receives bounded snapshot bytes only. Existing hash binding,
  provenance, request/response validation, fact limits, nesting limits,
  source-immutability, and lower-authority process isolation are unchanged.

## Non-claims

TOML evidence does not load a configuration consumer, resolve includes,
interpolate environment values, evaluate values, validate schemas, invoke a
package manager, compiler, toolchain, build script, editor, runtime, or
network service, execute repository code, mutate the workspace, or infer
configuration-to-code behavior.

## Roadmap checkpoint

The Master PRD, Phase 1 PRD, revised roadmap, ADR-0004, ADR-0010, ADR-0018,
ADR-0023, and ADR-0026 were reassessed after this slice. The source-span
requirement rules out semantic-only TOML extraction but is satisfied by the
existing isolated worker pattern. This is a local dependency and parser
admission; it does not change roadmap sequencing, client authority, or the
product trust boundary. The next Phase 1 language/configuration work remains
deliberately bounded YAML, with client admissions continuing as separate
evidence tracks.
