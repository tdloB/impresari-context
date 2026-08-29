# Hostile Repository Admission Foundation — Architecture Requirements Document

## Document Control

- Product: Impresari Context.
- ARD ID/version: IC-HRA-ARD-001 / 0.1.
- Status: HRA-1 and HRA-2 implemented; initial HRA-3 coverage planning and assessment assembly implemented.
- Date: 2026-08-26.
- Sequence: Security expansion step 1 of 3.
- Related records:
  - [Hostile Repository Admission PRD](../product/hostile-repository-admission-prd.md)
  - [ADR-0073](../decisions/0073-evidence-only-hostile-repository-admission.md)
  - [ADR-0010](../decisions/0010-structural-worker-protocol-and-isolation.md)
  - [ADR-0013](../decisions/0013-extension-contracts-without-code-loading.md)
  - [Security Threat Model](../security/threat-model.md)

## Architecture Objective

Extend the current evidence compiler with a deterministic, non-executing
admission-preparation path that can inventory hostile-repository risk surfaces,
bind security observations to exact evidence, account for missing analysis, and
support a separate deterministic stage decision without granting new runtime
authority to the Context core.

## Governing Architecture Decisions

### AD-HRA-001 — Evidence and authority remain separate

The Context core produces inventory, findings, coverage, and immutable
assessments. A pure evaluator consumes an assessment and versioned policy. The
evaluator has no filesystem, network, process, model, credential, or approval
capability, and its most permissive result is eligibility for a named isolated
quarantine profile.

### AD-HRA-002 — No analyzer code enters the core

Step 1 reuses ADR-0013's manifest and hostile-output normalization concepts but
does not load or execute analyzer artifacts. Deep binary, installer, script AST,
malware, dependency, and reputation work remains outside the core.

### AD-HRA-003 — Coverage is canonical

The assessment records required analysis and its state independently from
findings. Zero findings cannot compensate for missing, failed, stale, or skipped
analysis.

### AD-HRA-004 — Platform target differs from host platform

Windows-oriented artifacts are inventoried on macOS and Linux. Host platform
does not determine whether an artifact can execute elsewhere. Deep hostile
format parsing is deferred to step 2 workers.

### AD-HRA-005 — Stage decisions are not safety verdicts

Canonical decisions use only four closed values. `isolated_execution_eligible`
is a bounded routing result and cannot be converted into general execution
authority by prompt, repository content, analyzer output, or transport metadata.

## System Context

```text
Authorized caller
      |
      v
Existing capability/policy gateway
      |
      v
Read-only snapshot controller -----> Untrusted repository
      |
      +----> Security artifact inventory
      +----> Execution-surface extractor
      +----> Analyzer requirement planner
      |
      v
Assessment assembler <----- normalized external results (future)
      |
      +----> immutable assessment/cache
      +----> existing metadata-first audit
      |
      v
Pure admission evaluator <----- versioned consumer policy
      |
      v
Stage decision (no execution token)
```

## Logical Components

| Component | Responsibility | New authority |
| --- | --- | --- |
| Admission request validator | Validate snapshot, purpose, policy profile, and budgets | None |
| Artifact classifier | Classify bounded regular-file bytes and metadata | None |
| Execution-surface extractor | Emit narrow observed facts from admitted text/config formats | None |
| Analyzer requirement planner | Map artifacts and observations to required analysis capabilities | None |
| Coverage ledger | Track complete lifecycle state for every requirement | None |
| Result intake normalizer | Validate future externally supplied envelopes | None |
| Assessment assembler | Produce canonical immutable assessment | None |
| Reference admission evaluator | Apply deterministic rules to immutable input | None |
| Existing cache/audit adapters | Store derived records and minimized events | Existing bounded writes only |

## Trust Zones

### HRA-Z1 — Context policy process

Trusted only as an approved pinned build. It owns authorization, snapshot
identity, core classification, assessment assembly, and budget enforcement.

### HRA-Z2 — Repository workspace

Always hostile. It can control bytes, paths, file counts, encodings, links,
magic values, apparent platform, comments, configuration, and instructions.

### HRA-Z3 — Derived admission cache

Sensitive and replaceable. Records are namespaced by workspace and snapshot,
schema-versioned, integrity-checked, and never the sole source of exact evidence.

### HRA-Z4 — External analyzer result boundary

Absent at runtime in step 1 except for synthetic/conformance envelopes. All
future input is hostile derived data and cannot provide exact authority or
policy instructions.

### HRA-Z5 — Policy evaluator

Receives only canonical assessment and policy bytes. It cannot read source,
resolve evidence, start a process, contact a provider, or issue a credential.

### HRA-Z6 — Consumer and human authority

Own final risk acceptance, exceptions, and any later execution authorization.
Consumer identity cannot weaken core invariants.

## Canonical Data Contracts

### Artifact inventory record

Required fields include:

- schema and contract versions;
- workspace and snapshot identities;
- lossless relative path identity and safe display path;
- content SHA-256 and exact byte size;
- regular-file confirmation and platform metadata availability;
- declared extension and bounded detected format;
- artifact classes and platform targets;
- archive/encryption/ambiguity flags;
- required analyzer capability identifiers;
- explicit classification method and limitations.

