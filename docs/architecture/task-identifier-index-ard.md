# Task Identifier Index — Architecture Requirements and Design

- ARD ID/version: IC-TII-ARD-129 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-03.
- Governing PRD: [IC-TII-129](../product/task-identifier-index-prd.md).
- Decision:
  [ADR-0129](../decisions/0129-build-a-bounded-task-identifier-index-at-preparation.md).

## Where the cost goes

```text
preparation (once)                    request (per task)
──────────────────                    ──────────────────
read each admitted file               task signals
        │                                   │
        ▼                                   ▼
 admit code-shaped identifiers        index lookup   ← no repository read
        │                                   │
        ▼                                   ▼
 snapshot-bound forward index ─────▶  nominated files
                                            │
                                            ▼
                                      dense scoped graph
```

The reads move from the request to preparation, and from *per identifier* to
*once per file*. That is the whole design. A scan-per-identifier costs about
3,900 reads each; the index costs one pass and then nothing.

## Forward, not inverted

The index maps **file → identifiers**, and a lookup scans those sets.

An inverted index would answer a single lookup faster, but nomination asks about
every admitted identifier at once and then ranks files by how many they match —
which is a per-file question. The forward shape produces that count directly,
keeps one entry per file so the bounds are obvious, and needs no posting-list
merge. On a repository the size of astropy the scan is over roughly 1,800
in-memory sets, which is immaterial beside the structural extraction that
follows.

## One rule, one definition

Index terms are admitted by exactly the function task signals use. This is not a
convenience; it is a correctness property. Two independent definitions of "looks
like code" would drift, and the failure would be silent: a task naming
`_required_columns` would match nothing because the index had quietly decided
that shape was not interesting.

The shared rule means the index holds a term precisely when a task could name
it.

## Bounds

Three explicit ceilings, each recorded when reached:

| bound | why |
| --- | --- |
| indexed files | preparation stays proportional to the admitted snapshot |
| identifiers retained per file | a generated or vendored file cannot dominate |
| identifier length | a pathological token cannot inflate an entry |

A breach is an unknown, never a silent truncation. The pattern follows
[ADR-0121](0121-use-bounded-progressive-structural-disclosure.md): a limit that
bites must be visible.

## Snapshot binding

The index carries the snapshot identity that produced it and refuses a lookup
for any other. A stale index is the same class of defect as a stale graph, and
gets the same treatment: explicit refusal rather than a plausible wrong answer.

## What the index deliberately does not hold

Identifiers and portable paths. No source bytes, no spans, no excerpts, no
line numbers. It answers membership, not evidence, so it cannot become a second
retrieval path that bypasses the exact-source and provenance rules in
`SEC-INV-011`.

## Preserved invariants

`SEC-INV-002`, `SEC-INV-003`, `SEC-INV-007`, `SEC-INV-008`, `SEC-INV-009`, and
`SEC-INV-011` hold unchanged. The index reads the workspace during preparation
exactly as structural extraction already does, writes nothing, executes nothing,
and retains no source content.
