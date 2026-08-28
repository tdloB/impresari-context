# ADR-0061: Human-readable model context for agent evaluation

- Status: Accepted and implemented; corrected Anthropic smoke completed
- Date: 2026-08-28
- Scope: Developer-only provider adapters in `context-evaluation`

## Context

ADR-0059 permits bounded developer evaluation adapters, and ADR-0060 permits
fixed OpenAI and Anthropic provider adapters. The treatment path present when
this decision was proposed passed the canonical serialized `ContextPacket`
directly to the model. Exact source excerpts in that packet are Base64URL
because the packet is a
byte-authoritative, provider-neutral machine contract.

The first Rust pilot produced a valid 64,993-byte packet but an initial
treatment user message of 55,659 Anthropic tokens. The same selected excerpts
decoded as UTF-8 text counted as 16,856 tokens. Packet construction completed
locally in less than 0.2 seconds, while the treatment arm reached its
600-second process timeout. This confirms that raw wire-format delivery is an
adapter defect. It also leaves separate questions about retrieval breadth,
overlap, excerpt size, and evidence ordering, but those cannot be measured
fairly while the adapter representation is wrong.

The architecture needs a durable distinction between:

- the canonical packet used for integrity, portability, and byte accounting;
  and
- the model-facing rendering used by a particular trusted consumer.

## Decision

Add one provider-neutral, evaluation-only model-context renderer between the
canonical packet and both provider request envelopes.

The renderer will parse the public `ContextPacket` type, invoke core packet
validation, canonically decode every excerpt, and bind every evidence item to
the exact allow-listed frozen source file, full-file content hash, byte span,
match interval, and excerpt bytes. Unsupported non-UTF-8 evidence or any
integrity, path, arithmetic, source, or budget failure will stop before network
access.

The renderer will serialize a separately identified and versioned UTF-8 JSON
data contract. It will include packet identity and accounting, safety metadata,
provenance, trust/freshness information, derived source line coordinates, and
the exact decoded source text. Source text will be JSON-escaped and explicitly
labeled untrusted. Machine-only path-unit encodings and Base64URL excerpt
fields will not be sent to the model.

The canonical packet remains unchanged and remains the packet integrity and
model-neutral budget authority. The model-context rendering receives its own
finite byte ceiling, stable renderer identifier/version, exact byte count, and
SHA-256. Persisted study records contain only those source-free measurements,
not the rendering.

The renderer will preserve all evidence records in packet order and preserve
every exact decoded excerpt. It will not rank, deduplicate, merge, summarize,
resize, omit, or retrieve evidence. Both providers will receive identical
rendered bytes; provider-specific code may change only the API envelope.

The first corrected study will use a new adapter version, renderer version,
execution manifest, and study identity. It will rerun Baseline A, Treatment,
and Baseline B. Raw-wire treatment results will not be pooled with corrected
results or used as Impresari product-performance evidence.

## Architectural Classification

This is a local adapter correction. It does not authorize changes to
`context-core`, `context-retrieval`, packet schemas, packet budgets, evidence
selection, provider authority, provider endpoints, model settings, tools,
caching, or persistence.

Possible retrieval and packet architecture weaknesses remain recorded as
revisit triggers. They require valid post-correction measurements and a
separate PRD, ARD, and ADR if pursued.

## Reconciliation With Existing Decisions

- **ADR-0001:** preserved. The adapter translates a public core contract for a
  consumer without reimplementing retrieval, policy, packet construction, or
  evidence selection.
- **ADR-0005:** preserved. Canonical packet bytes and packet identity do not
  change. The rendered context has a separate identity and version.
- **ADR-0007:** preserved. Canonical UTF-8 packet bytes remain the mandatory
  model-neutral hard budget. Rendered bytes and provider tokens are separate
  consumer measurements.
- **ADR-0059:** preserved. Rendering occurs only in the explicitly consented,
  bounded developer evaluation adapter and adds no persistence or authority.
- **ADR-0060:** narrowed. Provider adapters may no longer append the raw packet;
  they must consume the shared validated rendering. Model, endpoint, tool,
  cache, credential, and cold-arm decisions remain unchanged.
