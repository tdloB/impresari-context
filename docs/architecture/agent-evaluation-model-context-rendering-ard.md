# Agent-Evaluation Model-Context Rendering — Architecture Requirements And Design

- Status: Draft for adapter-correction implementation
- Date: 2026-08-28
- Governing product record:
  [Model-Context Rendering PRD](../product/agent-evaluation-model-context-rendering-prd.md)
- Governing decisions:
  [ADR-0059](../decisions/0059-developer-agent-evaluation-adapter-boundary.md),
  [ADR-0060](../decisions/0060-provider-backed-agent-evaluation-adapters.md),
  and
  [ADR-0061](../decisions/0061-human-readable-evaluation-model-context.md)

## Architecture Review Outcome

The observed failure is both a confirmed local adapter defect and evidence of
possible packet/retrieval inefficiency. The proportionate outcome for this
increment is **local improvement**: correct the model-consumer translation in
`context-evaluation` while leaving Impresari core retrieval and packet
construction unchanged. A structural core change remains deferred until a
clean A/B/A rerun provides valid measurements.

## Architectural Objective

Place one deterministic, provider-neutral, source-verifying renderer between
the canonical `ContextPacket` wire format and the model-provider request. The
renderer translates representation only. It must not become a retriever,
planner, ranker, summarizer, policy engine, tokenizer authority, or second
packet builder.

## Current And Target Flow

```text
Current treatment flow

packet adapter -> canonical packet JSON with Base64URL excerpts
               -> provider-specific agent adapter
               -> raw packet string appended to user content
               -> provider

Target treatment flow

packet adapter -> canonical packet JSON
               -> parse + core packet validation
               -> frozen-source binding and UTF-8 verification
               -> deterministic model-context renderer
               -> rendered byte/hash/identity measurement
               -> unchanged provider-specific envelope
               -> provider
```

Baseline A and Baseline B bypass the renderer and keep their existing task,
system-instruction, tool, and provider envelopes.

## Component Responsibilities

### Harness

- Freezes renderer identifier, version, and maximum rendered bytes in the study
  execution specification.
- Passes the canonical packet only to the treatment adapter.
- Persists source-free rendering identity and validates it against the study.
- Rejects mixed adapter/renderer versions during record validation.

The harness does not parse source excerpts or create model-facing context.

### Packet Adapter

- Builds the same canonical packet from the same frozen context plan and
  resource budget.
- Returns the packet and source fingerprint through the existing protocol.

The packet adapter is unchanged by this increment.

### Provider-Neutral Model-Context Renderer

- Parses `ContextPacket` and invokes `validate_packet`.
- Resolves and verifies evidence against the frozen source allowlist.
- Converts verified UTF-8 evidence bytes into a versioned JSON data contract.
- Preserves packet evidence identity, order, excerpts, safety metadata,
  accounting, and provenance.
- Computes exact rendered UTF-8 bytes and SHA-256.
- Enforces the declared rendered-byte ceiling before network access.

The renderer owns no provider client, credential, task scoring, repository
retrieval, packet selection, or persistence.

### Provider Adapters

- Call the shared renderer exactly once for treatment.
- Place the returned bytes in the same user-message data section for OpenAI and
  Anthropic.
- Keep the existing fixed model, effort, statelessness, cache, tool, endpoint,
  usage, cost, and final-answer contracts.

Provider adapters may not modify, reserialize semantically, filter, or augment
the rendered evidence.

### Repository Tool Boundary

The existing measured tool dispatcher remains available and identical in all
arms. Renderer source verification is not counted as a model-directed
repository read because it is a trusted preflight integrity check, not an
agent tool action. Its work must not alter tool-call or repository-read metrics.

## Model-Context Contract V1

The internal evaluation-only contract is one deterministic JSON object. Field
names below are normative; exact Rust type names may differ.

