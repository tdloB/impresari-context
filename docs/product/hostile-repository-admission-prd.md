# Hostile Repository Admission Foundation — Product Requirements Document

## Document Control

- Product: Impresari Context.
- PRD ID/version: IC-HRA-PRD-001 / 0.1.
- Status: HRA-0 and HRA-1 implemented; HRA-2 implementation in progress.
- Date: 2026-08-26.
- Owner: Aaron Boldt.
- Sequence: Security expansion step 1 of 3.
- Related records:
  - [Master Product PRD](master-prd.md)
  - [System Boundaries](../boundaries.md)
  - [Security Threat Model](../security/threat-model.md)
  - [Hostile Repository Admission ARD](../architecture/hostile-repository-admission-ard.md)
  - [ADR-0073](../decisions/0073-evidence-only-hostile-repository-admission.md)
  - [Isolated Analyzer Runner PRD](isolated-analyzer-runner-prd.md)
  - [Disposable Quarantine Runner PRD](disposable-quarantine-runner-prd.md)

## Executive Decision

Add an evidence-only hostile-repository admission foundation to Impresari
Context without allowing the Context core to load analyzers, execute repository
code, contact threat-intelligence services, or decide that a repository is
safe.

The foundation inventories security-relevant artifacts and execution surfaces,
records exact evidence and coverage, accepts separately produced analyzer
results through a versioned untrusted-output boundary, and emits an immutable
assessment. A separate deterministic policy evaluator may classify the next
permitted stage as `blocked`, `manual_review_required`,
`analysis_incomplete`, or `isolated_execution_eligible`. Only the last state
may admit work to the future Quarantine Runner; it never authorizes execution
on an ordinary host.

## Problem

Impresari Context can safely retrieve evidence from an untrusted repository,
but its current contracts do not answer the narrower admission questions needed
before someone installs, builds, tests, previews, or runs an unfamiliar MVP:

- Which files could initiate execution?
- Which platform-specific artifacts require specialized inspection?
- Which expected analyses ran, failed, skipped files, or were unavailable?
- Which exact evidence supports a security finding?
- Does policy permit the repository to advance to isolated execution?

A malware scanner alone cannot answer these questions. A repository may contain
no known malware signature while still using an install hook, IDE task,
privileged container definition, CI workflow, or script to access credentials
or download a payload.

## Product Boundary

This step extends the evidence product. It does not create an antivirus engine,
plugin runtime, package installer, build runner, virtual machine manager, or
general security approval authority.

| Responsibility | Owner in this step |
| --- | --- |
| Immutable repository snapshot and exact evidence | Impresari Context core |
| Security-relevant artifact and execution-surface inventory | Impresari Context core |
| Analyzer requirement and coverage ledger | Impresari Context core |
| Malware, YARA, binary, dependency, or deep platform analysis | Future Isolated Analyzer Runner |
| Deterministic stage-eligibility evaluation | Separate authority-free policy evaluator or consumer |
| Risk exception and ordinary-host execution decision | Authorized human/consumer |
| Dynamic execution and observation | Future Disposable Quarantine Runner |

## Goals

1. Make security-relevant repository contents visible without executing them.
2. Preserve exact, snapshot-bound evidence for every observed finding.
3. Represent required, completed, failed, skipped, and unavailable analysis.
4. Support Windows-oriented static admission from macOS and Linux hosts.
5. Normalize externally supplied analyzer results as untrusted derived data.
6. Produce deterministic, explainable stage-eligibility decisions.
7. Preserve the current read-only, no-network, no-arbitrary-process core.
8. Make incomplete analysis fail closed without claiming maliciousness.
9. Create stable contracts that later scanner and quarantine components can use.

## Non-Goals

This step will not:

- execute repository source, binaries, installers, scripts, builds, tests, or
  package-manager operations;
- load or invoke ClamAV, YARA, PowerShell, package managers, native binary
  parsers, or third-party analyzer artifacts;
