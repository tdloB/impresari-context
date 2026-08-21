# Impresari Context — Verifiable Local Context MVP PRD

## Document Control

- Product: Impresari Context.
- PRD ID/version: IC-MVP-001 / 0.1.
- Status: Approved as the MVP implementation baseline.
- Release slice: Slice A — Verifiable local context.
- Date: 2026-08-20.
- Parent: [Master Product PRD](master-prd.md).
- Security authority: [Security Threat Model](../security/threat-model.md).
- Evaluation authority: [Evaluation PRD](evaluation-prd.md).

## Goal

Deliver a local-only, read-only engine that opens one explicitly authorized
software workspace, creates a deterministic snapshot, retrieves exact source
within bounded requests, packages verifiable context, and detects when evidence
is stale.

The MVP must prove the evidence and safety foundation before structural graphs,
model-assisted retrieval, durable memory, extensions, MCP, hosted services, or
AI App Builder OS integration are added.

## Problem Statement

An AI client can read files and run text search, but it usually lacks a shared
contract for:

- proving which workspace state was read;
- restricting reads to a canonical approved root;
- attaching exact current source to every delivered fact;
- staying within a declared context budget;
- distinguishing evidence, assumptions, conflicts, and unknowns;
- re-expanding compressed context;
- identifying stale packets after repository changes; and
- transferring context without silently granting new capabilities.

The MVP addresses those problems without claiming code understanding beyond
exact paths, metadata, and lexical evidence.

## Primary Users

1. A developer operating the engine against a local repository.
2. A CLI user inspecting or exporting context.
3. A programmatic local client integrating the core library.
4. A reviewer validating evidence and packet freshness.

The AI App Builder OS is not an MVP dependency or acceptance client.

## MVP Outcomes

| Outcome | Release evidence |
| --- | --- |
| Authorized local reads only | Path-boundary and isolation suite passes with zero disclosures |
| Reproducible workspace state | Snapshot identity is stable for identical eligible content and rules |
| Useful bounded retrieval | Evaluation recall and context-reduction gates pass |
| Verifiable packets | Every exact fact has a resolvable evidence reference |
| Visible staleness | Controlled changes invalidate affected snapshots and references |
| Client-neutral core | CLI and one in-process reference client pass the same conformance suite |
| No hidden external behavior | Network, process execution, telemetry, and source writes are absent or mechanically denied |

## Scope

### Included

- one local workspace per operation/session;
- explicit canonical workspace-root authorization;
- eligible-file discovery with recorded ignore and limit rules;
- detection and policy handling for files, directories, symlinks, hard-link
  ambiguity where detectable, special files, submodules, nested repositories,
  and unsupported/binary content;
- deterministic workspace snapshot and freshness status;
- exact file metadata and content hashing;
- exact path lookup;
- bounded filename, path, literal, and lexical/pattern search;
- deterministic candidate ranking with documented tie-breaking;
- exact evidence references to content and spans;
- bounded immutable context packets;
- evidence expansion;
- packet integrity and freshness validation;
- explicit conflicts, assumptions, unknowns, truncations, and redactions;
- metadata-first local audit events;
- human-readable CLI output and versioned structured output;
- an in-process programmatic interface using the same capability semantics.

### Excluded

- AST, symbol, call, dependency, or impact graphs;
- embeddings, vector search, semantic retrieval, reranking models, or LLM calls;
- summaries whose meaning is produced by a model;
- durable knowledge or cross-session factual memory;
- source edits, patches, commits, or command execution;
- network access, remote workspaces, hosted services, or telemetry upload;
- extension loading;
- MCP, HTTP, IDE, or AI App Builder OS adapters;
- multi-workspace queries;
- archives or generated artifacts that require execution to inspect;
- automatic repository repair, dependency installation, or self-update.

## Capability Surface

The MVP implements these protocol-independent operations:

