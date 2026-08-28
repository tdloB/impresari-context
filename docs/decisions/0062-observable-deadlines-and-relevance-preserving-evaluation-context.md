# ADR-0062: Observable deadlines and relevance-preserving evaluation context

- Status: Accepted and implemented
- Date: 2026-08-28
- Scope: Developer evaluation harness plus deterministic context selection used by its production packet adapter

## Context

ADR-0061 corrected the model-facing packet representation. A controlled rerun still timed out after 600 seconds. The adjacent baseline remained stable, but no treatment usage or turn record survived. Offline reconstruction found ten full-size Rust excerpts, substantial overlap, a broad ambiguous query, hidden packet limits, and relevance-insensitive evidence ordering.

The renderer must remain a fidelity boundary. Removing evidence there would hide retrieval behavior and corrupt the experiment. Blindly increasing the timeout would spend more without identifying the failed stage.

## Decision

Adopt two separate, versioned increments.

First, make provider execution observable and deadline-aware. Adapters emit bounded source-free progress, the harness persists failure records, every provider request fits within the remaining arm deadline, Anthropic uses bounded streaming, and provider effort/output/request limits become immutable study fields.

Second, make packet policy explicit and selection relevance-preserving. The study supplies the complete resource policy. Plan-step order and deterministic path/span order define selection priority. Exact duplicates and fully covered overlaps are removed before packaging with explicit limitation metadata. Packet eviction follows selection priority rather than evidence hash. Corpus admission and offline budget curves gate live execution.

The first post-change live study retains Claude Opus 5 high effort and cold no-cache behavior. Effort, output-limit, or caching experiments require separate study identities and complete A/B/A arms.

## Consequences

- Paid work completed before a timeout becomes visible without retaining source or responses.
- Failure artifacts are auditable but never count as successful measurements.
- Packet identities change when evidence ordering/selection changes; affected schemas, adapter versions, fixtures, and studies must advance.
- The packet adapter loses hidden defaults.
- Core evidence selection becomes deterministic by relevance precedence rather than digest accident.
- Provider streaming and progress parsing add code and adversarial-test burden.
- Smaller packets are accepted only when expected-range coverage remains intact.

## Alternatives Rejected

### Increase the timeout

Rejected because two opaque failures already provide no evidence that more time would complete or be cost-effective.

### Remove or summarize evidence in the renderer

Rejected because the renderer must expose, not conceal, retrieval behavior.

### Lower the packet byte limit without priority changes

Rejected because hash-based eviction can discard the required evidence.

### Change to medium effort immediately

Rejected for the first causal rerun because it changes the model condition together with retrieval. It remains an approved later stratum if compact high-effort context fails.

### Enable prompt caching in the cold study

Rejected because cross-arm cache state can violate cold isolation. A separately designed intra-arm cache experiment remains possible.

## Security And Privacy

Progress/failure schemas are source-free, secret-free, bounded, and deny unknown fields. Raw SSE, provider bodies, prompts, tool data, source, packets, answers, headers, and credentials are never persisted. Existing environment clearing, source containment, symlink rejection, consent, and fixed endpoints remain mandatory.

## Fitness Checks

- Timeout fixtures persist a valid failure record with the last completed stage.
- Sensitive sentinels never appear in progress, failure, success, or curve records.
- The adapter returns before the outer kill boundary under deterministic request-timeout tests.
- Required evidence survives the admitted primary budget.
- Overlap metrics are deterministic and fully covered duplicates are suppressed.
- Complete Rust A/B/A succeeds before other corrected live studies run.

## Revisit Triggers

- A public packet schema is required to represent multi-match merged evidence honestly.
- Relevance ordering becomes probabilistic or model-dependent.
- Compact high-effort requests remain too slow.
- Provider streaming cannot preserve response semantics.
- Multi-repository evaluation changes the admitted packet policy.