- upload files or hashes to an external service;
- declare a repository `clean`, `safe`, `trusted`, or malware-free;
- interpret an Authenticode signature as proof of benign behavior;
- authorize ordinary-host, production, credentialed, or networked execution;
- automatically remediate, delete, rewrite, quarantine, or commit source files;
- replace endpoint protection, incident response, or a professional malware
  investigation;
- weaken ADR-0013's prohibition on extension artifact loading and execution.

## Users And Jobs

| User | Job |
| --- | --- |
| Founder or evaluator | Submit an unfamiliar MVP for safe, static admission preparation |
| Security reviewer | Trace every finding to exact source or artifact identity |
| AI coding client | Receive bounded findings without treating repository text as instructions |
| Analyzer integrator | Learn which exact artifacts and analyses are required by a stable contract |
| Policy administrator | Define deterministic stage rules outside repository content |
| Auditor | Reproduce the snapshot, coverage, policy, and decision provenance |

## Terminology

- **Artifact inventory:** Snapshot-bound facts about regular files, formats,
  sizes, hashes, permissions, and security-relevant classifications.
- **Execution surface:** A file or declaration that may cause code to run, such
  as a package lifecycle hook, build task, installer action, IDE task, service,
  or container entrypoint.
- **Finding:** An observed or derived security-relevant statement with severity,
  confidence, evidence, method, and limitations.
- **Coverage ledger:** The complete set of required analyses and their states.
- **Assessment:** An immutable combination of inventory, findings, coverage,
  unknowns, and provenance.
- **Admission decision:** A deterministic statement about the next permitted
  stage, not a claim of safety.

## Initial Scope

### Repository and cross-platform inventory

The initial inventory must identify, hash, and classify at least:

- source, script, configuration, archive, document, executable, library,
  installer, and unknown/binary artifacts;
- executable permission indicators where the platform exposes them;
- package manifests, lockfiles, lifecycle scripts, build definitions, task and
  debugger configuration, dev-container files, Dockerfiles, Compose files, CI
  workflows, Git submodule declarations, and agent-instruction files;
- ambiguous extensions, mismatched file magic, nested archives, links, special
  objects, oversized files, encrypted content, and unsupported formats;
- potential credential, home-directory, Docker-socket, host-namespace,
  privileged-container, network-download, and shell-interpreter references.

The inventory reports facts and analyzer requirements. It does not infer
malicious intent from a filename, extension, executable bit, or single string.

### Initial Windows-oriented static support

Windows-oriented repositories are in the initial product scope even when the
Context host is macOS or Linux.

| Artifact or behavior | Step 1 responsibility |
| --- | --- |
| PowerShell `.ps1`, module, data, and manifest files | Identify, hash, retrieve exact text, and mark for isolated PowerShell analysis |
| Batch and command scripts | Identify, hash, retrieve exact text, and inventory apparent execution surfaces |
| PE executables and DLLs | Identify by extension and bounded magic; hash; do not load or deeply parse |
| MSI packages | Identify and hash; mark installer tables and custom actions as required future analyses |
| Service declarations | Locate service configuration and code references where statically expressed |
| Registry modification | Locate `.reg`, script, installer, and configuration references where statically expressed |
| Windows build configuration | Inventory MSBuild, solution/project, NuGet, packaging, signing, and installer definitions |

Deep PE/COFF, Authenticode, MSI database, PowerShell AST, and Windows-specific
behavior analysis belongs to isolated workers in step 2 because their parsers
consume hostile input.

## Critical User Journeys

### Journey 1 — Prepare an unfamiliar repository

1. The user selects one explicit local workspace and optional expected revision.
2. Context opens it through the existing path-authorization boundary.
3. Context creates or verifies an immutable snapshot.
4. The admission inventory classifies eligible artifacts within declared
   limits.
5. The user receives inventory completeness, exclusions, and unknowns without
   any repository code running.