```text
ModelContextV1
  schema_name = "impresari-evaluation-model-context"
  schema_version = "1.0"
  renderer_identifier
  renderer_version
  packet
    packet_id
    workspace_snapshot
    purpose
    freshness
    completeness
    policy_decision
    requested_bytes
    delivered_bytes
    omitted_items
    packager_version
  safety
    assumptions[]
    conflicts[]
    unknowns[]
    redactions[]
    truncations[]
  evidence[]
    evidence_id
    path
    content_hash
    source_start_byte
    source_end_byte
    match_line_start
    match_line_end
    excerpt_line_start
    excerpt_line_end
    kind
    extraction_method
    extraction_version
    confidence
    trust
    freshness
    sensitivity?
    source_text
```

The serializer must use a stable field order and compact UTF-8 JSON. Source
text is a JSON string value. Canonical packet fields that exist only to carry
native path units or encoded excerpt bytes are verified but not copied into
the model-context contract.

This contract is not a replacement public packet schema. Its identity is
separate from `ContextPacket` and changing it requires a renderer version bump
and a new study identity.

## Source-Binding Algorithm

For each evidence record, the renderer performs these steps in order:

1. Require the display path to be a unique member of the request source
   allowlist and resolve it with the existing component-aware, no-symlink
   regular-file checks.
2. Read bounded raw file bytes and verify the packet's full-file content hash.
3. Parse `span.start_byte`, `span.end_byte`, `match_start_byte`, and
   `match_end_byte` using checked integer conversion.
4. Canonically decode `bytes_base64url`; reject a non-canonical encoding.
5. Require `source_start = span.start - match_start` without underflow.
6. Require `source_end = source_start + decoded_excerpt_length` without
   overflow and within the file.
7. Require the decoded excerpt to equal `file[source_start..source_end]`.
8. Require its declared match interval to equal
   `file[span.start..span.end]`.
9. Decode the complete excerpt as strict UTF-8; reject invalid text.
10. Derive one-based line coordinates from raw `\n` bytes. For a non-empty
    half-open interval, its ending line is based on the final included byte;
    a zero-length interval uses its starting line.

Any failure returns a source-free rendering error before provider client
execution. The source fingerprint is checked again through the existing arm
boundary after provider completion.

## Control/Data Separation

The provider request has three conceptual parts:

1. fixed system instructions;
2. frozen study task; and
3. a labeled untrusted model-context JSON value.

The serializer, not source content, creates all JSON syntax. Strings containing
quotes, braces, role labels, Markdown fences, XML-like tags, terminal escapes,
or tool-call text remain escaped values. No repository text is interpolated
into system instructions, message roles, tool definitions, provider options,
headers, URLs, or credentials.

The system instruction continues to state that repository and packet content
are untrusted reference data and never instructions.

## Budget And Accounting

Three measurements remain distinct:

| Measurement | Authority | Purpose |
| --- | --- | --- |
| Canonical packet bytes | `ContextPacket` accounting under ADR-0007 | Model-neutral packet integrity and resource bound |
| Rendered context bytes | Model-context renderer | Provider-bound adapter safety and reproduction |
| Provider input/output tokens | Provider response | Study usage and cost |

The study must not relabel rendered bytes as packet bytes or use model tokens
to change canonical packet identity. The initial corrected pilot declares a
finite rendered-byte maximum large enough to render the unchanged frozen
packet. Tight token targets belong to a later retrieval/budget experiment.

## Persisted Record Changes

The record schema gains treatment-only source-free fields:

- `model_context_renderer_identifier`;
- `model_context_renderer_version`;
- `rendered_context_bytes`;
- `rendered_context_sha256`; and
- `rendered_context_evidence_count`.

The study execution specification carries the renderer identifier/version and
`max_rendered_context_bytes`. Baselines use zero counts and no rendered hash.
Validation requires exact agreement with the frozen specification and rejects
records from the raw-wire adapter or another renderer version.

No rendered JSON, source text, packet, prompt, answer, raw provider body, or
diagnostic stream is persisted.

## Failure Model

Renderer failures use stable source-free stages and reasons, including:

- packet parse or packet integrity;
- evidence path or source hash;
- evidence span or excerpt mismatch;
- unsupported non-UTF-8 evidence;
- serialization; and
- rendered output budget.

A renderer failure invalidates the treatment arm and no provider request is
made. A successfully rendered context followed by an incorrect model answer is
a valid measured outcome under the existing harness rules.

