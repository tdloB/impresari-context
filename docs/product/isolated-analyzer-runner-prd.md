# Isolated Analyzer Runner — Product Requirements Document

## Document Control

- Product: Impresari Analyzer Runner for Impresari Context.
- PRD ID/version: IC-IAR-PRD-001 / 0.1.
- Status: Proposed; documentation and planning only. Implementation is not
  authorized by this record.
- Date: 2026-08-26.
- Owner: Aaron Boldt.
- Sequence: Security expansion step 2 of 3; depends on accepted step 1 contracts.
- Related records:
  - [Hostile Repository Admission PRD](hostile-repository-admission-prd.md)
  - [Hostile Repository Admission ARD](../architecture/hostile-repository-admission-ard.md)
  - [Isolated Analyzer Runner ARD](../architecture/isolated-analyzer-runner-ard.md)
  - [ADR-0074](../decisions/0074-separate-isolated-analyzer-runner.md)
  - [ADR-0013](../decisions/0013-extension-contracts-without-code-loading.md)
  - [Disposable Quarantine Runner PRD](disposable-quarantine-runner-prd.md)

## Executive Decision

Build analyzer execution as a separate, independently releasable component that
receives only snapshot-bound staged artifacts, runs each approved analyzer with
minimum capabilities, and returns bounded hostile output through the Context
normalization boundary.

The initial implementation is vendor-neutral with reference adapters for local
ClamAV and YARA, followed by first-party repository execution-surface and
Windows static analyzers. Optional threat-intelligence enrichment follows
Privacy B: local analysis first, exact SHA-256-only provider queries through a
separate gateway, and no file upload.

## Problem

Static admission requires specialized tools that cannot safely run inside the
Context policy process. Malware scanners and binary/installer parsers consume
attacker-controlled bytes, may contain native code, can crash or be exploited,
and often expect filesystem access. Threat-intelligence lookups add credentials,
network disclosure, provider terms, freshness, and availability concerns.

The system needs useful scanning without giving a scanner access to the original
repository, Context cache, home directory, credentials, network, or canonical
evidence stores.

## Product Boundary

The Analyzer Runner is not part of the trusted Context core. It is a local
supervisor and worker boundary for static analysis only.

It may inspect staged artifact bytes. It may not execute the repository's code
as an application, install packages, run builds/tests, modify source, grant
admission exceptions, or authorize ordinary-host execution.

## Goals

1. Execute approved static analyzers outside the Context process.
2. Keep each analyzer isolated from the repository root and other analyzers.
3. Support vendor-neutral analyzer manifests and normalized results.
4. Ship useful open-first reference adapters without requiring a paid service.
5. Provide initial Windows-oriented deep static analysis on macOS and Linux.
6. Enforce complete resource, environment, filesystem, process, and network
   boundaries.
7. Implement Privacy B hash reputation without artifact upload.
8. Make scanner failure, stale rules, skipped artifacts, and partial coverage
   visible to deterministic admission policy.
9. Allow adapters and rule databases to be disabled independently.

## Non-Goals

The initial Analyzer Runner will not:

- execute repository binaries, scripts, installers, package lifecycle hooks,
  build definitions, tests, macros, or embedded active content;
- unpack without bounded depth, byte, count, type, and recursion policies;
- upload files, excerpts, filenames, paths, or repository metadata;
- query threat intelligence directly from an analyzer worker;
- provide real-time endpoint antivirus or on-access scanning;
- claim that a signed executable is benign;
- treat one provider or scanner as authoritative;
- download unreviewed rules or binaries during a scan;
- run a general third-party plugin supplied by the inspected repository;
- make final risk-acceptance decisions.

## Initial Analyzer Portfolio

### Required reference analyzers

| Analyzer | Initial role | Execution posture |
| --- | --- | --- |
| ClamAV adapter | Known signature, archive, file-type, and supported heuristic detection | Short-lived local worker using a pinned engine and signed database identity |
| YARA adapter | Approved textual/binary repository and malware-family rules | Short-lived local worker using a pinned ruleset; includes disabled by default |
| Execution-surface analyzer | Package hooks, CI, IDE, container, shell, credential, persistence, and download patterns | First-party deterministic worker |
| Binary inventory analyzer | Executable/library format metadata, hashes, entropy/packing signals, imports and sections where safely supported | First-party or admitted library in isolated worker |
| Dependency provenance analyzer | Lockfile and manifest sources, mutable Git/URL dependencies, scripts, and integrity gaps | First-party deterministic worker; no package installation |