### Journey 2 — Inspect execution surfaces

1. The user requests an admission assessment for the snapshot.
2. Context finds supported execution-surface declarations using bounded,
   deterministic rules.
3. Every observation references exact bytes or artifact identity.
4. Ambiguous or unsupported cases become required analyses or unknowns.
5. Repository instructions cannot change the rules or suppress findings.

### Journey 3 — Incorporate analyzer results later

1. A future isolated runner returns a digest-bound result envelope.
2. Context verifies schema, analyzer identity, snapshot, artifact hashes, limits,
   and declared evidence references.
3. Valid results enter the assessment as `untrusted_derived_data`.
4. Invalid or authority-claiming output becomes metadata-only quarantine.
5. The core never treats analyzer text as control or exact-source authority.

### Journey 4 — Determine the next permitted stage

1. A consumer submits an immutable assessment and a versioned admission policy
   to the separate deterministic evaluator.
2. The evaluator verifies identities, freshness, required coverage, and rule
   compatibility.
3. It returns exactly one stage decision plus rule-level reasons.
4. An exception requires an authorized human and a new auditable decision.
5. No decision from this flow authorizes ordinary-host execution.

## Functional Requirements

### A. Snapshot and inventory

| ID | Requirement |
| --- | --- |
| HRA-FR-001 | Bind every inventory, finding, assessment, and decision to one exact workspace snapshot |
| HRA-FR-002 | Reuse existing authorized, read-only, regular-file access and never broaden the workspace root |
| HRA-FR-003 | Identify content type using bounded evidence and report extension/magic disagreement |
| HRA-FR-004 | Record every skipped, unsupported, oversized, encrypted, linked, or ambiguous artifact |
| HRA-FR-005 | Preserve file size, content hash, lossless path identity, and applicable permission metadata |
| HRA-FR-006 | Apply file, byte, traversal, time, memory, output, archive-depth, and finding-count limits |

### B. Execution-surface evidence

| ID | Requirement |
| --- | --- |
| HRA-FR-007 | Inventory package lifecycle hooks without invoking a package manager |
| HRA-FR-008 | Inventory build, test, task, debugger, editor, dev-container, container, and CI execution declarations |
| HRA-FR-009 | Detect declarative privileged mounts, host namespaces, Docker socket access, and broad filesystem access where statically observable |
| HRA-FR-010 | Identify download-and-execute, shell-evaluation, credential-path, persistence, and external-command patterns as review signals, not malware proof |
| HRA-FR-011 | Treat comments, documentation, filenames, and agent instructions as untrusted content that cannot alter admission policy |
| HRA-FR-012 | Distinguish observed configuration from derived suspiciousness and unknown intent |

### C. Windows-oriented evidence

| ID | Requirement |
| --- | --- |
| HRA-FR-013 | Recognize PowerShell, batch, PE, DLL, MSI, registry, service, MSBuild, NuGet, and Windows packaging artifacts |
| HRA-FR-014 | Require isolated deep analysis for PE/DLL/MSI and any parser not admitted to the Context core |
| HRA-FR-015 | Report platform-specific unsupported and partial states without treating a non-Windows host as proof of non-executability |
| HRA-FR-016 | Never invoke Windows loaders, installers, script hosts, registry tools, or signature-verification executables |

### D. Findings, coverage, and assessment

| ID | Requirement |
| --- | --- |
| HRA-FR-017 | Use closed finding categories, severities, classifications, methods, and confidence values |
| HRA-FR-018 | Require exact evidence for observed source findings and artifact hashes for binary findings |
| HRA-FR-019 | Record analyzer identity, artifact digest, ruleset/database identity, timestamps, limits, and result digest for derived findings |
| HRA-FR-020 | Represent required analysis as `not_requested`, `pending`, `completed`, `partial`, `failed`, `unavailable`, or `stale` |
| HRA-FR-021 | Prevent zero detections from implying complete coverage or safety |
| HRA-FR-022 | Produce an immutable assessment with findings, coverage, exclusions, unknowns, conflicts, and limitations |
| HRA-FR-023 | Reject stale or cross-snapshot analyzer results |

