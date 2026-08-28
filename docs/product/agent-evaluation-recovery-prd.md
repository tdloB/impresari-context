# Impresari Context — Observable And Efficient Agent Evaluation PRD

## Document Control

- Product: Impresari Context.
- PRD ID/version: IC-EVAL-RECOVERY-001 / 0.1.
- Status: Implemented; Anthropic pilot complete, OpenAI preflight blocked by credential rejection.
- Date: 2026-08-28.
- Scope: Developer-only evaluation execution telemetry, explicit packet/provider limits, task-corpus admission, and relevance-preserving bounded evidence selection.
- Parent requirements: [Master PRD](master-prd.md), [Evaluation PRD](evaluation-prd.md), and [Agent-Context Evaluation Harness PRD](agent-context-evaluation-harness-prd.md).
- Governing architecture: [Agent-Evaluation Recovery ARD](../architecture/agent-evaluation-recovery-ard.md).
- Governing decision: [ADR-0062](../decisions/0062-observable-deadlines-and-relevance-preserving-evaluation-context.md).

## Evidence And Problem

Two Anthropic Rust treatments exceeded the frozen 600-second process limit. The second used the corrected human-readable renderer, proving that raw Base64URL transport was not the only cause. Both adjacent baselines completed in about nine seconds with nearly identical provider usage.

The frozen Rust packet contained ten 4,096-byte excerpts for a one-line answer: 40,960 decoded source bytes, 13,180 overlapping bytes, and competing human-readable mappings for the queried enum. Shell and Ruby each produced one much smaller evidence item. The provider adapter persisted nothing after the timed-out treatment because usage and tool activity are returned only after the entire arm succeeds.

The current production packet adapter also hardcodes its complete resource policy. These hidden values cannot be swept, frozen independently, or audited from a run record.

## Outcomes

1. Every provider turn is attributable without persisting prompts, source, packets, answers, or secrets.
2. A timeout produces a valid source-free failure record rather than erasing completed-stage evidence.
3. Provider and process deadlines form one coherent hierarchy and fail before forced termination where possible.
4. Packet limits are explicit immutable study variables.
5. Required evidence is prioritized before supporting evidence; overlapping evidence is bounded and measured.
6. Ambiguous or answer-leaking tasks fail corpus admission before live execution.
7. Offline budget curves establish evidence retention, precision, density, overlap, and size before paid runs.
8. The first corrected live gate remains Anthropic Rust A/B/A at the already approved model and high effort.

## Increment One — Observable Execution And Deadlines

### IC-EVAL-RECOVERY-FR-001 — Source-Free Progress Protocol

Provider adapters must emit a bounded JSON-lines progress protocol on stderr. Events may contain only protocol/version, provider, arm, stage, turn number, elapsed milliseconds, rendered-context measurements, provider usage, stop reason, tool-call count, and an optional hashed provider request identifier.

Events must never contain source paths, source text, excerpts, packets, prompts, answers, tool arguments/results, raw response bodies, environment values, credentials, or HTTP headers.

### IC-EVAL-RECOVERY-FR-002 — Success And Failure Records

Successful run records must persist completed per-turn source-free telemetry. A failed arm must write one bounded `failure-NNNNN.json` record containing immutable study/run identity, failure stage/reason code, elapsed time, and validated progress events. Failure records are not successful measurements and cannot be summarized with completed arms.

### IC-EVAL-RECOVERY-FR-003 — Deadline Hierarchy

The study must freeze an adapter deadline and maximum provider-request duration. The adapter must receive those limits, reserve a shutdown margin, refuse to start a request that cannot fit, and apply `min(provider_request_limit, remaining_arm_time)` to every request. The harness remains the final kill boundary.

### IC-EVAL-RECOVERY-FR-004 — Streaming Anthropic Transport

Anthropic generation must use bounded SSE streaming, reconstruct the same final Message semantics, capture progress without source content, and reject malformed, oversized, incomplete, or unexpected events. Streaming must not alter model, effort, tools, task, context, output schema, or scoring.

### IC-EVAL-RECOVERY-FR-005 — Frozen Provider Controls

Provider effort, maximum output tokens, request timeout, and turn limit must be declared in the study specification and copied to every record. The first recovery run keeps Claude Opus 5 at `high` and the prior output ceiling; effort or output-limit changes require a new study stratum.

### IC-EVAL-RECOVERY-FR-006 — Token Preflight

Before live generation, a provider-specific preflight must count the exact initial request including system instructions, tools, task, and treatment context where supported. Anthropic preflight uses its Token Counting API. Counts are source-free measurements; token counting never authorizes model generation.

## Increment Two — Explicit Budgets And Evidence Selection

### IC-EVAL-RECOVERY-FR-007 — Explicit Packet Resource Policy

The study specification and adapter request must carry every `ResourceBudget` value: requested bytes, evidence items, files, excerpt bytes per item, matches, traversal depth, elapsed milliseconds, and memory bytes. The packet adapter must not substitute hidden defaults. Records must persist the exact policy.

