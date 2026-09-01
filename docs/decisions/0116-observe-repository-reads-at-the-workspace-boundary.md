# ADR-0116: Observe Repository Reads At The Workspace Boundary

- Status: Implemented provider-free
- Date: 2026-09-01
- Decider: Aaron Boldt through the active evaluation roadmap continuation
- Related PRD: [Provider-Free Product Read Observer PRD](../product/provider-free-product-read-observer-prd.md)
- Architecture: [Provider-Free Product Read Observer ARD](../architecture/provider-free-product-read-observer-ard.md)

## Context

The independent evaluator requires product-owned evidence of every repository
read used to construct treatment context. Impresari currently exposes one MCP
`context_build` call, but that call can perform many startup, discovery,
planning, and exact-evidence reads. Inferring internal reads from the outer tool
call or returned evidence would undercount work and reproduce the invalid
measurement boundary that delayed the correctness pilot.

## Decision

Instrument the existing capability-relative `AuthorizedWorkspace::read_exact`
boundary with a process-local cumulative ledger. Bind those counters to a
harness-compatible fingerprint computed from the same exact bytes admitted by
the current snapshot. Project the closed telemetry schema through `LocalEngine`
and include it in every successful MCP `context_build` result.

For the admitted evaluator lifecycle, one fresh MCP process performs one
startup snapshot and one context build. The attestation includes both. It is
complete only when all relevant source objects and counters are representable
without omission or observer failure.

## Consequences

- Repository reads become measured facts rather than product or model claims.
- Snapshot construction is no longer hidden outside `context_build`
  accounting.
- The first real evaluator compatibility check can remain provider-free.
- Ordinary context packets and authority boundaries remain unchanged.
- Long-lived clients see process-to-date counters; a future per-operation or
  amortized accounting contract would require a new versioned decision.

## Alternatives

- Count MCP tool calls: rejected because one context tool performs nested reads.
- Count returned evidence items: rejected because discovery and rejected
  candidates also consume repository I/O.
- Let the evaluator trace product filesystem syscalls: rejected as
  platform-specific, difficult to bind to product semantics, and unnecessary
  when a single capability boundary already exists.
- Add counters only to planner code: rejected because startup snapshot and
  other retrieval paths would remain invisible.
- Resume paid testing without telemetry: rejected because the resulting
  efficiency comparison would not be causally interpretable.

## Revisit triggers

Revisit before adding concurrent workspace reads, resetting counters,
persisting telemetry, reporting per-call deltas, amortizing snapshot work,
changing the fingerprint algorithm, bypassing `read_exact`, adding remote
telemetry, or using the observer to authorize a provider run or performance
claim.