### Initial Windows-oriented analyzers

| Capability | Required output |
| --- | --- |
| PowerShell static | AST/token facts, encoded/dynamic invocation signals, download/credential/persistence capabilities, parse limitations |
| Batch static | Command-chain, interpreter, download, environment, persistence, and obfuscation signals |
| PE/DLL metadata | Headers, sections, imports/exports, resources, overlay, debug/signature metadata, packer/entropy indicators |
| Authenticode inspection | Signature presence, verification evidence available to the tool, signer/timestamp metadata, and explicit non-benign limitation |
| MSI inspection | Tables, embedded payload inventory, execution sequence, scripts, binaries, and custom actions without installation |
| Windows persistence static | Service, scheduled task, startup, registry, WMI, and related declarations statically observable in the repository |
| Windows build surface | MSBuild targets/tasks, NuGet hooks, packaging/signing steps, downloaded tools, and custom execution |

Exact parser/library selection is an implementation admission decision, not a
PRD promise. Every native parser remains inside a worker fault domain.

## Threat-Intelligence Strategy

### Required local baseline

- ClamAV engine and signed database files updated by a separate controlled
  updater.
- Approved YARA rules with exact source, license, version, digest, reviewer, and
  activation state.
- Optional locally downloaded known-file/reputation datasets where licensing and
  operational size permit.
- No MISP requirement.

### Optional hash-only providers

Initial provider adapters may include community services such as MalwareBazaar
and CIRCL Hashlookup. Commercial providers may be added later through the same
contract. Free public services must not be used contrary to commercial-use,
rate-limit, or redistribution terms.

Provider lookup rules:

1. Local analysis and local cache lookup happen first.
2. Only a lowercase canonical SHA-256 value is submitted.
3. The request contains no source, filename, path, repository, owner, revision,
   artifact size, scanner finding, or user-entered text.
4. Each provider is explicitly enabled by consumer policy.
5. Provider credentials are held only by the reputation gateway.
6. Responses are provenance-bound, cached with TTL, and never treated as exact
   source authority.
7. No match means unknown, not benign.
8. File submission and sample download are prohibited.

MISP remains an optional later adapter for organizations that already operate a
trusted instance or require multi-feed correlation. Impresari will not require
users to deploy MISP for the initial product.

## Users And Jobs

| User | Job |
| --- | --- |
| Repository evaluator | Run approved scanners without exposing the normal workstation |
| Security reviewer | Compare evidence, detections, rulesets, and coverage |
| Integrator | Add an analyzer behind a stable vendor-neutral contract |
| Administrator | Pin analyzer/ruleset/provider versions and revoke them |
| Admission policy owner | Consume normalized complete/partial/failed analysis states |

## Critical User Journeys

### Journey 1 — Run the open local baseline

1. Context produces an exact assessment plan and artifact manifest.
2. The Runner validates its own policy, analyzer pins, resource profile, and
   snapshot identities.
3. It stages only required artifact bytes into a private job area.
4. ClamAV, YARA, and first-party workers run separately with no network or
   credentials.
5. The Runner validates and normalizes complete outputs.
6. Context incorporates results as untrusted derived evidence and recomputes
   coverage.

### Journey 2 — Analyze Windows artifacts on macOS or Linux

1. The assessment requests applicable Windows capability IDs.
2. The Runner selects compatible platform-independent analyzers.
3. Each worker consumes staged bytes without invoking Windows loaders or tools
   that execute the artifact.
4. Results identify exact artifact hashes, parser provenance, observed metadata,
   heuristics, and unsupported structures.
5. Any unavailable Windows analysis remains visible and blocks eligibility when
   policy requires it.

### Journey 3 — Perform optional hash reputation

1. Local scanners complete first.
2. Policy identifies eligible artifact hashes and enabled providers.
3. The separate reputation gateway checks its fresh local cache.
4. A cache miss sends only SHA-256 to the selected provider.
5. The response is normalized with provider, query time, dataset freshness,
   limitations, and retention disclosure.
6. Failure or no match remains unknown.

### Journey 4 — Handle a hostile or broken analyzer

1. A worker crashes, times out, forks, writes excessively, changes staged input,
   emits malformed output, or attempts network access.
2. The supervisor kills the complete worker process tree.
3. No partial finding is promoted.
4. Only bounded source-free failure metadata is retained.
5. Coverage becomes failed/partial, and admission cannot silently advance.

