# Impresari Context — Agent-Evaluation Model-Context Rendering PRD

## Document Control

- Product: Impresari Context.
- PRD ID/version: IC-EVAL-RENDER-001 / 0.1.
- Status: Implemented and locally validated; corrected Anthropic smoke
  completed through the separately governed recovery increment.
- Date: 2026-08-28.
- Scope: Developer-only model-facing rendering of a validated treatment packet
  in `context-evaluation`.
- Parent requirements:
  [Agent-Context Evaluation Harness PRD](agent-context-evaluation-harness-prd.md),
  [Evaluation PRD](evaluation-prd.md), and
  [Master PRD](master-prd.md).
- Governing design:
  [Agent-Evaluation Model-Context Rendering ARD](../architecture/agent-evaluation-model-context-rendering-ard.md).
- Governing decisions:
  [ADR-0059](../decisions/0059-developer-agent-evaluation-adapter-boundary.md),
  [ADR-0060](../decisions/0060-provider-backed-agent-evaluation-adapters.md),
  and
  [ADR-0061](../decisions/0061-human-readable-evaluation-model-context.md).

## Problem

Before this correction, the treatment adapter appended the canonical
serialized `ContextPacket` directly to the provider prompt. That packet is a
machine wire format. Its exact source excerpts are stored as Base64URL so
arbitrary bytes can be represented canonically and verified across consumers.
Base64URL is not an appropriate model-facing source representation.

The first Rust pilot exposed the consequence. Its valid 64,993-byte packet
contained ten evidence records, with encoded excerpts accounting for most of
the payload. The initial treatment user content counted as 55,659 Anthropic
tokens. Rendering the same excerpts as verified UTF-8 text, without changing
their selection or size, counted as 16,856 tokens. The treatment then reached
the 600-second process timeout. These observations diagnose the adapter path;
they are not product-performance claims.

The current treatment results therefore do not isolate the value of Impresari
Context. They measure a model consuming a machine transport encoding that a
normal consumer should translate first.

## Purpose

Add a provider-neutral rendering boundary that converts one validated,
source-bound canonical packet into deterministic, readable, untrusted model
context before either production provider adapter sends it.

The correction must preserve the packet as the integrity and accounting
authority. It must not hide or compensate for retrieval behavior. The first
post-fix study must differ from the affected study only by the model-facing
rendering contract and the required adapter/specification version changes.

## Users And Decisions

Maintainers and evaluators use this correction to decide:

1. whether the A/B/A harness is presenting treatment evidence fairly;
2. whether a treatment result can be attributed to the packet rather than its
   wire encoding;
3. whether the same packet produces bounded, deterministic model context for
   OpenAI and Anthropic; and
4. which retrieval or packet-design changes, if any, are justified by a clean
   rerun.

## Product Principles

1. **Canonical packet remains authoritative.** Rendering never changes packet
   identity, packet bytes, evidence selection, or packet accounting.
2. **Models receive readable evidence.** Exact verified UTF-8 source text is
   presented as data; Base64URL source bytes are not sent to the provider.
3. **One treatment variable changes.** Retrieval query, selected evidence,
   ordering, excerpt extent, model, tools, effort, turns, timeout, and pricing
   remain frozen for the adapter-correction rerun.
4. **Untrusted source remains data.** Repository text cannot create prompt,
   tool, policy, or instruction authority.
5. **Failures are measurements only when safe.** A wrong answer remains a valid
   outcome; malformed, stale, unbound, non-UTF-8, or oversized context fails
   before provider transmission.

## Functional Requirements

### IC-EVAL-RENDER-FR-001 — Parse And Validate The Canonical Packet

The treatment adapter must parse the packet into the public `ContextPacket`
type and call the core packet validator before rendering. Unknown fields,
invalid canonical values, invalid packet identity, invalid accounting, invalid
Base64URL, or any other packet validation failure must fail closed before a
provider request.

### IC-EVAL-RENDER-FR-002 — Bind Every Excerpt To Frozen Source

For every evidence record, the renderer must:

- resolve exactly one repository-relative display path in the frozen source
  allowlist through the existing root-containment and symlink checks;
- verify the complete file content hash;
- decode the excerpt using canonical Base64URL;
- derive its absolute source interval from the evidence span and match offsets;
- verify the decoded excerpt and match bytes against the frozen file; and
- reject arithmetic overflow, underflow, out-of-range spans, ambiguous paths,
  source changes, or inconsistent bytes.

Internal packet validity alone is insufficient because the adapter is the last
trusted boundary before source leaves the host.

### IC-EVAL-RENDER-FR-003 — Render A Versioned Provider-Neutral Contract

The shared renderer must emit one deterministic UTF-8 JSON value whose source
strings are JSON-escaped data values. The contract must include:

- renderer identifier and version;
- packet identity, snapshot, purpose, freshness, completeness, policy decision,
  accounting, and packager version;
- assumptions, conflicts, unknowns, redactions, and truncations;
- every evidence record in packet order;
- evidence identity, verified repository-relative display path, content hash,
  source and match byte intervals, derived line coordinates, evidence kind,
  extraction provenance, confidence, trust, freshness, and sensitivity; and
