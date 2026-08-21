# Architecture

## 1. Purpose

Impresari Context, the project's internal working name, is a local-first,
client-neutral service that
turns a software workspace into a versioned set of structural facts, exact
evidence references, and bounded context packets.

It answers questions such as:

- What code is relevant to this task?
- Which symbols and files participate in a call or dependency path?
- What changed, and what may be affected?
- Which conclusions are observed, which are derived, and which remain unknown?
- Can every delivered conclusion be recovered to exact source content?

The engine does not decide what an agent should build, whether a release should
proceed, or whether a security risk is acceptable. Those decisions belong to
the consuming system and its human operators.

## 2. Architectural shape

```text
Consumer or agent
       |
       v
Transport adapter (MCP, CLI, SDK, or HTTP)
       |
       v
Capability gateway
  | policy | identity | budgets | request validation |
       |
       v
Retrieval planner
  | lexical retrieval | structural retrieval | change retrieval |
       |
       +-----------------------+
       |                       |
       v                       v
Workspace snapshot       Structural index
and exact evidence       and relationship graph
       |                       |
       +-----------+-----------+
                   v
          Evidence normalizer
     | provenance | confidence | freshness |
                   |
                   v
            Context packager
     | redaction | limits | recovery handles |
                   |
                   v
        Verifiable context packet
```

Persistence, extensions, and observability surround this path but cannot
bypass the capability gateway.

## 3. Core components

### 3.1 Workspace controller

The workspace controller establishes the only source roots the engine may
read. It produces a content-addressed workspace snapshot containing:

- canonical root identity;
- repository revision when available;
- working-tree fingerprint;
- ignore-policy fingerprint;
- parser and engine versions;
- creation time and freshness state.

It resolves symlinks before authorization, rejects paths outside approved
roots, and keeps source workspaces read-only. Cache writes go to a distinct,
explicitly configured storage root.

### 3.2 Policy and capability gateway

Every operation enters through one gateway. It applies:

- caller identity and role supplied by the consumer;
- workspace authorization;
- requested capability;
- data-sensitivity policy;
- output, time, memory, and traversal budgets;
- network, execution, and persistence restrictions;
- audit metadata.

The gateway returns a structured decision rather than relying on prompt text
for enforcement.

### 3.3 Ingestion and exact-evidence store

Ingestion discovers eligible source artifacts and records their content hashes
and metadata. Exact source remains the highest-authority evidence tier.

The store may cache source-derived fragments but must not make the cache the
only way to recover evidence. A packet reference resolves against the matching
workspace snapshot or reports that the evidence is stale; it must never
silently resolve against a different revision.

### 3.4 Structural index

The structural index builds one canonical graph for a workspace snapshot.
Parser implementations are adapters behind an original graph contract.

Initial node classes:

- workspace, package, directory, file;
- symbol, type, function, method, variable;
- import, export, route, configuration key;
- documentation concept and test.

Initial relationship classes:

- contains, declares, imports, exports;
- calls, references, implements, extends;
- tests, configures, documents;
- changed-with and possibly-impacts.

Every graph fact records its source span, extraction method, resolver version,
and confidence. Guessed or heuristically resolved edges are distinguishable
from parser-confirmed edges.

### 3.5 Retrieval planner

The planner combines bounded retrieval strategies without exposing separate,
overlapping products to the caller:

- exact path and symbol lookup;
- lexical and pattern search;
- structural-neighborhood traversal;
- caller, callee, import, and impact paths;
- change-aware retrieval;
- repository-map ranking.

The planner is deterministic by default. Optional semantic or model-assisted
retrievers may contribute candidates, but their output is labeled derived and
cannot replace exact evidence.

### 3.6 Evidence normalizer

All retrieval output becomes a common evidence record. The normalizer:

- attaches exact source references;
- identifies the extraction method;
- separates observed, derived, and asserted material;
- labels freshness and confidence;
- detects conflicting evidence;
- applies secret and sensitivity classification;
- treats repository and extension text as untrusted.

### 3.7 Context packager

The packager produces task-specific packets within a declared budget. It may
compress or summarize evidence, but it always preserves recovery references.

A packet is immutable after issuance. A materially different workspace state,
policy, graph, or engine version produces a new packet identity.

### 3.8 Session and handoff store

The session store holds temporary task state and packet references. It may
support multi-client handoffs without becoming an agent-routing system.

Durable knowledge is a separate, gated record type. Models and extensions may
propose durable facts, but only a consumer-defined approval process can promote
them. Proposed knowledge includes provenance, scope, expiration, and the
approving identity.

### 3.9 Extension host

Extensions add parsers, retrievers, analyzers, exporters, or transports through
versioned contracts. Each extension declares capabilities such as:

- workspace read;
- cache read or write;
- process execution;
- network destinations;
- environment-variable access;
- model access.

The default is no access. Extension output re-enters through evidence
normalization and cannot write directly to canonical graph or durable memory.