- **Threat model:** source remains untrusted data. JSON escaping plus fixed
  message/tool envelopes prevents repository text from gaining control
  authority; source binding detects stale or spoofed evidence before egress.

## Alternatives Considered

### Continue sending the canonical packet

Rejected. The observed token amplification and timeout show that a binary-safe
wire representation is not a suitable human/model consumption format.

### Change the core packet schema to store plain source text

Rejected. That would weaken the byte-authoritative cross-platform packet
contract and make one model-consumer need a core schema decision. The adapter
is the correct translation boundary.

### Decode Base64URL but send an ad hoc Markdown or delimiter format

Rejected. Repository text can imitate delimiters, roles, or tool syntax. A
trusted JSON serializer provides deterministic escaping and a testable data
boundary.

### Compact, deduplicate, reorder, or summarize evidence in the renderer

Rejected for this increment. Those operations change evidence selection or
semantics, hide possible retrieval weaknesses, and prevent an adapter-only
comparison.

### Reduce excerpt sizes or narrow the query at the same time

Deferred. Diagnostics show those changes could reduce the Rust prompt much
further, but they are retrieval/study-design variables. They require a clean
renderer result and separate architecture approval.

### Let each provider adapter render independently

Rejected. Duplicate renderers could create provider-dependent evidence,
confound cross-provider analysis, and double the integrity attack surface.

### Let the model decode Base64URL through a tool or instruction

Rejected. It wastes model/tool capacity, complicates accounting, and delegates
a deterministic trusted transformation to an untrusted probabilistic actor.

## Consequences

- Treatment prompts contain readable exact source instead of encoded source.
- Canonical packet identity and model-neutral accounting remain stable.
- Rendering adds bounded local source reads before the provider request; these
  are integrity checks and are excluded from model tool-read metrics.
- Study and record schemas gain renderer identity and source-free size/hash
  fields.
- Non-UTF-8 packet evidence is not supported by the initial text-model
  renderer and fails before egress.
- The corrected prompt may still be inefficient because evidence breadth,
  overlap, size, and order remain unchanged by design.
- Old treatment comparisons are retained only as diagnostic history, not
  product-performance evidence.

## Verification

- Golden tests for deterministic rendering bytes, SHA-256, field order, line
  coordinates, metadata preservation, and repeatability.
- Equivalence tests proving every evidence ID, order, and exact decoded excerpt
  is preserved once.
- Negative tests for malformed/tampered packets, non-canonical Base64URL,
  non-UTF-8, path/root/symlink/ambiguity failures, file hash mismatch, span and
  excerpt mismatch, source mutation, integer faults, and output overflow.
- Prompt-injection tests proving source text cannot alter system instructions,
  message roles, tools, provider parameters, endpoint, or credentials.
- OpenAI and Anthropic request snapshots proving identical shared rendered
  bytes and absence of raw packet/Base64URL source.
- Record tests proving renderer identity/size/hash/count without persisted
  source, packet, prompt, answer, raw provider output, environment, or secrets.
- A new Anthropic Rust A/B/A smoke under otherwise frozen conditions, followed
  by Shell and Ruby only after the Rust gate passes.
- Full locked repository formatting, lint, test, documentation, dependency,
  and security gate.

## Review Triggers

- Rendering requires changes to public packet schemas or core evidence policy.
- Evidence is ranked, deduplicated, resized, summarized, or omitted after
  packet construction.
- A renderer limit becomes a substitute for canonical packet budgeting.
- OpenAI and Anthropic require different evidence semantics.
- Non-UTF-8 source must enter a live provider study.
- Rendered context is persisted, logged, cached, or reused across arms.
- Corrected studies still miss token, cost, latency, correctness, evidence, or
  tool-read thresholds and a core retrieval change is proposed.

## Implementation Outcome

The shared renderer passed its local integrity, parity, boundedness, and
privacy checks. The first corrected rerun exposed independent deadline and
evidence-selection concerns. ADR-0062 governed those changes rather than
expanding this renderer's responsibility. After that increment, corrected
Anthropic Rust, Shell, and Ruby A/B/A smoke studies completed. The results
remain one-task mechanics evidence and are not a product-wide claim.
