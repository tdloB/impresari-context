# Progressive Structural Disclosure PRD

- PRD ID/version: IC-PSD-121 / 1.0.
- Status: Approved for implementation after ADR-0120 passes its independent
  provider-free MCP comparison.
- Date: 2026-09-01.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Progressive Structural Disclosure ARD](../architecture/progressive-structural-disclosure-ard.md).
- Governing decision:
  [ADR-0121](../decisions/0121-use-bounded-progressive-structural-disclosure.md).

## Problem

The trusted structural lifecycle makes graph preparation, cache lineage,
selection, reads, and latency measurable, but its first exact provider-free
MCP comparison showed that eager structural delivery can cost more context than
the ordinary packet. On the controlled fixture, the initial treatment response
was 8,063 bytes versus 4,149 bytes for the ordinary response even though warm
graph preparation fell from 645 milliseconds to 6 milliseconds. The graph and
cache are useful foundations; unconditional exact-source serialization is the
remaining delivery bottleneck.

## Outcome

An explicitly enabled local MCP process returns a small deterministic
structural map from the existing `context_build` request, then lets its owning
process-local session resolve compact graph items and expand exact source only
when requested. Every disclosure remains authorized, snapshot bound,
content-addressed, cumulative-budgeted, source verified, and auditable.

The ordinary and eager modes remain available. All modes advertise the same
MCP tools and accept the same closed inputs. Delivery mode is trusted process
configuration, never repository content or tool input.

## Requirements

1. Add one trusted startup delivery mode with exactly three values:
   `ordinary`, `eager_structural`, or `progressive_structural`. Structural modes
   require the complete ADR-0120 worker tuple. The default remains `ordinary`.
2. Keep the `context_build` input schema equal across modes. Progressive mode
   requires an already-open process-local session and returns a closed
   disclosure map instead of serializing the full structural packet.
3. A map item may contain only a content-addressed handle, authorized relative
   display path, structural fact/relationship class, bounded symbol label,
   confidence, freshness, and explicit unknown/truncation state. It contains no
   source excerpt, generated summary, prompt, answer, command, or authority.
4. Derive map and item identities from the current workspace/snapshot, graph,
   task-plan identity, selection policy, admitted budget, and canonical item
   content. Session ownership gates resolution but is not an identity input.
5. Add always-advertised `context_disclosure_lookup` and
   `context_evidence_expand` tools. Lookup accepts only a map/item handle from
   the owning session and bounded relation controls. Expansion accepts only an
   evidence handle from the owning session plus bounded before/after/excerpt
   byte requests.
6. Resolve handles only against the same consumer, process-local session,
   workspace, snapshot, graph, content hash, path identity, and policy. Reject
   stale, foreign, forged, closed-session, expired, malformed, or unavailable
   handles without substituting evidence.
7. Reuse the existing exact evidence-recovery implementation for source
   expansion. No progressive component may read source directly, invent spans,
   or downgrade graph facts into exact evidence.
8. Enforce one cumulative disclosure ledger per session covering map count,
   lookup count, expansion count, returned items, exact source bytes,
   serialized response bytes, repository reads, repeated reads, and elapsed
   milliseconds. The first over-limit operation fails before new source bytes
   are disclosed.
9. Return a closed receipt after every map, lookup, and expansion with mode,
   disclosure/session identities, operation, deterministic result identity,
   per-call consumption, cumulative consumption, remaining ceilings,
   truncation/exhaustion state, graph/snapshot identities, and read deltas.
10. Never hide exhaustion, unsupported structure, partial graphs, unresolved
    edges, stale state, or omitted candidates as an empty successful result.
11. Preserve source immutability, stdout purity, no-network operation, worker
    isolation, cache integrity, and repository-text-as-data boundaries.
12. Add closed schemas, unit/property/negative tests, MCP parity tests,
    cold/warm cache tests, cumulative-boundary tests, source-mutation tests,
    cross-session/workspace tests, and a frozen provider-free
    ordinary/eager/progressive mechanics gate.
13. Perform no provider call, official grading, benchmark submission,
    publication, or product-effect claim in this increment.

## Acceptance

- All three modes advertise byte-identical tool definitions.
- Equal source, task, policy, budget, worker, mode, and cache state produce the
  same map, item, evidence, and receipt identities apart from explicit request,
  event, session, and measured-time fields.
- On every frozen fixture, progressive initial delivered bytes are lower than
  eager structural delivered bytes, preserve every admitted initial anchor,
  and remain within a frozen delta from the ordinary response.
- Expanding every progressive handle reproduces byte-identical exact evidence
  to eager delivery or reports a previously explicit unknown/omission.
- Cumulative accounting equals independently observed response bytes and
  product-owned read telemetry; overflow is rejected before the first excess
  byte or read is returned.
- Wrong consumer/session/workspace/snapshot/graph/content, closed sessions,
  changed source, forged handles, and replay after process exit all fail closed.
- Existing clients that omit delivery mode retain current behavior.
- Formatting, warnings-denied Clippy, all-target tests, schemas, docs,
  repository policy, security boundaries, SBOM, and hosted macOS/Linux/Windows
  checks pass before merge.

## Evaluation Gate

The independent `repository-context-eval` project must compare three fresh
product processes on the same frozen tasks before any paid run:

1. ordinary delivery;
2. eager structural delivery;
3. progressive structural delivery with a deterministic scripted expansion
   policy.

The mechanics report must separate graph preparation, initial response,
lookup/expansion responses, cumulative delivered bytes, reads, repeated reads,
elapsed time, omissions, and source integrity. A scripted expansion policy is
mechanics evidence only and must not be presented as model behavior.

## Non-Goals

- LLM-written summaries, embeddings, semantic vector search, or provider calls.
- Durable or cross-process memory, conversation replay, prompt caching, or
  hidden retained tool state.
- MCP proxying, request rewriting, server-initiated sampling, elicitation,
  roots, network transport, or remote service operation.
- Background filesystem watching, automatic graph refresh, or source mutation.
- Changing canonical graph facts, exact evidence authority, native repository
  tools, or official SWE-bench grading.
- Claiming token, cost, latency, or correctness improvement from the
  provider-free gate.

## Stop Condition

Do not run another paid OpenAI or Anthropic comparison until the progressive
provider-free gate passes locally and in hosted CI, the independent adapter
proves tool/schema parity and cumulative accounting, and a separate pilot
manifest freezes model, tools, limits, repositories, tasks, repetitions,
pricing, and official grading.
