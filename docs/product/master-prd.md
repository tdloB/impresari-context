# Impresari Context — Master Product Requirements Document

## Document Control

- Product: Impresari Context.
- Naming status: Public project name confirmed by the founder on 2026-08-22.
- PRD ID/version: IC-MPRD-001 / 0.1.
- Status: Approved as the product baseline.
- Product owner: Aaron Boldt; legal project steward: BoldtHaus Studio, LLC.
- Date: 2026-08-20.
- Related records:
  - [Architecture](../architecture.md)
  - [System boundaries](../boundaries.md)
  - [Influences and provenance](../influences-and-provenance.md)
  - [ADR-0001](../decisions/0001-independent-core-and-thin-adapters.md)
  - [Verifiable Local Context MVP PRD](verifiable-local-context-mvp-prd.md)
  - [Security Threat Model](../security/threat-model.md)
  - [Evaluation PRD](evaluation-prd.md)
  - [Phase 0: Language and Client Foundation PRD](phase-0-language-and-client-foundation-prd.md)
  - [Revised Product Roadmap](revised-product-roadmap.md)

## Product Name

**Impresari Context** is an independent, open-source context and evidence engine
for AI-assisted software development.

## Goal

Make repository context compact enough for an AI client to use, precise enough
for a human to verify, and constrained enough to operate safely in an untrusted
workspace.

The product converts a permitted software workspace into versioned structural
facts, exact evidence references, and bounded context packets. It does not
become the user's agent orchestrator, development environment, or approval
authority.

## Approved roadmap

This Master PRD defines the enduring product baseline. The founder-approved
[Revised Product Roadmap](revised-product-roadmap.md) defines the complete
implementation sequence and phase scope:

1. Correct the public language/client contract and doctor surface.
2. Add Python, configuration evidence, and first-class Codex, Claude Code, and
   Cursor integrations.
3. Add Rust and Go plus Gemini CLI, GitHub Copilot CLI, and VS Code Copilot
   integrations.
4. Add the deterministic context planner.
5. Add Java, Kotlin, C#, impact evidence, and explicit incremental updates.
6. Add demand-led language support, beginning with Swift, PHP, Ruby, C/C++,
   Scala, Dart, and carefully constrained SQL.

The roadmap document is authoritative for the detailed requirements, status,
dependencies, and admission criteria of each phase. Future work must not
renumber or reinterpret those phases without a superseding roadmap decision.

## Background And Problem

### Current situation

AI coding tools commonly receive repository context through ad hoc file reads,
text search, large repository dumps, opaque embedding retrieval, or summaries
that cannot be traced back to current source. Different clients rebuild similar
context independently, resulting in duplicated indexing, inconsistent views of
the repository, avoidable token cost, and stale conclusions.

### User problem

Developers and AI clients need to answer questions such as:

- Which files and symbols are relevant to this task?
- What exact source supports a conclusion?
- Has the source changed since the context was produced?
- What is observed, what is derived, and what remains unknown?
- Can useful context be transferred between authorized clients without sending
  the entire workspace or silently creating durable memory?

### Product opportunity

An independent local-first engine can provide one evidence model, one canonical
workspace snapshot, and one bounded capability vocabulary across CLI, SDK, MCP,
and consumer-specific adapters. This makes retrieval and context packaging
measurable and replaceable without making a model provider or agent operating
system mandatory.

### Architectural influences

The design is informed by publicly demonstrated capabilities in LeanCTX and
Graft. It adopts general lessons about compressed recoverable context, context
budgets, sessions and handoffs, deterministic code structure, repository maps,
call tracing, and freshness. It will be an original implementation and will not
copy or mechanically translate upstream source, documentation, tests, prompts,
or assets.

## Product Principles

1. Evidence before summary.
2. Deterministic structure before model-generated interpretation.
3. Exact source remains the highest-authority evidence tier.
4. Every consequential derived claim must identify recoverable evidence.
5. Repository content and extension output are untrusted data.
6. The core is read-only with respect to source workspaces.
7. Network, execution, persistence, and durable-memory promotion are separate
   denied-by-default capabilities.
8. MCP is a transport, not the internal architecture.
9. Consumers own orchestration, approvals, and business policy.
10. One stable capability surface is preferable to overlapping context stacks.

## Users

