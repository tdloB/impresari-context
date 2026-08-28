# System boundaries

## 1. Product boundary

The open-source project is a context and evidence engine. It is not an agent
operating system, autonomous developer, IDE manager, hosted model gateway, or
business-policy authority.

The boundary exists to keep the public core broadly useful and prevent the AI
App Builder OS from becoming an implicit requirement.

## 2. Responsibility matrix

| Responsibility | OSS core | Consumer adapter | AI App Builder OS |
|---|---:|---:|---:|
| Workspace fingerprinting | Owns | Configures | Requests |
| Path authorization mechanism | Owns | Supplies roots | Supplies project scope |
| Structural index and graph | Owns | None | Consumes |
| Retrieval and exact evidence | Owns | Maps protocol | Consumes and verifies |
| Context packet base schema | Owns | Translates extensions | Adds OS fields |
| Compression and recovery | Owns | None | Sets task budgets |
| Secret-classification mechanism | Owns | Supplies optional rules | Defines OS data policy |
| Session and handoff primitives | Owns | Maps identities | Owns routing meaning |
| Agent identity | Treats as opaque | Translates | Owns |
| Agent routing and hierarchy | Never owns | Never owns | Owns |
| Workflow phases | Never owns | Never owns | Owns |
| Go/no-go and risk acceptance | Never owns | Never owns | Owns |
| Durable-knowledge mechanism | Owns proposal records | Maps approval | Owns promotion decision |
| Source-code changes | Never owns initially | Separate tool | OS-controlled workflow |
| External actions and publishing | Never owns | Separate system | Approval policy only |
| Runtime transport | Defines semantics | Implements | Selects/configures |

## 3. AI App Builder OS adapter boundary

The OS integration is a thin adapter maintained in the OS repository or as a
separately versioned adapter package. The OS depends on a pinned public-core
release. Core code is not copied into the OS repository.

### Inputs from the OS

- workspace root and expected revision;
- opaque project and task identifiers;
- caller identity and OS role;
- purpose such as audit, planning, implementation, or verification;
- permitted sensitivity and publication class;
- context and traversal budgets;
- required evidence and freshness thresholds.

### Outputs to the OS

- immutable context packet;
- exact evidence references and recovery handles;
- snapshot and graph identities;
- observed facts and derived claims as separate records;
- conflicts, assumptions, unknowns, and redactions;
- policy and freshness status;
- local audit identifiers.

### Information that stays in the OS

- private agent prompts and definitions;
- hierarchy and routing rules;
- founder or human approvals;
- phase gates and readiness decisions;
- public/private export decisions;
- customer, commercial, and organization-specific policies;
- credentials and external-action authority;
- final risk acceptance.

The adapter must not teach the core the OS hierarchy. A role such as
`security-reviewer` is passed as an opaque policy subject, not compiled into
the context engine.

## 4. Trust zones

### Zone A: engine control plane

Trusted only to the degree established by pinned builds and local policy.
Includes request validation, policy evaluation, and audit metadata.

### Zone B: source workspace

Always untrusted. Code, comments, documentation, filenames, Git metadata, and
configuration may contain malicious instructions or secrets.

### Zone C: derived cache

Sensitive and replaceable. It may contain source-derived data and must be
isolated by workspace identity. Corruption cannot be allowed to silently alter
exact evidence.

### Zone D: extensions

Untrusted by default. Each extension receives only declared capabilities and
its output is normalized as untrusted derived evidence.

### Zone E: consumer

Controls purpose, roles, and decisions but cannot override engine safety
invariants through prompt or packet content.

### Zone F: external network and model providers

Denied by default. Enabling access requires destination allowlists, data
classification, redaction policy, retention disclosure, and an auditable
consumer decision.

ADR-0060 defines one developer-only exception for explicitly consented agent
evaluation. It does not authorize network or model access in the OSS core,
MCP, extensions, consumer adapters, or product runtime.

## 5. Process and storage boundaries

The preferred local deployment uses separate roots:

```text
source workspace: read-only
engine cache:     read-write, project-isolated, replaceable
audit store:      append-oriented, metadata-first
export area:      explicit, scoped, user-controlled
```

For higher-risk work, the index builder and query service may run as separate
processes:

- the builder receives bounded workspace-read and cache-write access;
- the query service receives cache-read and exact-source-read access;
- neither receives network or general shell access;
- an extension receives only the minimum subset required by its manifest.

## 6. Explicit non-goals

The initial open-source core will not:

- orchestrate agents or choose which agent acts next;
- decide whether a project, pull request, or release should proceed;
- rewrite model-provider requests or base URLs;
- install global shell hooks or modify editor configuration;
- update itself or extensions automatically;
- execute arbitrary repository commands;
- edit, refactor, commit, push, publish, or deploy code;
- require a hosted LLM to build the canonical structural graph;
- treat model-generated summaries as verified facts;
- provide a second graph alongside another canonical graph;
- silently promote session content into durable memory;
- expose a dashboard beyond the local trust boundary by default;
- become a secrets manager, identity provider, or policy authoring authority;
- embed AI App Builder OS prompts, phases, or agent taxonomy.

These are product boundaries, not permanent prohibitions. Crossing one requires
a separate architecture decision, threat-model update, and evaluation gate.

## 7. Extension boundary

An extension contract must declare:

- identity, version, publisher, and artifact digest;
- compatible engine contract versions;
- operation types;
- filesystem roots and access modes;
- process and network capabilities;
- allowed environment keys;
- expected input and maximum output;
- determinism and model-dependency status;
- data-retention behavior;
- evidence and provenance fields it can produce.

No extension may:

- write directly into canonical stores;
- bypass redaction or policy evaluation;
- claim exact-source authority without a matching source hash and span;
- convert repository content into control instructions;
- broaden its capabilities at runtime;
- update itself inside an engine session.

The Slice D v1 contract is intentionally narrower than an extension host. It
validates digest-pinned declarations and bounded externally supplied output but
does not load or execute artifacts. Every privileged capability remains denied.
Declaring an operation as a `transport` does not expose MCP, HTTP, or a socket.

## 8. Failure boundaries

- Index failure leaves exact reads available and marks structural results
  unavailable.
- Stale graph state is reported, never concealed by query-time inference.
- Extension failure is isolated to the requested optional capability.
- Audit failure blocks privileged or outbound operations; local read-only
  retrieval may continue according to policy.
- Redaction uncertainty fails closed before outbound transmission.
- Engine unavailability does not change OS decisions or approvals; the OS may
  use a separately governed native-read fallback.

## 9. Versioning boundaries

Version independently:

- public capability contract;
- packet and evidence schemas;
- engine implementation;
- parser and resolver adapters;
- extension API;
- AI App Builder OS adapter;
- OS policy configuration.

Packets record every version required to reproduce or invalidate their
evidence. The OS may upgrade its adapter without forcing an engine release, and
the engine may release internal improvements without changing OS policy.