| Capability | Required behavior |
| --- | --- |
| `workspace.open` | Canonicalize, authorize, inspect, and return a workspace handle without broadening the requested root |
| `snapshot.status` | Return snapshot identity, source/revision metadata when available, rules/version fingerprints, and freshness |
| `index.build` | Build or refresh the MVP metadata/lexical index; this is not the later structural graph |
| `code.search` | Perform bounded exact path, filename, literal, lexical, and permitted pattern search |
| `code.describe` | Return exact file metadata and bounded source evidence; symbol description is explicitly unsupported |
| `context.build` | Produce an immutable evidence packet for a query and purpose within a declared budget |
| `evidence.expand` | Resolve an evidence reference to authorized exact content and span |
| `context.validate` | Validate packet schema, integrity, snapshot match, evidence resolution, and freshness |
| `handoff.export` | Export the packet in a scoped local serialization without adding authority |

Unsupported operations must return a versioned `unsupported_capability` error,
not a fabricated or partially compatible response.

## Workspace Lifecycle States

| State | Meaning | Permitted transition |
| --- | --- | --- |
| `unopened` | No workspace authority exists | `workspace.open` request |
| `authorizing` | Canonical path and policy are being evaluated | `ready` or `denied` |
| `ready` | Root authorized; snapshot may be built or queried | `indexing`, `stale`, `closed` |
| `indexing` | Eligible artifacts are being fingerprinted/indexed | `current`, `partial`, `failed` |
| `current` | Snapshot and index match observed eligible state | query, validate, refresh, close |
| `partial` | Declared eligible artifacts were skipped or limited | query with warnings, refresh, close |
| `stale` | Current state no longer matches the snapshot | rebuild or close |
| `denied` | Authorization failed with no workspace handle | new explicit request only |
| `failed` | Operation failed without broadening access | retry within same authority or close |
| `closed` | Handles invalidated for the local session | new explicit open only |

`current` is scoped to the engine's observed eligible content and recorded rules.
It does not mean the filesystem cannot change immediately after the check.

## Critical User Journeys

### Journey A — Open safely

**Entry:** The user supplies a local path and optional expected Git revision.

**Happy path:**

1. Reject empty, implicit current-directory, home-directory shorthand, and
   overbroad roots unless explicitly permitted by policy.
2. Resolve the canonical root and compare it with the exact approved scope.
3. Inspect filesystem type and deny unsupported special files.
4. Record root identity without exposing unrelated ancestor information.
5. Return an opaque workspace handle and policy-decision identifier.

**Failure:** Return a stable reason code such as `path_not_found`,
`root_not_allowed`, `symlink_escape`, `unsupported_filesystem_object`, or
`policy_denied`. Do not fall back to a broader directory.

### Journey B — Build a snapshot

**Entry:** An authorized workspace is ready.

**Happy path:**

1. Discover eligible artifacts using deterministic ignore and limit rules.
2. Record every skip category and whether it makes the result partial.
3. Hash eligible contents and relevant metadata using recorded canonicalization.
4. Include engine, discovery-rule, hash-contract, and index-format versions.
5. Create a snapshot and lexical-index identity.
6. Write cache only beneath the configured isolated cache root.

**Failure:** Preserve the last valid snapshot as stale or unavailable; never
label a partial new index as the previous current snapshot.

### Journey C — Search within a budget

**Entry:** The caller provides workspace snapshot, query type, query, limits,
purpose, and output budget.

**Happy path:**

1. Validate query syntax and enforce maximum complexity.
2. Search only eligible artifacts in the authorized snapshot.
3. Rank deterministically and apply stable tie-breaking.
4. Return bounded result summaries with exact evidence references.
5. Record truncation and unknowns when matches exceed limits.

**Failure:** A timeout, resource limit, invalid pattern, or stale index returns a
structured state with no unbounded retry.

### Journey D — Build a context packet

**Entry:** The caller supplies a task query, declared purpose, and budget.

**Happy path:**

1. Collect candidates using the MVP retrieval strategies.
2. Normalize exact evidence and any deterministic metadata-derived facts.
3. Allocate budget across packet metadata, evidence excerpts, and recovery
   index using a recorded algorithm version.
4. Emit the packet with observed facts, assumptions, conflicts, unknowns,
   redactions, truncations, and recovery handles.
5. Seal the packet identity.

No model-generated claim or summary is permitted in the MVP packet builder.

### Journey E — Expand and validate

**Entry:** The caller supplies a packet or evidence handle plus authorized
workspace context.