### E. Deterministic admission policy

| ID | Requirement |
| --- | --- |
| HRA-FR-024 | Evaluate only an exact assessment and exact policy version through a non-AI deterministic evaluator |
| HRA-FR-025 | Return only `blocked`, `manual_review_required`, `analysis_incomplete`, or `isolated_execution_eligible` |
| HRA-FR-026 | Include every matched policy rule and missing prerequisite in the decision |
| HRA-FR-027 | Treat stale, failed, unavailable, or required-but-missing analysis as incomplete unless a stricter rule blocks it |
| HRA-FR-028 | Ensure `isolated_execution_eligible` names the allowed quarantine profile and never grants host execution |
| HRA-FR-029 | Keep exception authority outside repository content, analyzers, models, and the Context core |

## Finding And Decision Language

Finding classification is one of:

- `observed`: directly recoverable from exact repository evidence;
- `derived`: produced by a named deterministic analyzer or rule;
- `reputation_match`: exact hash matched a named external dataset;
- `heuristically_suspicious`: a bounded heuristic matched but intent is unknown;
- `unknown`: required evidence or analysis is unavailable.

The product must never expose `clean`, `safe`, `trusted`, `approved`, or
`malware_free` as an assessment or admission state.

## Privacy And External Intelligence

Step 1 performs no network lookup. It defines the later Privacy B contract:

- SHA-256 is the only permitted outbound artifact identifier initially.
- File bytes, filenames, paths, repository identity, owner identity, and commit
  metadata are prohibited from the lookup payload.
- Each provider is separately enabled and disclosed.
- File upload is outside all three approved planning steps.
- A hash query is treated as lower disclosure, not zero disclosure.
- Provider absence, timeout, rate limit, or no match returns unknown reputation.

MISP is an optional later adapter. It is not an initial dependency. The initial
plan uses local ClamAV/YARA data plus optional direct hash providers.

## Success Metrics

| Metric | Initial target |
| --- | --- |
| Source mutation during admission inventory | Zero |
| Repository process, network, or analyzer execution by Context core | Zero |
| Exact evidence recovery for observed findings | 100% on conformance fixtures |
| Required-analysis coverage states represented | 100% |
| Stale/cross-snapshot analyzer results accepted | Zero |
| Malicious repository instructions that change policy or authority | Zero |
| Windows artifact recognition fixture pass | 100% of admitted fixture matrix |
| False `safe`/`clean` claims | Zero |
| Deterministic decision repeatability | Byte-stable for identical inputs and versions |

## Nonfunctional Requirements

### Security

- Preserve all current threat-model invariants.
- Parse new formats in the core only after a separate parser-admission review.
- Bound every read, classification rule, result, and collection.
- Escape control characters and preserve repository text as data.
- Keep source, queries, and findings out of default logs.

### Portability

- Run the inventory and decision conformance suites on all ADR-0003 Tier A
  platforms.
- Support Windows-oriented artifact recognition on macOS and Linux.
- Do not equate host platform with repository target platform.

### Explainability

- Every decision reason uses a stable rule identifier.
- Human-readable summaries are derived from canonical structured records.
- Missing coverage is displayed at the same prominence as detections.

## Implementation Plan And Gates

Implementation remains separately approval-gated. If authorized later, proceed
in the following order.

### HRA-0 — Freeze contracts and fixtures

- Define finding, inventory, coverage, assessment, policy, and decision schemas.
- Build only synthetic benign, suspicious, malformed, and hostile fixtures.
- Add Windows artifact classification fixtures without executing them.

Gate: schema review, threat-model delta, provenance review, and founder approval.

### HRA-1 — Inventory only

