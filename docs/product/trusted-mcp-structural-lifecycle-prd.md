# Trusted MCP Structural Lifecycle PRD

- PRD ID/version: IC-TMSL-120 / 1.0.
- Status: Approved for implementation after the provider-free structural utility gate passes hosted CI.
- Date: 2026-09-01.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Trusted MCP Structural Lifecycle ARD](../architecture/trusted-mcp-structural-lifecycle-ard.md).
- Governing decision:
  [ADR-0120](../decisions/0120-bind-structural-runtime-to-trusted-mcp-startup.md).

## Problem

The core engine can build a deterministic structural graph and select a task
seed, but the normal local MCP lifecycle builds only a source snapshot. The
current structural MCP form accepts a caller-supplied graph and start node,
which is unsuitable for a controlled baseline/treatment comparison: it lets an
external adapter choose evidence, excludes graph-build work from product
telemetry, and cannot prove which worker binary produced a reused parser
result.

## Outcome

An explicitly enabled local MCP process prepares one current structural graph
from trusted startup configuration before serving requests. Ordinary and
structural processes expose the same tools and accept the same `context_build`
request. When structural preparation is enabled, the server applies the
product-owned task-signal selector; when it is absent, existing ordinary
profile behavior remains unchanged.

## Requirements

1. Structural mode is enabled only by an all-or-none trusted startup tuple:
   absolute worker path, exact worker SHA-256, and existing empty non-workspace
   working directory. MCP request data cannot set or replace this tuple.
2. The default startup contract remains non-structural and backwards
   compatible. Partial, malformed, relative, symlinked, mismatched, or
   workspace-contained structural configuration fails before MCP readiness.
3. Build the complete source snapshot first, then build a graph through the
   existing capability-reduced worker launcher. Parser cache keys must include
   the exact worker binary digest in addition to source and parser-contract
   identities.
4. Bind the prepared graph to the startup snapshot and retain it only in the
   process. No graph, source text, prompt, answer, or task is persisted beyond
   the existing exact cache and audit contracts.
5. For ordinary `profile + query` context builds, use the prepared graph and
   `StructuralSeedRequest` automatically when available. Do not add a new
   treatment-only MCP tool or require a caller-provided node.
6. Emit a closed structural-lifecycle receipt beside the packet containing
   enabled state, graph and snapshot identities, worker digest, preparation
   elapsed milliseconds, and explicit success/source state. It adds no
   authority and is excluded from packet identity.
7. Product-owned cumulative read telemetry must include snapshot discovery,
   graph preparation, exact evidence recovery, and packet construction for
   the fresh MCP process. Adapter timing must include process launch through
   response receipt.
8. Any graph build, digest, cache-integrity, snapshot-freshness, seed, or
   source-verification failure fails closed. There is no silent ordinary-mode
   fallback after structural mode was requested.
9. Require deterministic packets and graph identities for equal source,
   task, policy, budget, worker digest, and cache state. Timing is measured but
   is not a deterministic identity input.
10. Add unit, MCP equivalence, negative/security, cold-cache, warm-parser-cache,
    cross-platform, source-immutability, and evaluator-mechanics coverage.
11. Perform no provider call, official grading, publication, benchmark
    submission, or product-effect claim in this increment.

## Acceptance

- Baseline and treatment use byte-identical `context_build` JSON apart from
  request/event identities required to prevent replay; only trusted process
  startup differs.
- A fresh structural cache accounts for graph reads and worker execution; a
  warm parser cache remains worker-digest-bound and reports its state.
- The structural receipt, packet, plan, and complete read telemetry are
  available through the existing MCP response.
- Existing no-worker MCP clients retain their current tool list and behavior.
- Formatting, warnings-denied Clippy, all-target tests, docs, repository
  policy, security boundaries, and hosted CI pass on macOS, Linux, and Windows.

## Non-Goals

- Model calls, token/cost claims, correctness grading, or SWE-bench execution.
- Client hooks, background daemons, automatic source mutation, or remote MCP.
- LLM-written summaries, embeddings, or Graft-compatible files.
- LeanCTX-style progressive map/signature/line delivery; that remains the next
  independent decision after static structural delivery is measurable.
- Durable session memory or cross-chat behavior.

## Stop Condition

Do not expose structural treatment to the independent evaluator until the
provider-free utility gate and this lifecycle pass local and hosted validation.
Do not run paid evaluation until the evaluator verifies identical request/tool
contracts and complete cold-lifecycle accounting provider-free.
