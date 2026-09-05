# Snapshot-Coherent Read Reuse PRD

## Document Control

- PRD ID/version: IC-SCRR-135 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-04.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Snapshot-Coherent Read Reuse ARD](../architecture/snapshot-coherent-read-reuse-ard.md).
- Governing decision:
  [ADR-0135](../decisions/0135-reuse-a-verified-read-within-one-request.md).
- Governing objective: [CLAUDE.md](../../CLAUDE.md) — 98% quality at 78% compression.

## Problem

Answering one task reads the repository several times over.

A context build executes one planned retrieval step per task signal, and a
literal step scans every admitted file. Measured on `astropy-13453`: **1,869
tracked files, 11,068 repository reads in a single request** — roughly six passes
over the same repository. The workspace's own ledger classifies **every one of
those reads as repeated**.

The cost is charged to the consumer's budget, and when it exceeds that budget the
request returns nothing: `astropy-13453` builds an 86-item map and discards it,
which is the one task still delivering no map after
[ADR-0134](../decisions/0134-truncate-a-disclosure-at-its-ceiling-rather-than-discarding-it.md).

This is the failure the governing objective names. A product that reads a
repository six times to answer one question is adding work, not substituting for
it, and the overage is invisible on every task that happens to stay under the
ceiling.

## Product Outcome

A file read and verified once during a request is not read again during that
request. The same evidence is produced from a fraction of the reads.

## Functional Requirements

1. Within one request, serve a repeated read of the same path from the bytes
   already read and verified, rather than reading the file again.
2. Preserve change detection. A reused read must still notice a workspace that
   changed underneath it, to at least the granularity a cheap metadata probe
   gives, and re-read when it may have.
3. Never serve content that failed verification, and never serve content for a
   path whose verification has not happened.
4. Bound retained bytes explicitly. Reuse stops at the bound rather than growing
   with the repository, and reaching it is recorded.
5. Report reads honestly. Telemetry counts the reads actually performed, so the
   improvement is visible rather than hidden by the accounting.
6. Produce identical evidence. Reuse is an efficiency, not a behaviour change:
   no step sees different bytes, different matches, or a different budget than
   it sees today.
7. Never retain content beyond the request that read it.

## Acceptance Criteria

- Repeated reads of one path within a request perform one file read.
- A file changed between two reads within a request is detected and re-read.
- Retained bytes stay under the bound; reaching it is disclosed.
- **Measured, offline, over all twenty-two astropy tasks:** repository reads per
  request fall substantially, map file recall and symbol recall are unchanged,
  and delivered bytes are unchanged.
- `astropy-13453` returns a map.
- The full repository gate passes.

## Non-Goals

- Reducing the number of planned steps. Fewer signals would mean less evidence;
  this changes cost, not selection.
- Caching across requests. A later request re-verifies, because a workspace may
  change between requests.
- Narrowing a literal search to indexed candidates. That would make literal
  search token-exact rather than substring, losing matches, and is a semantic
  change this PRD does not make.
- Batching several needles into one scan. The shared match budget would starve
  later needles and the results carry no needle attribution.