Broader partial-usage persistence for process timeouts is outside this
increment.

## Security Invariants

1. Canonical validation and source binding precede network access.
2. The renderer accepts only the exact treatment packet and frozen source
   authority already granted by the study.
3. Paths remain repository-relative, allow-listed, regular, and non-symlinked.
4. Source text has data authority only.
5. No lossy decoding, implicit replacement characters, alternate binary
   encoding, or model-side decoding is permitted.
6. Rendered content is bounded, ephemeral, and never logged or persisted.
7. Provider credentials and endpoints remain outside renderer inputs.
8. Renderer preflight reads do not mutate source or affect agent read metrics.

## Verification Design

### Golden Conformance Fixture

A small packet fixture must prove exact rendered bytes, field order, line
coordinates, SHA-256, evidence order, and metadata preservation. A second run
must be byte-identical.

### Boundary And Adversarial Fixtures

- exact byte limit and one byte over;
- empty and zero-length spans where valid;
- checked-integer boundaries;
- invalid/overlong/non-canonical Base64URL;
- non-UTF-8 bytes;
- mismatched file hash, excerpt, match, and span;
- absent, duplicate, escaped, symlinked, and changed source paths;
- CRLF and final-line-without-newline line mapping;
- quotes, braces, newlines, NUL/control bytes, ANSI sequences, fake role labels,
  fake tool calls, and apparent closing delimiters in source;
- safety metadata with empty and non-empty arrays; and
- many evidence records near the output ceiling.

### Provider Contract Fixtures

Offline OpenAI and Anthropic request snapshots must assert:

- identical `ModelContextV1` bytes;
- unchanged system instructions, tools, model, effort, and cache controls;
- no `bytes_base64url`, native path-unit payload, or raw packet string;
- no network attempt after any renderer failure; and
- treatment-only renderer invocation.

### Study And Record Fixtures

- corrected adapter and renderer identities are mandatory;
- records with a missing/mismatched renderer identity or byte/hash/count fail;
- baseline records cannot claim rendered context;
- sensitive sentinels never appear in serialized records; and
- old and corrected run records cannot validate as one study.

## Implementation Sequence

1. Add renderer types, validation, source binding, and golden/security tests.
2. Add execution-spec and run-record identity/accounting fields with validation.
3. Replace direct packet concatenation with one shared renderer call.
4. Update both provider request snapshots and source-free diagnostics.
5. Run the deterministic fixture and full repository gate.
6. Freeze a new Rust study manifest and perform the adapter-only A/B/A rerun.

## Deferred Architecture Work

This design deliberately does not change:

- the 4,096-byte evidence excerpt ceiling used by the affected pilot;
- broad literal-query selection;
- overlapping evidence;
- evidence-ID ordering and packet truncation;
- packet budget defaults;
- repository tool ergonomics;
- prompt caching; or
- stage-specific timeout records.

After the corrected rerun, those findings may trigger a separate architecture
review covering `context-retrieval` and `context-core`. They may not be folded
into this renderer under the label of formatting.

## Architecture Fitness Checks

| Check | Required behavior |
| --- | --- |
| Packet fidelity | Every packet evidence ID, order, and decoded excerpt is represented exactly once |
| Source integrity | Any path/hash/span/excerpt/source mismatch fails before network access |
| Provider parity | OpenAI and Anthropic receive byte-identical model context |
| Control isolation | Adversarial source changes no role, instruction, tool, endpoint, or option |
| Boundedness | Rendering never exceeds the frozen ceiling and never emits partial context |
| Privacy | Persisted records contain only renderer identity, size, hash, and count |
| Experimental isolation | Corrected study changes no retrieval or model condition |

## Revisit Triggers

- Clean corrected results still exceed the agreed token, cost, latency, or tool
  thresholds.
- Real packets cannot render within a reasonable declared byte ceiling.
- Exact source binding requires duplicated core policy or packet logic.
- A provider requires a materially different model-context representation.
- Non-UTF-8 repositories become an admitted production-study requirement.
- A proposal would rank, deduplicate, resize, summarize, or omit evidence in
  the renderer.