- decoded exact source text.

Machine-only path-unit encodings and `bytes_base64url` must not appear in the
provider-bound representation. The canonical packet remains available only in
bounded adapter memory for validation and source-free measurement.

### IC-EVAL-RENDER-FR-004 — Preserve Evidence Without Retrieval Changes

The renderer must preserve all evidence records, their order, and their exact
excerpt bytes. It must not rank, deduplicate, merge, summarize, resize, or omit
evidence. It must not rewrite the task query or request a new packet.

This requirement intentionally leaves observed overlap, broad matching,
excerpt size, and evidence ordering visible for later architectural analysis.

### IC-EVAL-RENDER-FR-005 — Keep Control And Data Separate

System instructions, frozen task text, and rendered context must remain
distinct labeled sections. The complete rendering is serialized by trusted
code; repository content is never concatenated as an instruction or used to
construct a field name, delimiter, tool definition, role, or provider option.

Both provider paths must use the same rendered bytes. Provider-specific code
may translate message envelopes only; it may not alter model context.

### IC-EVAL-RENDER-FR-006 — Enforce A Frozen Rendering Budget

The execution specification must declare a finite
`max_rendered_context_bytes`, subject to a library-defined maximum. Rendering
must complete and be measured before any provider request. An over-limit
rendering fails closed.

The corrected pilot must freeze this value before execution. This byte ceiling
is an adapter safety bound, not a new Impresari packet budget or a claim that
bytes predict provider tokens uniformly.

### IC-EVAL-RENDER-FR-007 — Record Source-Free Rendering Identity

Treatment records must include:

- renderer identifier and version;
- rendered-context UTF-8 byte count;
- SHA-256 of the exact rendered bytes; and
- rendered evidence count.

Baseline records must carry no rendered-context hash and zero rendered bytes
and evidence count. Records must continue to omit packet contents, rendered
context, excerpts, prompts, answers, raw provider bodies, and secrets.

### IC-EVAL-RENDER-FR-008 — Version And Isolate Corrected Results

The agent adapter version, renderer version, execution manifest, and study ID
must change before the corrected run. Records made with the raw-wire adapter
must not be pooled with corrected treatment records.

Existing baseline records may remain diagnostic evidence, but the complete
corrected A/B/A study must rerun all three arms so matched ordering and provider
conditions are preserved.

## Nonfunctional Requirements

### Determinism

The same canonical packet, frozen source bytes, renderer version, and execution
limits must produce byte-identical rendered context and the same SHA-256 on
repeated offline runs.

### Security And Privacy

- Validation, source binding, UTF-8 decoding, and budget checks happen before
  network access.
- Non-UTF-8 evidence fails closed for this text-model adapter; it is not
  silently replaced, lossy-decoded, or transmitted in another binary encoding.
- Source text remains explicitly untrusted and cannot alter provider tools,
  roles, system instructions, credentials, endpoints, or execution limits.
- No new filesystem, process, network, model, environment, or persistence
  authority is introduced.
- Diagnostics identify a source-free stage and stable reason without including
  source, packet, prompt, answer, or secret data.

### Provider Parity

OpenAI and Anthropic must receive the same renderer output for the same packet.
Provider request snapshots may differ only in the provider-specific envelope
defined by ADR-0060.

### Performance

Rendering must be bounded by the canonical packet size, source allowlist, and
declared output ceiling. The implementation must not make a provider request
or begin a tool loop until rendering succeeds.

## Acceptance Criteria

1. A valid packet is parsed, validated, source-bound, and rendered
   deterministically before either provider request.
2. The rendered provider content contains decoded exact UTF-8 source and
   contains neither the `bytes_base64url` field nor its encoded source value.
3. Every packet evidence record appears exactly once in original packet order;
   no excerpt is shortened, merged, deduplicated, or reordered.
4. Packet identity, safety metadata, provenance, trust, freshness, completeness,
   and omission/accounting information survive the translation.
5. Rendered paths, hashes, spans, match offsets, and source bytes are verified
   against the frozen allowlist before network access.
6. Tests reject malformed packets, tampering, invalid Base64URL, non-UTF-8,
   source/hash/span mismatch, root escape, symlinks, ambiguous paths, mutation,
   arithmetic faults, and rendered output overflow.
7. Prompt-injection fixtures containing role labels, JSON syntax, delimiters,
   ANSI/control characters, and tool-like text remain escaped source values and
   do not change the request envelope.
8. OpenAI and Anthropic offline request-shape tests prove byte-identical model
   context and absence of the raw canonical packet.
9. Run-record tests prove rendering identity is persisted without source,
   prompt, packet, answer, raw output, environment values, or secrets.
10. Baseline prompt snapshots remain unchanged apart from the explicit schema
    and adapter version fields required by the new study version.
11. The corrected Rust A/B/A smoke completes under the frozen limits before
    Shell, Ruby, or a broader corpus is rerun.