### 3.10 Observability and evaluation

The engine records structured operational metadata without logging source or
secrets by default:

- request and policy-decision identifiers;
- snapshot, graph, and packet identifiers;
- latency, resource use, and output size;
- evidence recovery and stale-reference rates;
- retrieval recall and correction signals supplied by evaluation harnesses;
- extension and engine versions.

Product telemetry is off by default and is distinct from local audit records.

## 4. Canonical contracts

The examples below describe required semantics, not a final serialization
format.

### 4.1 Evidence reference

```yaml
evidence_id: ev_content_hash
workspace_snapshot: ws_content_hash
artifact:
  path: src/example.ts
  content_hash: sha256:...
span:
  start_line: 10
  start_column: 1
  end_line: 18
  end_column: 2
kind: exact_source
extraction:
  method: parser
  resolver: typescript-adapter
  version: 0.1.0
confidence: confirmed
trust: untrusted_workspace_content
```

### 4.2 Derived claim

```yaml
claim_id: claim_content_hash
statement: Request handling may reach the administrative write operation.
classification: derived
evidence:
  - ev_source
  - ev_sink
derivation:
  method: structural_path
  graph_snapshot: graph_content_hash
confidence: probable
verification: exact_source_required
```

### 4.3 Context packet

```yaml
packet_id: packet_content_hash
schema_version: 1
workspace_snapshot: ws_content_hash
request_id: consumer_request_id
purpose: security_review
created_at: RFC3339 timestamp
freshness: current
policy_decision: policy_decision_id
budget:
  requested_units: 12000
  delivered_units: 8400
observed_facts: []
derived_claims: []
assumptions: []
conflicts: []
unknowns: []
evidence_index: []
recovery_handles: []
redactions: []
```

### 4.4 Knowledge proposal

```yaml
proposal_id: knowledge_content_hash
scope: workspace
statement: Authentication middleware is attached at the API router boundary.
evidence: []
proposed_by: consumer_or_extension_identity
expires_at: RFC3339 timestamp
status: proposed
```

## 5. Initial capability surface

The core begins with a small protocol-independent vocabulary:

| Capability | Purpose |
|---|---|
| `workspace.open` | Authorize and fingerprint a workspace |
| `snapshot.status` | Report revision, graph version, and freshness |
| `index.build` | Build or update the deterministic structural index |
| `code.search` | Find paths, text, symbols, and ranked structural candidates |
| `code.describe` | Return a file or symbol API with exact evidence |
| `code.trace` | Traverse callers, callees, imports, dependencies, or impact |
| `context.build` | Produce a bounded, purpose-specific context packet |
| `evidence.expand` | Recover exact content for a packet reference |
| `context.validate` | Recheck packet freshness, integrity, and evidence resolution |
| `handoff.export` | Export a scoped packet for another authorized consumer |

MCP, CLI, SDK, and future service adapters map to these capabilities. They do
not create independent semantics.

## 6. Security invariants

1. A source path is authorized after canonical resolution, not before.
2. The core never writes to the source workspace.
3. Repository content cannot change policy or tool permissions.
4. Compressed output never becomes the sole evidence for a claim.
5. Outbound network access requires an explicit destination policy.
6. Extensions cannot directly mutate canonical evidence or durable knowledge.
7. Snapshot mismatches fail visibly rather than returning apparently current
   evidence.
8. Secrets are neither logged nor sent to optional model providers by default.
9. Code execution is not an implicit property of search or analysis.
10. A consumer cannot gain capabilities by changing prompt text.

## 7. Initial delivery slices

### Slice A: verifiable local context

- one local workspace;
- read-only source access;
- deterministic snapshot and freshness;
- lexical retrieval;
- exact evidence expansion;
- bounded context packet;
- local audit metadata.

### Slice B: structural intelligence

- parser adapter contract;
- symbols and containment;
- imports, references, and calls where supported;
- repository map and trace queries;
- confidence and source-span provenance.

### Slice C: AI App Builder OS reference adapter

- translate OS task purpose and role into capability requests;
- map engine packets into OS context packets;
- preserve OS public/private and approval policy;
- require exact-source verification for consequential findings.

### Slice D: controlled extensibility

- signed or hash-pinned extension manifests;
- capability declarations and sandbox policy;
- normalized untrusted output;
- extension evaluation and quarantine.

Durable memory, remote services, semantic retrieval, and additional transports
follow only after these slices meet security and evaluation gates.

## 8. Success criteria

The architecture succeeds when:

- a client can trace every consequential claim to exact, current source;
- one graph snapshot supplies structural answers consistently;
- the engine works without a hosted model or outbound network;
- the same core serves the OS and an unrelated reference client;
- OS-specific policy remains outside the core;
- disabling an optional extension does not break evidence recovery;
- an unavailable engine can be bypassed without corrupting consumer state;
- representative evaluations improve retrieval quality or context cost over
  native tools without weakening security.
