# Provider-Free Product Read Observer PRD

- PRD ID/version: IC-PFPRO-116 / 1.0.
- Status: Implemented provider-free; local cross-repository compatibility passed.
- Date: 2026-09-01.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Provider-Free Product Read Observer ARD](../architecture/provider-free-product-read-observer-ard.md).
- Governing decision:
  [ADR-0116](../decisions/0116-observe-repository-reads-at-the-workspace-boundary.md).
- External consumer contract: `repository-context-eval` protocol v3.1 and
  `impresari_context_repository_read_telemetry` 1.0.

## Problem

The independent evaluator can count its own repository reads, but it cannot
currently observe the reads Impresari performs while opening, snapshotting,
planning, and assembling a packet. Treating one outer `context_build` call as
one repository read materially understates product work and prevents a causal
comparison with a cold agent.

The observer must be product-owned and provider-free. A model, adapter, packet,
or caller-supplied number is not authoritative read evidence.

## Outcome

Every successful MCP `context_build` result includes a complete, source-bound
read attestation when the fresh process can account for every eligible source
read. The attestation is measured at the capability-relative workspace read
boundary and covers startup snapshot construction as well as packet planning
and evidence assembly.

## Requirements

1. Instrument `AuthorizedWorkspace::read_exact`; do not infer reads from packet
   evidence, planner steps, tool calls, cache entries, or model output.
2. Count every invocation that actually reads file bytes, even when a later
   size, mutation, or validation check rejects those bytes.
3. Count exact bytes materialized at that boundary and use checked arithmetic.
   Overflow or observer failure must make the attestation incomplete.
4. Count a repeated read whenever the same lossless relative path is read more
   than once during the fresh MCP process lifecycle.
5. Include startup snapshot reads. The evaluator launches one fresh MCP process
   and performs one `context_build`; the reported counters are cumulative for
   that bounded lifecycle.
6. Derive the source fingerprint from exact snapshot bytes and portable
   repository-relative paths using sorted `path NUL bytes NUL` SHA-256, matching
   the evaluator's isolated-source contract, including its canonical
   `sha256:` prefix.
7. Set `complete=true` only when the read ledger is healthy, the snapshot is
   complete, every observed source path is portable and canonically ordered,
   and the snapshot contains no skipped object.
8. Emit exactly schema
   `impresari_context_repository_read_telemetry` version `1.0` with:
   `source_fingerprint_sha256`, `repository_file_reads`,
   `repeated_repository_file_reads`, `source_bytes_read`, and `complete`.
9. Keep the observer read-only, process-local, non-persistent, bounded, and free
   of ambient paths, source text, prompts, answers, network, provider, execution,
   publication, or benchmark-submission authority.
10. Preserve packet bytes, planner semantics, policy decisions, source
    immutability, and existing CLI/MCP authority flags.

## Acceptance

- Unit tests prove exact counts, repeated-read counts, byte counts, and a
  harness-compatible fingerprint over multiple nested paths.
- Negative tests prove skipped/incomplete discovery cannot claim complete
  telemetry and rejected pre-read objects do not invent bytes.
- MCP tests prove `context_build` exposes the exact closed schema, matching
  source identity, and no added authority.
- A provider-free cross-repository compatibility test accepts a real Impresari
  MCP response through the independent evaluator.
- Formatting, warnings-denied Clippy, all-target tests, docs, and hosted CI pass.

## Non-Goals

- Running OpenAI, Anthropic, or any other model provider.
- Running or publishing SWE-bench results.
- Claiming token, cost, latency, correctness, or read reduction.
- Selecting evidence, changing the deterministic planner, or implementing a
  Graft-style semantic graph or LeanCTX-style progressive delivery.
- Persisting product telemetry or adding remote telemetry.

## Stop Conditions

Paid evaluation remains prohibited until this observer and the independent
provider-free utility fixtures both pass against the exact product binary. A
telemetry pass authorizes measurement only; it does not establish usefulness.