**Happy path:**

1. Verify schema and integrity.
2. Verify workspace and snapshot identities.
3. Recheck the referenced artifact and content identity.
4. Return the exact authorized span or current/stale/unavailable status.

Content from a different snapshot must never be returned as if it satisfied the
old reference.

### Journey F — Export locally

**Entry:** The caller supplies a valid packet and local export destination
inside an explicitly allowed export root.

**Happy path:** Serialize the scoped packet, preserve sensitivity/redaction
metadata, and record an export audit event. Export does not grant the recipient
workspace access or durable-memory authority.

## Functional Requirements

### Authorization and discovery

| ID | Requirement | Priority |
| --- | --- | --- |
| MVP-FR-001 | Require an explicit workspace-root argument; no implicit broad-root default | Must |
| MVP-FR-002 | Canonicalize and authorize the root before enumerating or reading child content | Must |
| MVP-FR-003 | Prevent path traversal and symlink escape for discovery, reads, cache, and export | Must |
| MVP-FR-004 | Keep source access read-only and prove no source mutation in tests | Must |
| MVP-FR-005 | Apply deterministic ignore, hidden-file, binary, size, count, depth, and special-file rules | Must |
| MVP-FR-006 | Report all skip/partial categories without disclosing unauthorized names | Must |
| MVP-FR-007 | Revalidate authorization at content-read time to reduce path-swap/TOCTOU exposure | Must |

### Snapshot and cache

| ID | Requirement | Priority |
| --- | --- | --- |
| MVP-FR-008 | Bind snapshot identity to eligible contents, discovery-policy fingerprint, engine version, and hash-contract version | Must |
| MVP-FR-009 | Report repository revision and working-tree state when available without requiring Git | Should |
| MVP-FR-010 | Isolate cache by canonical workspace identity and snapshot; cache is replaceable | Must |
| MVP-FR-011 | Detect tampered, corrupt, incompatible, or partial cache and rebuild or fail visibly | Must |
| MVP-FR-012 | Never use cache as the sole evidence source when matching source cannot be verified | Must |
| MVP-FR-013 | Provide deterministic cache purge for one exact workspace identity without touching source | Should |

### Retrieval

| ID | Requirement | Priority |
| --- | --- | --- |
| MVP-FR-014 | Support exact file/path lookup and bounded filename search | Must |
| MVP-FR-015 | Support bounded literal and lexical search over eligible text artifacts | Must |
| MVP-FR-016 | If pattern search is supported, use a bounded engine/strategy resistant to catastrophic backtracking | Must |
| MVP-FR-017 | Use deterministic ranking and tie-breaking documented by algorithm version | Must |
| MVP-FR-018 | Enforce per-request time, file, byte, match, output, and memory budgets | Must |
| MVP-FR-019 | Return truncation, timeout, unsupported encoding, and skipped-file states explicitly | Must |
| MVP-FR-020 | Do not claim symbol, semantic, dependency, call, or impact understanding | Must |

### Evidence and packets

| ID | Requirement | Priority |
| --- | --- | --- |
| MVP-FR-021 | Every evidence item contains workspace snapshot, artifact path, content hash, span, evidence kind, extraction method/version, confidence, and trust classification | Must |
| MVP-FR-022 | Spans use one documented encoding/line-column contract and handle line-ending/Unicode cases deterministically | Must |
| MVP-FR-023 | Context packets conform to a versioned schema and contain the fields defined by the architecture | Must |
| MVP-FR-024 | Packet building never exceeds the declared output budget; metadata overhead is accounted for | Must |
| MVP-FR-025 | Packet identity changes if evidence, policy decision, budget outcome, algorithm, or snapshot materially changes | Must |
| MVP-FR-026 | Evidence expansion rechecks authorization, snapshot, content hash, and span before returning source | Must |
| MVP-FR-027 | Packet validation distinguishes valid-current, valid-stale, corrupt, incompatible, denied, and partially unavailable | Must |
| MVP-FR-028 | Local handoff export preserves packet identity and cannot add omitted evidence or authority | Must |

### Policy, audit, and interfaces

