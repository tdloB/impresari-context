# Linux IAR-1B Candidate Maintenance PRD

- Status: Accepted for implementation
- Date: 2026-08-30
- Owner: Aaron Boldt
- Decision: ADR-0077
- Parent: ADR-0074 isolated analyzer runner

## Problem

PRs 131–133 established source-free synthetic Linux confinement evidence on
two architectures and three materially different kernel lines. Exact CI
receipts can become stale or cease to describe the current runner. Without a
closed maintenance contract, a historical pass could be misrepresented as
broad or current production support.

## Outcome

Publish one exact candidate manifest and a source-free evaluator. The evaluator
accepts only caller-supplied target observations and returns one of:
`compatible_candidate`, `stale_evidence`, `changed`, `missing_evidence`,
`unsupported`, or `unavailable`. Only `compatible_candidate` keeps the narrow
candidate claim active.

## Initial Scope

- Candidate scope: GitHub-hosted Ubuntu 24.04 x86_64 and arm64 exact images,
  kernel, architecture, Landlock ABI, profile, probe, composite check, and
  evidence receipt recorded in the released manifest.
- Diversity evidence only: the Ubuntu 22.04 and 26.04 exact-host receipts from
  PR 133. These can demonstrate kernel diversity but cannot become candidate
  support through this evaluator.
- Freshness: each evidence unit has an explicit `observed_at` and
  `fresh_through`. Expiry withdraws the candidate claim.

## Safety And Authority

The evaluator reads only its manifest. It does not inspect the host, source,
repository, cache, credentials, processes, services, network, or environment.
It cannot repair drift, rerun CI, use privilege, start a worker, execute an
analyzer, or admit production. Every receipt fixes `production_admitted=false`,
`real_analyzer_authorized=false`, and all authority fields to `denied`.

## Acceptance

- Closed JSON Schema for manifests and health receipts.
- Exact SHA-256 binding to the frozen profile, native synthetic probe,
  composite check, and every evidence fixture.
- Deterministic tests for all six public states plus diversity-only rejection.
- Malformed manifests and production overclaims fail closed.
- Original-synthetic fixture provenance remains complete.
- Documentation distinguishes candidate health from production support and
  keeps IAR-2 closed.

## Explicit Non-Goals

Production Linux admission, broad distro/kernel support, host discovery,
background monitoring, persistent services, automatic remediation, real
analyzer execution, repository content as analyzer input, network access,
credentials, and release publication are not part of this increment.