- Add bounded security-relevant artifact classification.
- Emit inventory and exclusions without findings or decisions.
- Prove source immutability and no network/process creation.

Gate: Tier A filesystem, resource, mutation, and adversarial tests.

### HRA-2 — Execution-surface observations

- Add narrow manifest/configuration rules one ecosystem at a time.
- Start with formats already safely readable by the core.
- Keep unsupported syntax explicit.

Gate: reviewed rule corpus, false-positive analysis, and exact-evidence recovery.

### HRA-3 — Coverage and assessment

- Add required-analysis planning and immutable assessment generation.
- Accept only synthetic external analyzer envelopes through ADR-0013's existing
  non-executing boundary.

Gate: stale, spoofed, partial, excessive, and cross-snapshot result tests.

### HRA-4 — Reference admission evaluator

- Add a pure policy evaluator with no filesystem, process, network, or model
  capability.
- Keep it logically outside the evidence core.

Gate: policy truth tables, monotonic-denial properties, exception denial, and
independent security review.

### HRA-5 — Step 1 release readiness

- Run evaluation, documentation, compatibility, and clean-install matrices.
- Publish limitations stating that analyzers and quarantine execution are absent.

Gate: explicit founder approval before Step 2 implementation begins.

## Acceptance Criteria

| ID | Given | When | Then |
| --- | --- | --- | --- |
| HRA-AC-001 | A repository containing prompt injection | Admission inventory runs | Content cannot change tools, policy, or authority |
| HRA-AC-002 | PE, DLL, MSI, PowerShell, batch, registry, service, and Windows build fixtures | Inventory runs on macOS and Linux | Every artifact is classified or explicitly unknown without execution |
| HRA-AC-003 | An install hook and privileged Compose file | Execution-surface inspection runs | Exact observed findings and evidence are returned |
| HRA-AC-004 | An oversized or encrypted artifact | Assessment is built | Coverage is partial/incomplete and no clean claim appears |
| HRA-AC-005 | A stale analyzer envelope | Normalization is attempted | It is rejected or quarantined and cannot influence admission |
| HRA-AC-006 | Identical assessment and policy | Evaluation repeats | The same canonical decision and reasons are produced |
| HRA-AC-007 | Full required coverage with no blocking findings | Policy evaluates | At most `isolated_execution_eligible` is returned |
| HRA-AC-008 | Any assessment | A user requests ordinary-host authorization | The capability is unavailable |

## Risks And Controls

| Risk | Control |
| --- | --- |
| Users interpret no findings as safety | Prohibited language and prominent coverage/unknowns |
| Static patterns over-report legitimate tooling | Observed/heuristic separation and exact evidence |
| New parsers enlarge the core attack surface | Minimal core classification; deep parsing in isolated workers |
| Windows artifacts are misread on non-Windows hosts | Byte-oriented identification and platform-independent fixtures |
| Repository text manipulates an AI consumer | Typed data/control separation and trust labels |
| Policy grows into business-risk authority | Closed stage states and external human exception owner |
| Later analyzers silently weaken core guarantees | Separate runner ADR, protocol, and release gate |

## Rollback And Disable Strategy

- The admission capability is separately feature-gated.
- Disabling it leaves all existing Context retrieval and packet behavior intact.
- New caches and assessments are derived and replaceable.
- A schema incompatibility fails closed and does not migrate silently.
- Rollback never changes a repository or converts an old assessment into a
  current one.

## Approval Boundary

The founder authorized HRA-0 contract freezing and, after reviewing that
evidence, separately authorized HRA-1 on 2026-08-29. The standing roadmap
directive and confirmed HRA-2 boundary permit narrow execution-surface
observations without a repeated approval ceremony. The initial HRA-2 corpus is
limited to exact npm lifecycle keys under a strict top-level `scripts` object.
Analyzers, network access, uploads, deep hostile-format parsing, repository
execution, policy decisions, and quarantine execution remain outside this
increment.
