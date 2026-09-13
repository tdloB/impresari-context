# Snapshot-Coherent Read Reuse — Architecture Requirements and Design

- ARD ID/version: IC-SCRR-ARD-135 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-04.
- Governing PRD: [IC-SCRR-135](../product/snapshot-coherent-read-reuse-prd.md).
- Decision:
  [ADR-0135](../decisions/0135-reuse-a-verified-read-within-one-request.md).

## Where the reads go

```text
one context build
  ├── step: literal "Header"      ─▶ scan every admitted file
  ├── step: literal "fromstring"  ─▶ scan every admitted file
  ├── step: literal "Card"        ─▶ scan every admitted file
  └── …one step per task signal   ─▶ scan every admitted file
```

`search_candidates` reads each candidate once per call, and the engine calls it
once per planned step. Reads therefore grow as *signals × files*: 1,869 tracked
files become 11,068 reads on `astropy-13453`.

The workspace ledger already says so. It counts a read as repeated when that
path has been read before, and on that task **every read is a repeat**.

## Why not the two obvious alternatives

**Batch the needles into one scan.** `search_candidates` already accepts a slice
of needles and tests all of them per file, so one call could replace six. But
`budget.max_matches` is shared across the needles in a call and the loop stops at
it, so a common term would consume the budget and starve the terms after it.
Evidence items also record only their span, not which needle produced them, so
per-step attribution would have to be reconstructed. Both are real changes to
what a consumer receives.

**Narrow the candidates using the identifier index.** The index knows which files
contain a given identifier, which would cut a 1,869-file scan to a handful. But
the index is token-based and literal search is substring-based: narrowing would
silently stop matching `Header` inside `SubHeader`. That is a semantic change,
and it should be argued on its own merits rather than smuggled in as an
optimisation.

Reuse changes neither. Every step sees the same bytes, the same matches and the
same budget it sees today — it just does not pay to read the same file twice.

## The safety property this must not lose

Today every read re-verifies its content hash against the snapshot artifact, so a
workspace changing mid-request is caught and the request fails with
`RefreshSnapshot`. A cache that simply returns stored bytes would blind that:
the first read would verify, and every later step would be served content that is
no longer on disk.

So a reused read is validated by a **cheap metadata probe** — the file's length
and modification time — before it is served. The probe is a `stat`, not a read,
and any discrepancy forces a real read and a fresh verification.

That is weaker than re-reading every time, and the weakening is stated plainly:
detection now has metadata granularity rather than content granularity, so a
change that preserves both length and timestamp within one request would go
unnoticed. It is accepted because the request already fixes its answer to one
snapshot, and because the alternative costs six passes over the repository on
every task.

## Bounds

| bound | why |
| --- | --- |
| retained bytes | reuse must not grow with the repository |
| request lifetime | content is dropped when the request that read it ends |

Reaching the byte bound stops further retention rather than evicting, so the
behaviour is deterministic: the same request retains the same files.

## Telemetry stays honest

The ledger counts reads actually performed. A served reuse is not a read and is
not counted as one, so the improvement is visible in the receipt rather than
hidden by accounting that pretends the read still happened.

This matters beyond tidiness: the consumer's budget is charged from these
counters, and `astropy-13453` fails today because the count exceeds the ceiling.

## Preserved invariants

`SEC-INV-002` holds: nothing is written. `SEC-INV-003` holds: repository content
remains data. `SEC-INV-011` holds and is the reason for the metadata probe —
content served as exact source is content that was hash-verified against the
snapshot, and a path whose verification has not happened is never served from
memory.

Retention is bounded and request-scoped, so no repository content outlives the
request that read it.