The classifier does not retain an unrestricted sample inside the record.

### Execution-surface record

Required fields include:

- stable finding and rule identifiers;
- observed or heuristic classification;
- category and bounded severity;
- exact evidence reference or artifact identity;
- ecosystem and execution stage;
- interpreter/tool name only when observed;
- potential capability classes such as process, network, credential path,
  persistence, privilege, host mount, or code download;
- limitations and required follow-up analysis.

Command text, environment values, absolute paths, and source excerpts remain in
exact evidence rather than control or audit fields.

### Analyzer requirement record

```text
requirement_id
capability_id
artifact identities
reason rule IDs
mandatory/optional
minimum compatible analyzer contract
state
state reason
analyzer/ruleset identity when attempted
started/completed/fresh-until timestamps
result envelope digest
```

State is one of `not_requested`, `pending`, `completed`, `partial`, `failed`,
`unavailable`, or `stale`.

### Security finding

A finding contains no authority. It carries:

- category, severity, confidence, and classification;
- snapshot and affected artifact identities;
- exact evidence references when observed;
- analyzer and ruleset/database provenance when derived;
- detection name as untrusted provider data;
- false-positive and limitation fields;
- remediation class, never an executable remediation command.

### Repository assessment

The canonical assessment contains ordered inventories, findings, coverage,
conflicts, unknowns, exclusions, limit events, policy-relevant facts, and
component/version digests. Its identity is a domain-separated SHA-256 over
canonical JSON under the existing hashing policy.

### Admission decision

The decision binds:

- assessment ID and snapshot ID;
- policy ID, version, and digest;
- evaluator version;
- one closed stage decision;
- matched deny/manual/incomplete/eligibility rule IDs;
- required human action and allowed next-stage profile, if any;
- `ordinary_host_execution_authorized: false`;
- `authority_added: false`.

## Request And Processing Sequence

1. Validate explicit workspace, cache, caller, purpose, and resource budget.
2. Open or refresh the existing read-only snapshot.
3. Enumerate only admitted regular files through the workspace controller.
4. Perform bounded content classification; record every skip and ambiguity.
5. Apply narrow execution-surface rules to admitted formats.
6. Derive analyzer requirements from artifact and surface facts.
7. Merge only valid, matching, fresh external result envelopes when available.
8. Compute coverage independently from finding count.
9. Assemble and hash the immutable assessment.
10. Optionally pass assessment and policy to the pure evaluator.
11. Emit the assessment/decision and metadata-first audit events.

No step invokes repository configuration, discovers tools from the repository,
or performs a network request.

## Content Classification Architecture

Classification proceeds from least risky to more specific:

1. Existing regular-file and path authorization.
2. Exact byte size and prefix budget check.
3. Conservative magic/header recognition.
4. Extension-to-magic consistency comparison.
5. Text/binary/encoding classification.
6. Narrow artifact class assignment.
7. Required analyzer planning.

Malformed data returns unknown or partial. Classification never falls back to
opening the artifact through an operating-system-associated application.

## Windows Static Admission Architecture

### Core-safe inventory

The core may recognize bounded signatures such as PE `MZ`/PE header shape and
compound-file/MSI candidates, but does not traverse arbitrary internal binary
structures until a parser receives an independent admission decision.

PowerShell, batch, registry, MSBuild, NuGet, and installer-related text/config
are retrieved as ordinary untrusted bytes. Narrow deterministic facts may be
emitted only for formats already supported by a safe core parser or a deliberately
limited lexical rule.

### Required analyzer routing

The planner assigns future capability IDs including:

- `windows.powershell.static`;
- `windows.batch.static`;
- `windows.pe.metadata`;
- `windows.authenticode.inspect`;
- `windows.msi.tables`;
- `windows.msi.custom_actions`;
- `windows.persistence.static`;
- `windows.build.execution_surface`.

Absence of a compatible analyzer keeps the assessment incomplete. It does not
block merely because the host is macOS or Linux.

## Policy Evaluation Architecture

Policy rules operate over typed fields and closed comparisons. They cannot
contain shell commands, regular expressions over raw source, model prompts, or
provider credentials.

Evaluation order is monotonic:

1. Validate policy and assessment identities.
2. Apply hard-block rules.
3. Apply incomplete-analysis rules.
4. Apply mandatory-manual-review rules.
5. Apply isolated-eligibility rules only if no prior class matched.

Adding a finding, unknown, failed requirement, or stricter policy cannot make a
decision more permissive. A human exception is a separate signed/audited
consumer record and is never written into the assessment.

## Storage And Retention

- Inventory and assessment caches are project-isolated and replaceable.
- Exact source remains in the authorized workspace, not duplicated by default.
- External detection names and provider metadata are treated as untrusted.
- Audit records store IDs, counts, versions, decisions, and failure classes, not
  source, command strings, paths, hashes sent to providers, or secret values.
- Retention uses the existing resource profile until a dedicated admission
  profile is approved.