12. Formatting, locked Clippy/tests/doc-tests, security checks, and the full
    `scripts/check.sh` gate pass.

## Required Test Coverage

| Layer | Required evidence |
| --- | --- |
| Unit | Packet parsing, source binding, line derivation, escaping, stable bytes/hash, order preservation, and byte ceiling |
| Negative/security | Tamper, invalid encoding, non-UTF-8, hash/span/path mismatch, traversal, symlink, mutation, prompt injection, control bytes, and overflow |
| Provider contract | Identical shared rendering in OpenAI and Anthropic request snapshots; no raw packet or Base64URL excerpt |
| Record contract | Treatment renderer metadata present; baseline metadata empty; sensitive sentinel audit passes |
| Regression | Existing deterministic five-language harness fixture and all baseline behavior remain valid |
| Live smoke | New Anthropic Rust A/B/A first; other language/provider runs only after it passes |

## Rollout

1. Add the provider-neutral renderer and its offline security tests.
2. Add frozen renderer identity, limits, and source-free record fields.
3. Route both provider adapters through the renderer and prove parity offline.
4. Run the complete local validation gate.
5. Freeze a new study manifest and rerun Anthropic Rust A/B/A under the
   previously agreed model, effort, tools, retrieval plan, packet budget, turn
   limit, timeout, and pricing schedule.
6. If Rust completes with valid records, rerun Shell and Ruby, then repair the
   OpenAI credential gate before any OpenAI paid run.
7. Use corrected results to decide whether a separate retrieval/packet
   architecture PRD, ARD, and ADR are warranted.

## Non-Goals

- Changing `context-core`, `context-retrieval`, packet schemas, or canonical
  packet byte accounting.
- Changing queries, match caps, evidence ranking, deduplication, overlap,
  excerpt extent, truncation, or packet construction.
- Adding prompt caching, conversation reuse, tools, provider models, effort
  levels, timeouts, retries, or provider endpoints.
- Treating an 803-token diagnostic rendering, which combined narrower retrieval
  and compact excerpts, as the acceptance target for this adapter-only fix.
- Claiming that the Rust timeout proves an Impresari core performance defect.
- Restoring or publishing the contaminated treatment comparison as product
  performance evidence.
- Redesigning failure-record telemetry beyond source-free renderer diagnostics.

## Risks And Controls

| Risk | Control |
| --- | --- |
| Renderer silently improves retrieval | Preserve every evidence record, order, and exact excerpt; freeze the old retrieval plan |
| Source spoofing survives packet validation | Rebind each decoded excerpt to the allow-listed frozen file and content hash |
| Repository prompt injection crosses into control | Trusted JSON serialization, explicit untrusted-data labeling, fixed system/tool envelopes |
| New output is still too large | Finite rendered-byte ceiling and observed provider token/cost reporting; address retrieval separately |
| Old and corrected results are pooled | New adapter/renderer versions and study ID; complete A/B/A rerun |
| Model-facing format becomes an accidental public packet schema | Keep it evaluation-only, provider-neutral, internal, and separately versioned |

## Requirement-To-Evidence Matrix

| Requirement | Acceptance evidence |
| --- | --- |
| FR-001/002 packet and source integrity | Unit, tamper, path, mutation, and source-binding tests |
| FR-003 readable deterministic rendering | Golden rendering fixture plus repeated byte/hash comparison |
| FR-004 no retrieval changes | Packet-to-rendered evidence identity/order/excerpt equivalence test |
| FR-005 control/data separation | Injection fixtures and provider request snapshots |
| FR-006 hard rendering bound | At-limit and over-limit tests proving no provider call |
| FR-007 source-free observability | Run-record schema and sentinel non-persistence tests |
| FR-008 result isolation | Study validation rejects mixed renderer/adapter versions |
| Provider parity | Shared-renderer and provider-envelope tests |
| Release readiness | Full local gate and corrected Rust A/B/A record validation |

## Open Questions Deferred From This Increment

- Should core retrieval use smaller match-centered windows?
- Should overlapping evidence be merged or deduplicated?
- Should candidate relevance order replace hash-based packet ordering?
- Should task-class-specific packet budgets be added?
- Should a separate realistic study stratum permit within-arm prompt caching?

Those questions require clean post-fix measurements and a separate architecture
review. They must not be answered inside the adapter correction.

## Post-Implementation Outcome

The renderer passed its local fidelity, source-binding, provider-parity,
privacy, and boundedness checks. A renderer-corrected Rust rerun then exposed
separate deadline observability, task ambiguity, packet-budget, relevance, and
overlap concerns. Those concerns were not hidden inside this renderer; they
were governed and implemented separately by the
[Recovery PRD](agent-evaluation-recovery-prd.md) and
[ADR-0062](../decisions/0062-observable-deadlines-and-relevance-preserving-evaluation-context.md).

After that separate increment, corrected Anthropic Rust, Shell, and Ruby A/B/A
smoke studies completed. This satisfies the renderer's mechanics gate but does
not create a statistically powered product-performance claim.