| User | Primary need | Initial priority |
| --- | --- | --- |
| Individual developer | Give an AI client relevant, verifiable local context without exposing the entire repository | Primary |
| AI coding client or agent framework | Retrieve bounded context through a stable protocol-independent contract | Primary |
| Maintainer or reviewer | Reproduce evidence and detect stale or unsupported conclusions | Primary |
| Security reviewer | Trace findings to exact source while preserving strict workspace boundaries | Secondary |
| Tool integrator | Build a CLI, SDK, MCP, IDE, or workflow adapter without adopting a private operating system | Secondary |
| Organization administrator | Apply workspace, sensitivity, retention, and extension policies | Later |

End customers are not required to use the AI App Builder OS. The OS is the
first reference consumer, not a runtime dependency.

## Jobs To Be Done

1. When an AI client receives a software task, provide the smallest useful set
   of current repository evidence within a declared budget.
2. When a conclusion matters, recover the exact source and verify that it still
   belongs to the same workspace snapshot.
3. When repository state changes, identify stale packets and rebuild rather
   than silently presenting old evidence as current.
4. When multiple clients collaborate, transfer an immutable, scoped packet
   without transferring orchestration authority or automatically creating
   durable memory.
5. When richer analysis is installed, constrain it through explicit capability
   and provenance contracts.

## Outcomes And Success Metrics

Targets are release gates for the relevant slice. The Evaluation PRD defines
measurement details and permits recalibration only through recorded review.

| Outcome | Metric or evidence | Baseline | Initial target | Horizon |
| --- | --- | --- | --- | --- |
| Verifiable context | Valid exact-evidence recovery rate | No shared mechanism | 100% for current references in the conformance suite | MVP |
| Visible staleness | Stale snapshot/reference detection | Ad hoc | 100% of controlled mutations detected | MVP |
| Safe workspace isolation | Unauthorized path and cross-workspace disclosure | Tool-dependent | Zero successful disclosures in the adversarial suite | Every release |
| Useful bounded retrieval | Task evidence recall at fixed budget | Native-read baseline | At least 0.90 and not materially below the declared baseline | MVP and later |
| Lower context cost | Median delivered context units at matched recall | Native-read baseline | At least 30% reduction in the initial benchmark; target subject to baseline review | MVP |
| Deterministic behavior | Repeatability for deterministic operations | Tool-dependent | Byte-stable normalized results under identical inputs and versions | MVP |
| Client neutrality | Independent reference client conformance | None | CLI plus one non-OS programmatic client pass the contract suite | Before 1.0 |
| Structural usefulness | Supported graph facts carry exact spans and confidence | None | 100% of emitted facts meet graph provenance contract | Structural slice |

## Scope And Release Boundaries

### Slice A — Verifiable local context

- one authorized local workspace;
- read-only source access;
- deterministic snapshot and freshness state;
- path and lexical retrieval;
- exact evidence references and expansion;
- bounded immutable context packets;
- packet integrity and freshness validation;
- local metadata-first audit records;
- CLI and programmatic library interface.

### Slice B — Structural intelligence

- parser adapter contract;
- files, packages, symbols, and containment;
- imports, exports, references, and calls where supported;
- repository maps and bounded trace queries;
- confirmed versus heuristic relationships;
- source spans, resolver versions, and confidence;
- incremental structural index updates.

### Slice C — Context lifecycle and reference integration

- richer context planning and compression;
- session-scoped packet references;
- immutable handoff export;
- AI App Builder OS reference adapter;
- a second non-OS reference consumer;
- governed native-read fallback when the engine is unavailable.

### Slice D — Controlled extensibility

- versioned parser, retriever, analyzer, exporter, and transport contracts;
- integrity-pinned extension manifests;
- denied-by-default filesystem, process, environment, model, and network access;
- output normalization and quarantine;
- MCP transport and additional adapters where justified.

### Later, separately gated scope

- durable knowledge after explicit promotion;
- optional semantic or model-assisted candidate retrieval;
- remote or hosted service modes;
- multi-tenant deployment;
- IDE integrations;
- organization administration and policy distribution.

### Explicit non-goals

The product will not initially:

- orchestrate agents or choose which agent acts next;
- edit, refactor, commit, push, publish, or deploy source code;
- execute arbitrary repository commands;
- require an LLM or outbound network for canonical indexing;
- proxy or rewrite model-provider traffic;
- install global shell hooks or silently modify editor configuration;
- provide a second graph alongside another canonical graph;
- treat summaries as verified facts;
- promote session material into durable memory without approval;
- own release, legal, security-risk, or business go/no-go decisions;
- embed private AI App Builder OS prompts, agents, phases, or policy.

## Critical User Journeys

### Journey 1 — Open and snapshot a workspace

1. The caller supplies an approved root and optional expected revision.
2. The engine resolves the canonical path before authorization.
3. Policy validates the root and read capability.
4. The engine discovers eligible artifacts using recorded ignore and file-type
   rules.