## Functional Requirements

### A. Product and package separation

| ID | Requirement |
| --- | --- |
| IAR-FR-001 | Ship the Runner separately from the Context core process and release artifact |
| IAR-FR-002 | Use a versioned request/result protocol with exact binary, analyzer, ruleset, policy, and snapshot identities |
| IAR-FR-003 | Keep existing Context behavior usable when the Runner is absent, disabled, incompatible, or revoked |
| IAR-FR-004 | Never discover or load analyzer code from the inspected repository |

### B. Artifact staging and worker isolation

| ID | Requirement |
| --- | --- |
| IAR-FR-005 | Receive only a Context-authorized artifact manifest and exact bytes or content-addressed staged objects |
| IAR-FR-006 | Rehash every staged artifact and reject snapshot mismatch before analysis |
| IAR-FR-007 | Give each worker a fresh private input/output/temp area and no original repository root |
| IAR-FR-008 | Make staged input immutable to the worker where the platform supports it and detect mutation everywhere |
| IAR-FR-009 | Clear environment, close unrelated handles, use an empty/minimal PATH, and provide no home directory or credentials |
| IAR-FR-010 | Enforce CPU, memory, elapsed time, process count, file count, byte, recursion, output, and disk quotas |
| IAR-FR-011 | Kill the complete worker process tree on completion, cancellation, violation, or timeout |
| IAR-FR-012 | Deny worker network, socket, device, host-process, and unrelated filesystem access |

### C. Analyzer governance

| ID | Requirement |
| --- | --- |
| IAR-FR-013 | Require exact analyzer ID, version, publisher/provenance, artifact digest, contract, supported capabilities, and platform matrix |
| IAR-FR-014 | Require explicit input types, output schema, determinism, limits, dependencies, license, update channel, and revocation state |
| IAR-FR-015 | Admit each analyzer through security, dependency/SBOM, license, malformed-input, resource, and platform review |
| IAR-FR-016 | Disable an analyzer independently without changing other analyzers or the Context core |
| IAR-FR-017 | Never treat an artifact digest as verified publisher identity unless a separate signature/provenance check establishes it |

### D. Findings and coverage

| ID | Requirement |
| --- | --- |
| IAR-FR-018 | Validate complete output before accepting any finding |
| IAR-FR-019 | Bind each result to request, snapshot, artifact hashes, analyzer, ruleset/database, limits, and timestamps |
| IAR-FR-020 | Distinguish detection, heuristic, metadata, reputation match, parse failure, skipped content, and unsupported capability |
| IAR-FR-021 | Preserve per-artifact scan and skip counts rather than only an aggregate scanner exit code |
| IAR-FR-022 | Prevent analyzer output from claiming exact-source, policy, approval, or execution authority |
| IAR-FR-023 | Convert crash, timeout, stale database, and unavailable scanner into explicit coverage state |

### E. Threat intelligence and updater

| ID | Requirement |
| --- | --- |
| IAR-FR-024 | Separate signature/ruleset updating from scan execution |
| IAR-FR-025 | Verify downloaded database/ruleset integrity, provenance, license, freshness, and rollback policy before activation |
| IAR-FR-026 | Keep analyzer workers offline even when hash reputation is enabled |
| IAR-FR-027 | Permit hash egress only through a provider-specific gateway and explicit policy |
| IAR-FR-028 | Prohibit file upload, sample download, provider comments, and provider mutation APIs |
| IAR-FR-029 | Cache provider results by provider/hash/dataset with bounded TTL and deletion controls |
| IAR-FR-030 | Treat provider logging/retention and commercial-use terms as admission requirements |

## Security And Privacy Requirements

- A scanner compromise must not expose the source repository, Context cache,
  host home directory, credentials, or network.
- A ruleset can cause resource use or false detections; rules are executable
  policy inputs and require pinning, review, limits, and rollback.
- Unique hashes can reveal possession of a file. Online reputation is explicit,
  provider-specific, logged locally, and disableable.
- Default diagnostics contain no source, paths, hashes, rules, credentials, or
  raw scanner output.
- Raw analyzer output is retained only in an optional job-private bounded spool
  for local review and never enters canonical Context stores.

## Success Metrics

