# ADR-0129: Build a Bounded Task Identifier Index at Preparation

- Status: Accepted
- Date: 2026-09-03
- Related PRD: [Task Identifier Index](../product/task-identifier-index-prd.md)
- Architecture: [Task Identifier Index](../architecture/task-identifier-index-ard.md)
- Completes: [ADR-0128](0128-extract-structure-for-nominated-files-not-whole-repositories.md)

## Context

ADR-0128 chose to nominate candidate files from the task and extract structure
densely over only those. Its architecture assumed a caller could supply the
identifier-to-file mapping "from whatever index it already holds." No such index
exists. Building the wiring is what surfaced the omission.

Answering the question by searching per identifier costs roughly 3,900
repository reads each. Nine identifiers is 36,286 reads against a 10,000
ceiling, so every task exhausted its budget and returned nothing at all.

Two cheaper variants were measured and neither rescues it. The promoted lexical
index returns no result on this path. Reducing identifier count preserves
nomination recall exactly — six identifiers recall 16 of 27 reference files,
identical to sixteen — but still costs roughly 19,700 reads.

The deeper problem is where the cost is charged. Nomination is a planning step.
Billing it to the request's context read budget makes the product add work
rather than replace it, which is precisely the failure the governing objective
names.

## Decision

Build a bounded forward index during preparation, mapping each admitted file to
the code-shaped identifiers it contains, and answer nomination lookups from
memory with no repository read.

Admit index terms using exactly the function task signals use, so the two sides
cannot drift. Bind the index to its snapshot and refuse to answer for another.
Bound indexed files, identifiers per file, and identifier length, recording any
breach as an explicit unknown. Retain identifiers and portable paths only — no
source bytes, spans, or excerpts.

## Consequences

Nomination becomes affordable, which is what unblocks ADR-0128 and the path from
0 of 27 measured map recall toward the measured nomination ceiling of 16 of 27.

Preparation gains one pass over admitted files. That cost is paid once and is
the same read the structural pass already performs, rather than a cost repeated
per identifier per request.

The shared admission rule is a hard coupling and is meant to be. Two definitions
of "looks like code" would drift silently: a task naming a symbol would match
nothing because the index had decided that shape was uninteresting. One
definition makes the failure impossible.

Holding only identifiers and paths keeps the index from becoming a second
retrieval path that bypasses exact-source provenance. It answers membership, not
evidence.

No security invariant changes. This record grants no execution, network,
publication, or submission authority.