5. It creates a content-addressed snapshot identity and reports freshness.
6. Failure returns a structured error without partially authorizing a broader
   root.

### Journey 2 — Build bounded task context

1. The caller supplies a purpose, query, workspace snapshot, and budget.
2. Policy validates caller, workspace, sensitivity, and requested capability.
3. The retrieval planner selects exact and ranked candidate evidence.
4. The normalizer separates observed facts, derived claims, assumptions,
   conflicts, unknowns, and redactions.
5. The packager emits an immutable packet within budget.
6. The packet records recovery handles, versions, and policy decision.

### Journey 3 — Expand and verify evidence

1. The caller supplies an evidence or recovery handle.
2. The engine validates workspace and snapshot identity.
3. Exact source is returned only if the matching content remains authorized and
   resolvable.
4. A mismatch returns stale or unavailable status, never content from another
   revision or workspace.

### Journey 4 — Trace repository structure

1. The caller requests a supported relationship or impact path.
2. The engine verifies graph freshness and traversal budget.
3. Results identify confirmed and heuristic edges separately.
4. Each fact provides source evidence and resolver provenance.
5. Unsupported-language or unresolved edges remain explicit unknowns.

### Journey 5 — Export a handoff

1. An authorized caller selects a packet and narrower export scope.
2. Policy applies sensitivity, redaction, and size rules.
3. The engine exports an immutable packet and audit identifier.
4. The receiving consumer must separately authorize the workspace and packet.
5. The handoff conveys evidence, not agent identity, routing, approval, or
   durable-memory authority.

## User Stories

| ID | Actor | Need | Outcome | Priority |
| --- | --- | --- | --- | --- |
| IC-US-001 | Developer | Open only the repository I selected | Other local paths remain inaccessible | Must |
| IC-US-002 | AI client | Search source within a declared budget | I receive relevant candidates without a repository dump | Must |
| IC-US-003 | Reviewer | Expand a claim to exact source | I can independently verify it | Must |
| IC-US-004 | Developer | Know when a packet is stale | Old context is not mistaken for current evidence | Must |
| IC-US-005 | Integrator | Use stable capabilities through my preferred transport | Transport changes do not change semantics | Must |
| IC-US-006 | Security reviewer | Distinguish observed and derived material | Confidence is not confused with fact | Must |
| IC-US-007 | Agent framework | Traverse structural relationships | I can ask bounded repository questions | Should |
| IC-US-008 | Collaborator | Export a scoped handoff | Another authorized client can resume without rediscovery | Should |
| IC-US-009 | Administrator | Restrict extensions and sensitive data | Optional functionality cannot silently broaden access | Later |
| IC-US-010 | Maintainer | Reproduce benchmark and security results | Releases are evidence-backed | Must |

## Functional Requirements

| ID | Requirement | Priority | Release |
| --- | --- | --- | --- |
| IC-FR-001 | Resolve and authorize workspace paths before any source read | Must | A |
| IC-FR-002 | Create a deterministic snapshot identity covering source state, policy-relevant discovery rules, and engine/parser versions | Must | A |
| IC-FR-003 | Keep all core source-workspace access read-only | Must | A |
| IC-FR-004 | Support bounded exact path, file, pattern, and lexical search | Must | A |
| IC-FR-005 | Represent exact evidence with workspace, content hash, artifact, span, extraction, confidence, and trust fields | Must | A |
| IC-FR-006 | Produce immutable bounded context packets with facts, claims, assumptions, conflicts, unknowns, evidence index, recovery handles, and redactions | Must | A |
| IC-FR-007 | Expand evidence only against its matching authorized snapshot and fail visibly when stale or unavailable | Must | A |
| IC-FR-008 | Apply caller, workspace, capability, sensitivity, and budget policy through one gateway | Must | A |
| IC-FR-009 | Record metadata-first local audit events without source content or secrets by default | Must | A |
| IC-FR-010 | Return versioned structured errors with no sensitive path or content leakage | Must | A |
| IC-FR-011 | Build one canonical structural graph per workspace snapshot | Must | B |
| IC-FR-012 | Attach source spans, resolver identity, version, extraction method, and confidence to every graph fact | Must | B |
| IC-FR-013 | Support bounded containment, import, reference, call, dependency, and impact traversal where adapters provide evidence | Should | B |
| IC-FR-014 | Distinguish unsupported, unresolved, heuristic, and confirmed relationships | Must | B |
| IC-FR-015 | Export scoped immutable handoff packets without routing or approval authority | Should | C |
| IC-FR-016 | Provide a protocol-independent contract used by CLI, SDK, MCP, and consumer adapters | Must | C |
| IC-FR-017 | Require declared, pinned capabilities for every extension and normalize all extension output | Must | D |
| IC-FR-018 | Deny extension process, network, environment, model, and undeclared filesystem access by default | Must | D |
| IC-FR-019 | Allow knowledge proposals but prohibit automatic durable promotion | Later | Later |
| IC-FR-020 | Operate without a hosted model or outbound network for all canonical evidence and graph capabilities | Must | All |