| Metric | Initial target |
| --- | --- |
| Analyzer access outside staged job area | Zero successful accesses in adversarial suite |
| Analyzer network access | Zero; reputation gateway tested separately |
| Source or staged-input mutation | Zero accepted mutations |
| Malformed/partial output promoted | Zero |
| Required artifact accounting | 100% scanned or explicitly skipped/failed |
| Ruleset/database identity recorded | 100% of results |
| Hash lookup payload fields beyond SHA-256 | Zero |
| File uploads/downloads | Zero |
| Windows static analyzer conformance | 100% on admitted fixture matrix |
| Core operation when Runner disabled | Unchanged and passing |

## Implementation Plan And Gates

### IAR-0 — Protocol and synthetic worker

- Freeze request, result, capability, analyzer manifest, and failure schemas.
- Implement only a synthetic no-op/fault worker in later authorized work.
- Prove framing, pinning, cancellation, resource, and all-or-nothing semantics.

Gate: ADR-0074 acceptance, threat-model expansion, and independent protocol
review.

### IAR-1 — Local supervisor confinement

- Implement per-job staging, process supervision, platform confinement, quotas,
  cleanup, and source-free audit.
- Use no real scanner or network.

Gate: Tier A escape, handle, process-tree, mutation, resource, crash, and cleanup
suites. Claims remain `application_enforced` where OS confinement is incomplete.

### IAR-2 — YARA reference adapter

- Add one pinned YARA version and one small reviewed Impresari ruleset.
- Disable includes, external paths, unapproved modules, and custom repository
  rules by default.

Gate: rule bomb, malformed binary, output flood, false-positive, license, SBOM,
and platform review.

### IAR-3 — ClamAV reference adapter

- Add short-lived `clamscan`-style execution with pinned signed database inputs.
- Keep `freshclam` or equivalent updater outside the scan job.

Gate: archive/scan limits, database rollback/freshness, detection accounting,
false-positive, crash, and platform review.

### IAR-4 — First-party repository and Windows analyzers

- Add one analyzer capability at a time, beginning with execution surfaces and
  bounded Windows artifact metadata.
- Admit every parser dependency independently.

Gate: per-capability fixture, threat, accuracy, coverage, dependency, and
platform evidence.

### IAR-5 — Privacy B gateway

- Add provider-neutral hash lookup with a synthetic provider first.
- Add community providers only after terms, privacy, retention, rate-limit, and
  commercial-use review.

Gate: packet capture proves SHA-256-only egress; redirect, DNS, proxy, TLS,
credential, log, cache, and provider-failure suites pass.

### IAR-6 — Step 2 private pilot

- Run only on synthetic, known-benign, and purpose-built adversarial fixtures.
- Compare scanners and record disagreement rather than collapsing it.

Gate: explicit founder approval before real untrusted repositories or Step 3
implementation.

## Acceptance Criteria

| ID | Given | When | Then |
| --- | --- | --- | --- |
| IAR-AC-001 | A malicious analyzer fixture | It attempts host access | Access is denied, worker is killed, and coverage fails closed |
| IAR-AC-002 | A PE/DLL/MSI fixture on macOS or Linux | Windows static analyzers run | Metadata/findings are returned without loading or executing the artifact |
| IAR-AC-003 | A malformed parser-crash fixture | Analysis runs | No partial result is promoted |
| IAR-AC-004 | Stale ClamAV/YARA data | Policy requires fresh analysis | Coverage is stale/incomplete |
| IAR-AC-005 | Online reputation enabled | A cache miss is queried | Network capture contains only provider protocol metadata and SHA-256 |
| IAR-AC-006 | Provider says hash not found | Result is normalized | Reputation remains unknown |
| IAR-AC-007 | Provider or scanner is disabled | Assessment is rebuilt | Required coverage reports unavailable and no fallback provider is enabled |
| IAR-AC-008 | Analyzer Runner is absent | Existing Context commands run | Existing behavior remains operational |

## Rollback And Disable Strategy

- Revoke an analyzer binary, ruleset, database, or provider independently by
  exact identity.
- Stop new jobs before changing an activated version.
- Preserve old result provenance while marking it stale or revoked.
- Delete job-private staged bytes and raw output under a documented retention
  policy; do not delete source.
- Disable all online reputation without disabling local scanners.
- Disable the entire Runner without changing Context retrieval or assessments
  based solely on core observations.

## Approval Boundary

This PRD provides implementation planning but authorizes no implementation,
scanner installation, rule download, network query, or analysis of a real
untrusted repository. Each implementation phase requires its stated gate and a
separate founder approval.
