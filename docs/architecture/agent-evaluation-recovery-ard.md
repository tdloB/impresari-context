# Agent-Evaluation Recovery — Architecture Requirements And Design

- Status: Implemented
- Date: 2026-08-28
- Governing product record: [Observable And Efficient Agent Evaluation PRD](../product/agent-evaluation-recovery-prd.md)
- Governing decision: [ADR-0062](../decisions/0062-observable-deadlines-and-relevance-preserving-evaluation-context.md)

## Review Outcome

Two triggers fired: repeated opaque treatment timeouts require an evaluation-adapter improvement, and a corrected renderer still receiving a low-density, overlapping packet requires an evidence-selection change. The response is two versioned increments, not a renderer workaround or timeout increase.

## Components And Authority

```text
study manifest
  |-- provider limits ----------> harness ----------> adapter process
  |                                  |                    |
  |                                  | progress pipe <----| source-free JSONL
  |                                  | failure record     | bounded SSE/provider loop
  |                                  | success record <---|
  |
  `-- packet resource policy ---> packet adapter ---> context engine
                                      |                  |
                                      |                  `-- priority + overlap selection
                                      `--> canonical packet --> source-bound renderer
```

The harness owns execution consent, immutable study identity, subprocess bounds, validated record persistence, and final kill authority. Provider adapters own fixed endpoints, streaming envelopes, per-request deadlines, progress emission, provider parsing, and tool dispatch. The packet adapter translates an explicit study resource policy; it owns no defaults. The engine owns deterministic evidence selection. The renderer remains a fidelity boundary and does not rank or remove evidence.

## Schema Version

Evaluation schema advances from `1.1` to `1.2`. Adapter versions advance independently. Old success records remain historical and cannot mix with `1.2`. Failure records use `agent-evaluation-failure` schema `1.0` and are never accepted by `load_records`.

## Progress Protocol

Each stderr line is either one strict `AdapterProgressEvent` or a final source-free diagnostic. Progress events have a fixed marker and version and are capped by the existing stderr ceiling plus a maximum event count.

Normative stages are `rendered`, `token_counted`, `provider_request_started`, `provider_response_completed`, `tools_dispatched`, and `completed`. Events include only applicable numeric/digest metadata. The harness parses events after process completion or termination, rejects malformed marked events, ignores no unmarked successful stderr, and stores validated events in success/failure records.

The adapter emits an event only after the named stage is true. Provider usage is emitted after a complete response, so paid work survives a later failure. Provider request identifiers are SHA-256 hashed before emission.

## Deadline Model

The manifest declares:

- `command_timeout_seconds` — outer harness kill boundary;
- `provider_request_timeout_seconds` — maximum one provider request; and
- a library-defined five-second graceful-shutdown reserve.

The adapter starts a monotonic arm clock before rendering. Before each provider request it computes `remaining = command_timeout - elapsed - reserve`. If remaining is zero it returns `arm_deadline_exhausted`. Otherwise request timeout is `min(provider_request_timeout, remaining)`. A request timeout returns a bounded provider failure before the outer harness kills the process under normal scheduling.

## Anthropic Streaming State Machine

The adapter requests SSE. It accepts bounded UTF-8 lines and the documented event sequence: message start, content block start/delta/stop, message delta, and message stop. Text deltas append to text blocks. Tool-use input JSON deltas append to a bounded string then parse exactly once. Thinking/redacted blocks are preserved in the assistant history but excluded from persisted telemetry. Usage and stop reason are reconciled at message completion. Unknown, duplicate, out-of-order, oversized, or incomplete state fails closed.

No raw event or content is logged. Response headers are inspected only for an optional request ID, immediately hashed, and discarded.

## Packet Resource Policy

`PacketResourceSpec` mirrors validated `ResourceBudget` values as integers. Conversion occurs in the packet adapter and fails closed. There are no adapter-local fallback values.

Production templates declare an initial primary policy and an offline curve. The primary policy is admitted only when the expected range is covered and overlap/density thresholds pass. Values are evaluation inputs, not new global runtime defaults.

## Evidence Selection

The engine assigns stable selection precedence from plan step order, then the search result's deterministic path/span order. A seen-ID set removes exact duplicates without destroying first occurrence.

For each candidate, derive its absolute excerpt interval from the authoritative span and match offsets. A candidate is suppressed only when a previously retained candidate in the same artifact fully covers both its match interval and excerpt bytes. Suppression adds a stable `overlap_covered` limitation marker. Partially overlapping candidates remain until a later contract explicitly supports multi-match merged provenance.

The packager receives already selected evidence. It may serialize in canonical order, but byte-budget eviction must remove evidence according to explicit selection priority rather than hash order. To avoid a public schema change in this increment, core packet construction preserves caller order and callers must supply deterministic order. Conformance tests cover reordered input and admitted callers.

## Corpus Admission And Coverage

`TaskSpec` gains a source-free `uniqueness_rationale`. Admission checks structural conditions and exact expected-range coverage. Semantic ambiguity still requires human corpus review; the frozen rationale makes that review auditable.

Coverage is true when any rendered/packet excerpt from the expected path contains the entire expected byte range. It does not require the evidence match span itself to equal the expected range.

## Offline Curve

The CLI `analyze-budgets` command executes only the packet adapter and shared source-bound analyzer. It never executes an agent or reads provider credentials. Each curve point receives its own explicit resource policy and records source-free measurements. The command fails if the primary point loses expected evidence or exceeds declared overlap.

## Fitness And Test Strategy

- Unit: deadline arithmetic, progress schemas, SSE state machine, interval coverage, overlap suppression, stable priority, resource conversion.
- Negative/security: malformed/oversized progress, hostile SSE, request-ID/source sentinel audit, deadline races, symlinks/mutation, integer faults, ambiguous task fixtures.
- Integration: timed-out helper writes one failure record; successful provider fixtures reconcile turn telemetry; old schemas cannot mix.
- Regression: deterministic five-language mechanics and all existing engine/core conformance.
- Offline: Rust task budget curve proves one-line evidence survives compact policies.
- Live: newly versioned Anthropic Rust A/B/A, then Shell/Ruby, then OpenAI.

## Tradeoffs

- Preserving deterministic caller order expands the core caller contract but aligns selection with ADR-0007 relevance tiers.
- Fully covered suppression reduces redundant match identities; coverage accounting and explicit limitation preserve honesty.
- Streaming increases parser complexity but yields bounded progress and follows provider long-request guidance.
- Cold no-cache runs intentionally pay repeated-prefix costs; prompt caching remains a separate production-realism experiment.

## Review Triggers

- Caller-order determinism cannot be proven across every packet builder.
- Overlap suppression removes required-range coverage.
- Streaming changes provider-visible semantics or usage accounting.
- Failure records expose source-derived data.
- Compact high-effort Rust still times out.

## Post-Implementation Review

The adapter and evidence-selection increments remained separately testable and
reversible. The compact Anthropic Rust gate completed, so the repeated timeout
does not currently justify a broader Context architecture change. The observed
baseline drift and single-repetition corpus do require more samples before any
headline product claim. OpenAI remains an external credential follow-up, not an
architecture blocker.