| ID | Requirement | Priority |
| --- | --- | --- |
| MVP-FR-029 | Route every capability through one policy gateway | Must |
| MVP-FR-030 | Treat purpose, caller, and role as data validated by policy, never prompt-level authority | Must |
| MVP-FR-031 | Emit metadata-first audit events with request, decision, workspace, snapshot, capability, outcome, limits, versions, and timing | Must |
| MVP-FR-032 | Exclude source content, queries containing source, secrets, and raw environment values from logs by default | Must |
| MVP-FR-033 | Provide versioned machine-readable output and stable error envelopes | Must |
| MVP-FR-034 | Provide human-readable CLI output without becoming a second semantic contract | Must |
| MVP-FR-035 | CLI and in-process interface invoke the same core capability handlers | Must |
| MVP-FR-036 | Perform no network requests, telemetry upload, arbitrary process execution, or self-update | Must |

## Proposed Interface Shape

Exact command names are provisional until the stack decision, but the CLI must
support an equivalent lifecycle:

```text
context workspace open <explicit-path>
context snapshot build --workspace <handle>
context snapshot status --workspace <handle>
context search --workspace <handle> --query <query> --budget <budget>
context build --workspace <handle> --purpose <purpose> --query <query> --budget <budget>
context evidence expand --workspace <handle> --evidence <id>
context packet validate --workspace <handle> --packet <path-or-id>
context handoff export --packet <id> --destination <explicit-path>
```

Machine-readable output goes to standard output. Human diagnostics and progress
go to standard error. Source excerpts are never printed as incidental debug
output.

## Data Contracts

The MVP must publish versioned schemas for:

- workspace handle metadata;
- snapshot status;
- discovery/skipped-artifact summary;
- search request and result;
- evidence reference;
- context packet;
- packet validation result;
- policy decision;
- audit event;
- error envelope.

The architecture examples define semantics, not the final serialization. A
separate ADR must select serialization and canonical hashing rules before code.

## Error Model

Every error includes:

- stable code and schema version;
- safe human message;
- retryability;
- operation and request identifier;
- workspace/snapshot identifier only when authorized;
- partial-result status;
- recovery action when one is safe;
- underlying detail only in an explicitly enabled local diagnostic channel.

Required error families include authorization, invalid input, budget exceeded,
stale state, corrupt/incompatible cache, unsupported capability, unsupported
artifact, integrity failure, partial result, internal failure, and unavailable
evidence.

## Privacy And Retention

- The MVP has no outbound data path.
- Product telemetry is absent, not merely disabled in configuration.
- Local audit retention is configurable and metadata-first.
- Cache and exports may contain sensitive source-derived material and must be
  permission-restricted and documented accordingly.
- Source-derived cache can be deleted without damaging the repository.
- A purge targets one resolved workspace/cache identity; broad recursive paths
  are prohibited.
- No durable knowledge store exists in the MVP.

## Nonfunctional Requirements

### Correctness

- Identical eligible inputs, rules, and versions yield identical normalized
  snapshot and deterministic query outputs.
- Line, column, byte, Unicode, newline, and encoding behavior is specified and
  covered by conformance fixtures.
- Partial discovery can never be presented as complete.

### Security

- All `MVP-BLOCK` requirements in the threat model must pass.
- No security control may depend solely on prompt instructions.
- Unsafe or uncertain authorization fails closed.

### Performance

- Operations are interruptible at safe boundaries and respect declared limits.
- Benchmarks report corpus size, eligible bytes/files, hardware, cold/warm
  cache, versions, and configuration.
- Absolute targets are set only after the evaluation baseline is captured;
  release cannot hide regression behind different hardware.

### Portability

- Clean installation must work on the approved OS/architecture matrix.
- No global shell, editor, Git, or model-provider configuration is changed.
- The engine can run inside an isolated local environment with network denied.

## Acceptance Criteria