## Roles, Permissions, And Human Approvals

The core receives opaque caller and role identifiers from a consumer. It owns
mechanical authorization but does not define organization roles.

| Action | Default | Approval owner |
| --- | --- | --- |
| Read authorized workspace | Denied until explicitly scoped | Local user or consumer policy |
| Write source workspace | Prohibited in initial product | Separate future architecture decision |
| Write engine cache | Allowed only to configured project-isolated cache | Local configuration |
| Export packet | Denied until scope and sensitivity policy pass | Consumer policy |
| Network or model access | Denied | Explicit destination/data policy and user decision |
| Process execution | Denied | Separate capability and threat-model update |
| Install/enable extension | Denied | User/administrator plus integrity verification |
| Promote durable knowledge | Denied | Consumer-defined human or governed approval |
| Accept legal/security/release risk | Never owned by core | Consumer and authorized human |

## Data, Integrations, And Sources Of Truth

### Sources of truth

1. Authorized exact workspace content at a named snapshot.
2. Deterministically extracted metadata and graph facts linked to exact content.
3. Derived claims linked to evidence and method.
4. Consumer assertions and model output, always labeled as asserted or derived.

### Primary data classes

- workspace identity and file metadata;
- source-derived content fragments and hashes;
- evidence references and structural facts;
- immutable context packets;
- policy decisions and local audit metadata;
- session references and, later, knowledge proposals.

### Initial integrations

- local filesystem;
- optional Git metadata through a non-mutating adapter;
- CLI and in-process library API.

MCP, AI App Builder OS, IDEs, hosted models, and external stores are later
adapters and are not MVP dependencies.

## AI And Automation Behavior

- Canonical snapshotting, lexical retrieval, evidence resolution, and graph
  extraction must not require an AI model.
- Optional semantic/model retrievers may propose candidates only after a later
  gate. Their output is derived, labeled, measurable, and replaceable.
- Repository text cannot issue instructions, alter permissions, or broaden
  capabilities.
- AI confidence never authorizes filesystem, network, process, persistence,
  publication, or durable-memory actions.
- A model-generated summary cannot become the sole support for a consequential
  claim.
- Model unavailability must not prevent exact evidence recovery.

## Nonfunctional Requirements

### Security and privacy

- Enforce the invariants and release gates in the Security Threat Model.
- Default to no network and no telemetry.
- Minimize source-derived persistence and isolate it by workspace identity.
- Never log source or detected secrets by default.

### Correctness

- Deterministic operations must be reproducible for identical inputs and
  versions.
- Snapshot, evidence, and packet identities must be content-addressed or bound
  to tamper-evident content identities.
- Unsupported analysis must be explicit rather than guessed.

### Performance

- All operations enforce time, output, memory, file-count, file-size, and
  traversal budgets.
- Performance targets are hardware-normalized and set through the Evaluation
  PRD after baseline measurement.

### Portability

- The supported operating-system and architecture matrix must be selected
  before implementation and tested in clean environments.
- Core contracts must not depend on an editor, agent framework, or shell hook.

### Accessibility and usability

- CLI output provides structured and human-readable modes.
- Errors identify recovery actions without disclosing unauthorized data.
- Documentation includes minimal setup, examples, security posture, and
  limitations.

### Maintainability

- Public schemas and capabilities are versioned.
- Parser, engine, transport, extension, and consumer adapter versions remain
  independently identifiable.
- Dependencies are pinned through reproducible lockfiles and license inventory.

## Acceptance Criteria

