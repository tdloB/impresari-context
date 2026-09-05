# ADR-0135: Reuse a Verified Read Within One Request

- Status: Accepted
- Date: 2026-09-04
- Related PRD: [Snapshot-Coherent Read Reuse](../product/snapshot-coherent-read-reuse-prd.md)
- Architecture: [Snapshot-Coherent Read Reuse](../architecture/snapshot-coherent-read-reuse-ard.md)
- Completes: [ADR-0129](0129-build-a-bounded-task-identifier-index-at-preparation.md)

## Context

A context build executes one retrieval step per task signal, and a literal step
scans every admitted file. Reads therefore grow as signals × files. Measured on
`astropy-13453`: 1,869 tracked files, **11,068 repository reads in one request**,
and the workspace ledger classifies every one of them as repeated.

The consumer's budget is charged from that count, so the request exceeds its
ceiling and returns nothing — the one task still delivering no map after
ADR-0134.

ADR-0129 fixed this shape once already, for nomination: searching per identifier
cost roughly 3,900 reads each, and a preparation-time index replaced it with one
pass. The same amplification remained in the evidence path, where it is charged
per request rather than once.

Reading a repository six times to answer one question is the failure the
governing objective names. It is addition, not substitution, and it is invisible
on every task that stays under the ceiling.

## Decision

Within one request, serve a repeated read of a path from the bytes already read
and verified for it, rather than reading the file again.

Validate a reuse with a cheap metadata probe — length and modification time —
before serving it, and fall back to a real read and a fresh verification on any
discrepancy. Never serve a path whose verification has not happened.

Bound retained bytes, stop retaining at the bound rather than evicting, and drop
everything when the request ends. Count only the reads actually performed.

Do not batch needles into a single scan, and do not narrow literal candidates
using the identifier index. Both change what a consumer receives — the first by
sharing a match budget across terms and losing per-step attribution, the second
by making literal search token-exact rather than substring — and this decision is
an efficiency, not a change of behaviour.

## Consequences

The same evidence is produced from a fraction of the reads. Every step sees the
same bytes, the same matches and the same budget as before, so recall must be
unchanged; the PRD requires measuring that it is, rather than assuming it.

Change detection weakens from content granularity to metadata granularity within
a request. A workspace edit that preserved both a file's length and its
modification time between two steps of one request would go unnoticed, where
today it would be caught. That is a real reduction in a safety property. It is
accepted because the request already fixes its answer to a single snapshot, and
because the alternative is six passes over the repository on every task — but it
is recorded here rather than buried, and a future decision may choose to pay
content-granularity verification on reuse if the cost proves acceptable.

Retention is bounded and request-scoped, so no repository content outlives the
request that read it, and a bounded retention means the cost does not grow with
the repository.

Counting only real reads makes the improvement visible in the receipt. That is
not cosmetic: the ceiling that fails `astropy-13453` is computed from these
counters.

No security invariant changes. `SEC-INV-011` is the reason for the metadata
probe: content served as exact source is content hash-verified against the
snapshot. This record grants no execution, network, publication, or submission
authority.