| ID | Given | When | Then | Evidence |
| --- | --- | --- | --- | --- |
| MVP-AC-001 | An explicit permitted root | It is opened | The returned canonical scope is no broader than requested | Unit/integration test |
| MVP-AC-002 | Traversal, symlink swap, ancestor path, or special-file fixtures | A read is attempted | No unauthorized bytes or names are returned | Adversarial suite |
| MVP-AC-003 | An unchanged corpus and rules | A snapshot is rebuilt | Normalized snapshot identity is identical | Reproducibility suite |
| MVP-AC-004 | A controlled eligible-file change | Status or evidence is checked | The prior snapshot/reference is stale | Mutation suite |
| MVP-AC-005 | A literal query with known matches | Search runs within budget | Expected matches and exact spans are returned deterministically | Retrieval fixture |
| MVP-AC-006 | More matches than permitted | Search runs | Results are bounded and truncation is explicit | Limit test |
| MVP-AC-007 | Malicious repository instructions and secret-like strings | Context is built | They remain untrusted evidence and policy-required redaction applies | Security test |
| MVP-AC-008 | A packet at a requested budget | It is serialized | Delivered units do not exceed the budget under the declared accounting method | Contract test |
| MVP-AC-009 | Every exact fact in a current packet | Evidence is expanded | Exact matching content and span resolve | Conformance test |
| MVP-AC-010 | A stale or cross-workspace handle | Expansion is requested | It fails visibly without substituting content | Isolation test |
| MVP-AC-011 | Source tree before and after all MVP tests | Operations complete | No source file, metadata intentionally in scope, Git state, or configuration was modified | Filesystem diff |
| MVP-AC-012 | Network-denied execution | Full conformance suite runs | All MVP functions work and no connection is attempted | Sandbox/network test |
| MVP-AC-013 | CLI and programmatic client | Same canonical request is executed | Normalized semantic response is equivalent | Cross-interface test |
| MVP-AC-014 | Clean machines on supported matrix | Package is installed and tests run | Installation succeeds without global mutation | Release rehearsal |
| MVP-AC-015 | Evaluation corpus and fixed budget | MVP is compared with baseline | Quality, context reduction, and security gates in IC-EVAL-001 pass | Evaluation report |

## Definition Of Done

The MVP is complete only when:

1. Every Must requirement has tests and traceability.
2. The threat model's MVP-blocking controls pass.
3. The Evaluation PRD release gates pass on the frozen benchmark manifest.
4. All public schemas are versioned and documented.
5. The CLI and programmatic reference client pass conformance.
6. Clean-install and network-denied rehearsals pass on the approved platform
   matrix.
7. Source workspaces remain unmodified across the full suite.
8. Dependency, license, provenance, and security-reporting artifacts are
   complete.
9. Limitations and unsupported capabilities are documented prominently.
10. Founder/maintainer approval records the release decision.

## Delivery Plan

1. Approve product, MVP, security, and evaluation documents.
2. Record stack, platform, storage, hash, serialization, and budget ADRs.
3. Define schemas and conformance fixtures before core implementation.
4. Implement path authorization and read-only workspace controller.
5. Implement deterministic discovery, hashing, snapshot, and cache isolation.
6. Implement bounded lexical retrieval and exact spans.
7. Implement evidence, packet, expansion, and validation contracts.
8. Implement policy, errors, audit, CLI, and reference library client.
9. Run security and evaluation suites; remediate without relaxing gates.
10. Rehearse clean installation and publish only after governance approval.

## Risks And Open Questions

| Question | Why it blocks implementation |
| --- | --- |
| Primary language/runtime | Determines memory-safety, distribution, embedding, concurrency, and dependency posture |
| Supported operating systems | Defines filesystem semantics and conformance matrix |
| Hash and path canonicalization | Determines snapshot/evidence identity and cross-platform behavior |
| Ignore/discovery rules | Determines what “complete” means |
| Pattern engine | Determines denial-of-service exposure |
| Cache format | Determines integrity, isolation, migration, and purge behavior |
| Budget unit | Determines packet contract and cross-model portability |
| Secret/sensitivity rules | Determines redaction behavior without overclaiming secret detection |
| Git behavior | Determines optional revision metadata and submodule/nested-repository treatment |
| License and steward | Required before the first public source release |

## Approval And Change Control

- Approved by/date: Founder, 2026-08-20.
- Structural graph, model retrieval, durable memory, extension execution, MCP,
  hosted service, multi-workspace access, source mutation, and OS integration
  are out of scope and require their own approved slice or architecture record.