### IC-EVAL-RECOVERY-FR-008 — Stable Relevance Priority

Multi-step context plans preserve declared step priority. Within a step, exact path and literal anchors precede broader supporting evidence, with deterministic path/span tie-breaking. Packet size reduction must remove the lowest-priority evidence first, never select by cryptographic identifier.

### IC-EVAL-RECOVERY-FR-009 — Overlap Handling

Before packaging, evidence from the same artifact must be analyzed by absolute excerpt interval. When one retained excerpt fully covers a later match, the later duplicate excerpt may be omitted while the packet reports a stable overlap-limited unknown/truncation reason. Partial overlap must be measured; algorithms must not concatenate, summarize, or invent source.

Required-evidence checks operate on source coverage as well as evidence identity so a retained covering excerpt can satisfy an expected range.

### IC-EVAL-RECOVERY-FR-010 — Corpus Admission

Each task must declare why its answer is unique, the expected evidence, and a context plan derived without using the answer. Admission rejects tasks with competing answer interpretations, expected evidence outside the allowlist, answer text embedded in the retrieval query, or no evidence retention at the primary budget.

The Rust smoke task must identify `fmt::Display for WorkspaceError` rather than ambiguously asking about every translation of `WorkspaceErrorCode`.

### IC-EVAL-RECOVERY-FR-011 — Offline Budget Curve

The evaluation CLI must analyze admitted tasks without provider generation across declared packet/excerpt/match budgets. It records packet bytes, rendered bytes, item count, unique source bytes, overlapping bytes/fraction, expected-range coverage, evidence precision proxy, density, and first covering rank. Results contain no source text.

### IC-EVAL-RECOVERY-FR-012 — Live Gate Sequence

No paid rerun occurs until the local gate, token preflight, and offline budget curve pass. Then run complete Anthropic Rust A/B/A. Only a completed valid Rust study permits corrected Shell and Ruby studies, followed by OpenAI. A compact high-effort Rust failure triggers a separately registered effort/limit experiment; results from different strata are never pooled.

## Security And Privacy Requirements

- Progress and failure records use strict schemas, size/event ceilings, and deny unknown fields.
- Provider error bodies, raw SSE data, headers, and request IDs are not persisted; a request ID may be locally hashed.
- No new network endpoint is allowed except the fixed Anthropic Messages and Token Counting endpoints and the existing fixed OpenAI endpoint.
- Adapter environments remain cleared and receive only allow-listed secrets.
- Failure handling does not broaden filesystem, process, repository, or provider authority.
- Generated diagnostics and budget reports remain outside evaluated source roots.

## Acceptance Criteria

1. Deterministic tests prove progress events survive timeout and contain no sensitive sentinels.
2. Failure records validate independently and cannot be loaded as successful records.
3. Per-request timeouts never exceed remaining arm time minus shutdown margin.
4. Anthropic SSE fixtures cover text, tool use with partial JSON, usage, stop reasons, error events, malformed events, overflow, and incomplete streams.
5. Success records contain exact completed-turn counts and usage matching the final provider response.
6. Study schema rejects missing or unsafe provider and packet limits.
7. Packet construction preserves plan priority for selection and remains byte deterministic.
8. Fully covered overlapping excerpts are suppressed deterministically and reported.
9. Offline curves prove required-range coverage before a budget is admitted.
10. Corpus admission rejects the original ambiguous Rust wording and accepts the revised task.
11. Formatting, locked tests, Clippy with warnings denied, security checks, and `scripts/check.sh` pass.
12. A newly identified Rust study completes A/B/A within frozen limits before other live strata proceed.

## Non-Goals

- No customer-facing runtime telemetry or provider integration.
- No semantic summarization, generated source explanation, hidden relevance model, or probabilistic ranking.
- No prompt caching in the headline cold study.
- No timeout increase as a substitute for diagnosis or context efficiency.
- No performance claim from one task, one repository, or one repetition.

## Rollout And Revisit

1. Implement and validate observable deadlines without changing packet selection.
2. Reassess the Master PRD and security boundary.
3. Implement explicit resource policy, stable selection, overlap handling, corpus admission, and offline curves.
4. Reassess retrieval architecture and thresholds.
5. Preflight and run the live gate sequence.

Revisit if required evidence cannot survive a practical budget, provider parity requires different context semantics, failure telemetry risks source disclosure, or compact high-effort treatments still miss the deadline.

## Implementation And Roadmap Reassessment

Both increments are implemented in evaluation schema `1.2`. The complete local
gate passed. The admitted Rust policy retained its required line at rank one
with one evidence item and zero overlap; its Anthropic A/B/A study completed
without a timeout. Corrected Shell and Ruby Anthropic studies also completed.
The OpenAI input-token endpoint returned HTTP 401 before counting, so no OpenAI
generation was authorized.

The Master PRD and existing roadmap require no product-scope change: this work
remains developer evaluation infrastructure and adds no customer runtime
authority. The pilot results justify a larger frozen multi-task corpus and
repetition design, not a product performance claim or an Impresari Context
architecture rewrite.
