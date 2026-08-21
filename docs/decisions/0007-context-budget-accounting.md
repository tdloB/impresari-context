# ADR-0007: Context-budget accounting

- Status: Accepted for implementation baseline
- Date: 2026-08-20
- Scope: Search responses, context packets, evidence expansion, handoffs, and
  future tokenizer-aware adapters

## Context

The engine promises bounded context without depending on one model provider or
tokenizer. “Tokens” are not a stable cross-model unit, while character counts
can understate encoded output and object/item counts do not cap payload size.

Budget accounting must be enforceable before output leaves the core, must
include metadata and provenance overhead, and must remain deterministic.

## Decision

Use **UTF-8 serialized bytes as the mandatory model-neutral hard budget unit**
for the MVP.

### Budget contract

Every bounded request declares:

```yaml
budget:
  unit_kind: utf8_bytes
  requested: "12000"
  hard: true
  max_evidence_items: 50
  max_files: 20
  max_excerpt_bytes_per_item: 2000
```

Exact field types and names are fixed in the schema work, but these semantics are
required:

- `unit_kind` is mandatory; the MVP accepts only `utf8_bytes`.
- The hard byte budget covers the complete canonical machine-readable response
  or packet payload, including metadata, facts, claims, unknowns, evidence
  index, redactions, and recovery handles.
- Transport framing, terminal decoration, and filesystem container overhead are
  outside the packet budget but have separate interface/output ceilings.
- Numeric budgets are serialized without cross-language precision ambiguity.
- Item, file, excerpt, search-match, traversal, time, and memory caps remain
  independent hard limits; a large byte budget cannot disable them.

### Accounting behavior

1. Validate the requested budget against policy minimum and maximum.
2. Reserve the required schema, identity, policy, freshness, and recovery
   metadata before selecting optional evidence.
3. If required metadata cannot fit, return `budget_too_small`; do not emit an
   invalid or misleading packet.
4. Select and truncate evidence deterministically using a versioned packager
   algorithm.
5. Serialize canonically and measure exact UTF-8 bytes.
6. If the first result exceeds the hard budget, deterministically remove or
   shorten the lowest-priority optional material and reserialize.
7. Never split an invalid UTF-8 sequence, JSON token, evidence identity, or
   required provenance field.
8. Record requested, reserved, delivered, omitted, truncation reasons, limits,
   and accounting/packager versions.

The packet's own accounting fields are included in final measurement. The
implementation must converge through a bounded deterministic procedure rather
than estimating and occasionally exceeding the limit.

### Evidence selection order

The MVP packager uses this priority order, with stable score/path/span
tie-breaking inside a tier:

1. required packet identity, policy, snapshot, freshness, and accounting fields;
2. explicit exact query anchors and required evidence;
3. high-ranked diverse evidence across relevant files;
4. conflicts and safety-significant unknowns;
5. additional supporting excerpts;
6. optional explanatory metadata.

The precise ranking formula requires conformance vectors and evaluation. A
consumer cannot hide conflicts/unknowns by requesting a smaller budget; if
required safety metadata cannot fit, packet creation fails.

### Evidence expansion budgets

`evidence.expand` is a separate request with its own hard byte, span, and item
limits. Recovery content is not retroactively counted against the original
packet, and the existence of a recovery handle never grants unlimited expansion.

### Human and transport views

- Canonical structured output is the budget authority.
- Human CLI rendering has a separate maximum and may be smaller, but cannot add
  source facts absent from the structured result.
- Compressed transport bytes do not change the uncompressed context budget.
- Handoff export preserves the original packet budget record; wrapping metadata
  uses a separately declared export envelope limit.

### Future tokenizer estimates

A later adapter may add advisory or hard tokenizer-specific budgets only when it
declares:

- `unit_kind` distinct from `utf8_bytes`;
- tokenizer/model family and immutable version/digest;
- whether counting is exact or estimated;
- fallback when the tokenizer is unavailable;
- conformance fixtures and evaluation impact.

The core will continue to report UTF-8 bytes. It will not silently label a
heuristic token estimate as exact or let a model/provider choice redefine stored
packet identity without versioning.

## Rationale

UTF-8 byte length is deterministic, directly enforceable on serialized output,
independent of model vendors, and reproducible in every client language. It is
not a perfect proxy for model cost, so named tokenizer measurements remain useful
secondary evidence rather than the universal contract.

## Consequences

### Positive

- Hard limits are measurable before delivery.
- Budget semantics survive model and transport changes.
- Metadata/provenance cannot escape accounting.
- Evaluation can compare exact context sizes reproducibly.
- Consumers can add tokenizer-specific constraints without corrupting the core
  model-neutral record.

### Costs

- Byte budgets do not map uniformly to every model's tokens.
- Iterative exact serialization adds some packager work.
- JSON field names and metadata consume visible budget.
- Very small budgets may fail instead of returning an underspecified packet.

## Alternatives Considered

### One universal token count

Rejected because tokenization varies by model, encoding, version, and provider.

### Character or line count

Rejected because Unicode encoding and long lines make them weak size bounds.

### Evidence-item count only

Rejected because item size varies widely and cannot prevent oversized output.

### Estimate first and allow small overshoot

Rejected because a “hard” safety/resource budget must not be probabilistic.

### Compressed byte count

Rejected because consumers must decompress the payload and compression ratio can
hide the actual context delivered.

## Verification

- Golden fixtures for canonical serialized byte counts.
- Property tests proving delivered bytes never exceed hard request budget.
- Boundary cases where accounting fields change digit length.
- Unicode, escaping, long-path, redaction, conflict, and unknowns fixtures.
- Determinism across repeated and cross-client conformance runs.
- Budget curves in IC-EVAL-001 pairing context reduction with evidence recall.

## Review Triggers

Review if a stable cross-provider token standard emerges, JSON overhead prevents
quality targets, a binary public transport is adopted, consumers require
multiple simultaneous hard units, or evaluation shows the priority policy hides
material evidence.