## Error And Failure Semantics

| Failure | Result |
| --- | --- |
| Snapshot changes during analysis | Stale failure; discard assessment |
| Unsupported or malformed artifact | Explicit unknown/required analysis |
| Resource ceiling reached | Partial assessment and incomplete decision |
| External result is stale or mismatched | Reject/quarantine result |
| Policy is missing or invalid | No decision; assessment remains usable |
| Evaluator fails | No permissive fallback |
| Cache is corrupt | Rebuild or fail; never trust cached finding |

## Threat Controls

| Threat | Mandatory control |
| --- | --- |
| Prompt or control injection | Typed channels, escaped rendering, no source-derived policy |
| Parser exploitation | Minimal core parsing; future hostile parsers isolated |
| File/zip bomb | Bounded reads and explicit archive-depth/expansion planning |
| Symlink/special-file escape | Existing capability-relative regular-file boundary |
| False authority | Exact-source verification only in core; derived trust labels |
| Coverage laundering | Separate coverage ledger and no clean state |
| Policy bypass | Closed deterministic policy and monotonic denial |
| Cross-workspace result replay | Snapshot/artifact binding on every envelope |
| Sensitive log leakage | Metadata-first safe errors and audit |

## Interfaces And Versioning

The implementation should add versioned capabilities without changing existing
command semantics. Candidate future capability names are:

- `admission.inventory.build`;
- `admission.assessment.build`;
- `admission.assessment.validate`;
- `admission.decision.evaluate`.

Exact CLI, MCP, and SDK exposure requires a later interface review. MCP does not
gain execution or external-network tools merely because an assessment exists.

Schema evolution rules:

- closed schemas reject unknown control fields;
- additive finding categories require compatibility review;
- changed decision meaning requires a new major contract version and ADR;
- historical assessments are never silently re-evaluated under a new policy;
- policy and ruleset freshness remain explicit.

## Verification Architecture

### Contract tests

- Canonical inventory, finding, coverage, assessment, and decision vectors.
- Unknown/duplicate field and version rejection.
- Stable ordering, hashing, and byte-repeatability.

### Adversarial repository tests

- Path traversal, links, special files, long names, Unicode/bidi, ANSI, control
  spoofing, archive bombs, huge files, encrypted files, mismatched magic, and
  rapid mutation.
- Prompt instructions asking the engine or an agent to disable policy, run a
  command, reveal a secret, or mark the repository safe.

### Windows fixture matrix

- Benign, suspicious, malformed, truncated, oversized, and polyglot PowerShell,
  batch, PE, DLL, MSI, registry, service, MSBuild, NuGet, and installer fixtures.
- Tests run on macOS and Linux as well as the Tier A Windows host.
- Fixtures are synthetic or clearly licensed and never execute.

### Policy properties

- Determinism and stable reason ordering.
- Monotonic denial.
- No eligibility with stale or mandatory missing coverage.
- No decision value can imply ordinary-host authority.

## Implementation Sequence

The component sequence mirrors HRA-0 through HRA-5 in the PRD. Each increment
must remain releasable behind a disabled capability flag and must not require
Step 2 or Step 3 to preserve current Context behavior.

The HRA-0 prerequisites and HRA-1 inventory are complete. Before implementing
any later increment:

1. Accept ADR-0073.
2. Update the security threat model and evaluation PRD.
3. Approve exact schemas and resource ceilings.
4. Approve the initial artifact/rule matrix and fixture provenance.
5. Record an explicit founder implementation authorization for that increment.

## Rollback And Failure-Domain Preservation

- Admission modules may be disabled without disabling retrieval, structural
  graphs, packets, MCP, or existing clients.
- Derived admission cache namespaces are independently removable.
- The policy evaluator can be removed without losing assessment evidence.
- A future analyzer runner outage changes coverage to unavailable; it does not
  change core authorization or widen execution.
- A future quarantine runner outage prevents progression and cannot make an
  assessment more permissive.

## Architecture Exit Criteria

HRA-1 is complete only while its inventory validates against the frozen schema,
uses the frozen profile, reports every exclusion, preserves source bytes, and
adds no process, network, analyzer, upload, deep-parser, decision, or execution
authority. HRA-2 begins with a strict JSON parse followed by exact top-level
`scripts` object/key recovery for a closed npm lifecycle-key set. It stores only
the exact key token as evidence, never interprets the value, reports unsupported
syntax, and remains subject to the same authority exclusions. The Compose
increment uses a deliberately limited lexical rule over four exact basenames,
one canonical `services`/service/property indentation, and only
`privileged: true`; block scalars, tabs, aliases, alternative scalar syntax,
and ambiguous layouts are unsupported rather than heuristically parsed. Later ecosystems
and HRA-3 require their own reviewed rule corpus and gates. The first HRA-3
slice groups exact artifact hashes by the inventory's analyzer capability IDs,
marks every required analyzer unavailable because execution is absent, and
assembles an immutable assessment only after recomputing coverage and checking
snapshot, artifact, finding, and evidence bindings. External result envelopes
remain a separate non-executing HRA-3 slice.