| ID | Given | When | Then | Evidence |
| --- | --- | --- | --- | --- |
| IC-AC-001 | An authorized repository | A client opens and snapshots it | A deterministic snapshot and freshness state are returned without source mutation | Conformance test |
| IC-AC-002 | An unauthorized, symlinked, or traversed path | A read is attempted | Access is denied before content disclosure | Adversarial test |
| IC-AC-003 | A bounded task query | Context is built | The immutable packet stays within budget and every exact fact has a recovery reference | Contract test |
| IC-AC-004 | A valid current evidence handle | Evidence is expanded | Exact matching source and span are returned | Test fixture |
| IC-AC-005 | A changed or mismatched workspace | Old evidence is expanded or validated | The engine returns stale/unavailable and never substitutes current content | Mutation test |
| IC-AC-006 | Malicious instructions in repository text | Retrieval runs | The content is returned only as untrusted evidence and cannot affect policy | Security test |
| IC-AC-007 | Identical inputs, policy, and versions | Deterministic operations repeat | Normalized results are byte-stable | Reproducibility test |
| IC-AC-008 | An unsupported language | Analysis is requested | Exact retrieval remains available and structural status is explicitly unsupported | Compatibility test |
| IC-AC-009 | A structural fact | It is emitted | Source span, extraction method, resolver version, and confidence are present | Schema test |
| IC-AC-010 | A public release candidate | Release gates run | Required functional, security, evaluation, provenance, license, and clean-install evidence passes | Release checklist |

## Rollout, Migration, And Rollback

1. Develop against synthetic and approved public repositories.
2. Release an experimental local-only CLI/library with no network, extensions,
   or compatibility promise beyond the declared schema version.
3. Validate against an unrelated reference client before the OS adapter becomes
   authoritative.
4. Integrate with the AI App Builder OS behind an explicit capability flag and
   governed native-read fallback.
5. Stabilize public contracts only after conformance and evaluation evidence.

Rollback consists of disabling the adapter or returning to the consumer's
native-read path. Engine caches are replaceable and must not become the only
copy of source or evidence.

## Risks, Assumptions, And Dependencies

| Risk or assumption | Effect | Control |
| --- | --- | --- |
| Broad language support dilutes correctness | Incomplete or misleading graph | Start with an explicit small language matrix |
| Context reduction harms recall | Smaller but unsafe packets | Measure recall at matched budgets and preserve recovery |
| Malicious repository content affects tools | Policy or data compromise | Structured gateway; data/control separation; adversarial tests |
| Cache becomes stale or crosses projects | Incorrect or confidential output | Content identities, isolation, validation, replaceability |
| Upstream inspiration creates provenance confusion | License and trust risk | Original implementation rules and contribution review |
| OS requirements leak into public core | Reduced usefulness and private coupling | Thin adapter and independent reference client |
| Premature extension system expands attack surface | Security and maintenance burden | Defer until core security/evaluation gates pass |
| Working name changes | Documentation/package migration | Keep naming isolated from protocol identifiers until public decision |

## Pre-Implementation Decision Gates

| Decision gate | Status / authority |
| --- | --- |
| Primary language/runtime and compiler policy | Resolved by [ADR-0002](../decisions/0002-rust-core-runtime.md) |
| Operating-system and CPU architecture matrix | Resolved by [ADR-0003](../decisions/0003-supported-platform-matrix.md) |
| Source-language and parser strategy | Resolved by [ADR-0004](../decisions/0004-source-language-and-parser-strategy.md) |
| Snapshot hashing, serialization, schemas, paths, and spans | Resolved by [ADR-0005](../decisions/0005-hashing-serialization-and-schema.md) and [ADR-0009](../decisions/0009-path-and-identity-encoding.md) |
| Cache/index persistence and migration policy | Resolved by [ADR-0006](../decisions/0006-local-cache-and-storage.md) |
| Context-budget unit and accounting | Resolved by [ADR-0007](../decisions/0007-context-budget-accounting.md) |
| License, contributions, attribution, and governance | Resolved with owner/counsel gate by [ADR-0008](../decisions/0008-license-contributions-and-governance.md) |
| CLI and programmatic packaging | Initial shape resolved by ADR-0002; exact crate/package names remain a scaffold task |
| Repository layout, CI trust, dependency policy, and release provenance | Policy resolved by ADR-0002/0008; exact workflow and signing mechanism remain bootstrap/release ADR tasks |
| Product, MVP, security, and evaluation baseline approval | Founder-approved on 2026-08-20 |

## Open Questions

- What absolute performance ceilings should be recorded after the first baseline
  run?
- Which policies belong in a portable declarative format versus consumer code?
- Which long-term security contact should supplement GitHub private
  vulnerability reporting if the project later needs an off-platform channel?

## Approval And Change Control

- Approved by/date: Founder, 2026-08-20.
- Consequential unresolved interpretations: implementation stack, license,
  owner/steward, language matrix, storage, and budget units.
- Material changes to product boundary, trust model, canonical evidence, source
  mutation, execution, network, hosted deployment, or durable memory require a
  new architecture decision plus security and evaluation review.
