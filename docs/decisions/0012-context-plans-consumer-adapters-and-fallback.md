# ADR-0012: Context plans, consumer adapters, and native-read fallback

- Status: Accepted for Slice C implementation
- Date: 2026-08-22
- Scope: Task-specific retrieval planning and reference-consumer integration

## Decision

The core accepts an ordered context plan of one to eight deterministic
retrieval steps. Every step uses an existing gateway-owned strategy and the
same hard resource budget. The engine deduplicates exact evidence by identity,
reports empty and limited steps explicitly, and constructs one immutable
packet. Adapters cannot contribute a second retrieval, authorization, evidence,
or packet implementation.

The AI App Builder OS reference adapter is a separately versioned translation
layer. It maps opaque consumer identity, role, purpose, time, plan, and budget
into the public engine contract. Its response adds no orchestration or approval
authority. An independent non-OS reference client calls the same public engine
method without depending on the OS-shaped adapter.

Native-read fallback is a consumer action, not an engine capability. The public
adapter contract may only say whether fallback may be considered. It never
performs or authorizes a read. Under `required`, fallback is always prohibited.
Under `preferred`, it may be considered only when the engine is unavailable or
the capability is explicitly unsupported. Integrity, policy, stale-state,
resource, and internal failures fail closed. The consumer must separately
authorize every permitted native read and preserve its own audit trail.

## Consequences

- One packet can combine complementary exact strategies without repository
  dumps or hidden model interpretation.
- OS and non-OS consumers prove the core is not coupled to one orchestrator.
- An engine integrity or authorization failure cannot be converted into an
  automatic, less-governed filesystem read.
- The first plan is deterministic evidence selection, not semantic/model
  compression. Model-assisted planning remains separately gated.
- Fallback availability may reduce disruption, but the consumer remains
  responsible for the security and completeness of its native-read path.

## Verification

- Multi-step evidence deduplication, explicit empty-step reporting, plan-count
  and elapsed-time limits, and packet integrity tests.
- Closed Draft 2020-12 request and fallback-decision contracts.
- Reference-client packet conformance without the OS adapter.
- Fallback matrix tests proving integrity, policy, stale, budget, and internal
  failures never permit consideration and no decision adds authority.

## Review trigger

Review before adding model-generated plans, automatic fallback execution,
cross-process sessions, consumer-specific authorization inside the core, or a
new contract major.
