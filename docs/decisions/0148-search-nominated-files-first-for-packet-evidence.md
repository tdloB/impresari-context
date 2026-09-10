# ADR-0148: Search Nominated Files First for Packet Evidence

- Status: Accepted
- Date: 2026-09-10
- Related PRD: [Nominated-File Evidence First](../product/nominated-evidence-first-prd.md)
- Architecture: [Nominated-File Evidence First](../architecture/nominated-evidence-first-ard.md)
- Extends: [ADR-0128](0128-extract-structure-for-nominated-files-not-whole-repositories.md)

## Context

A context packet's evidence is what its plan steps find: quoted literals, paths,
code identifiers and lexical terms, each searched against the snapshot. Each
search walked every file in snapshot order, which sorts encoded path units
rather than paths, and each response was cut to the packet budget by dropping
records from the end. The packet then admitted records in the order they were
found and dropped from the end again to fit.

So the evidence an agent received was whatever matched first in an order
unrelated to the task. `.github/`, `CHANGES.rst` and `cextern/` all sort before
`astropy/` in that order. On the twenty-two-task astropy corpus, 31 of the 43
evidence records `main` delivered came from changelog, continuous-integration,
vendored or documentation files, and the evidence named a file the accepted
change touched for 1 of 27 reference files. Each record carries an excerpt
window up to the requested excerpt size, 4 KiB in the evaluation, so most of
the evidence budget went to text unrelated to the task.

Nomination (ADR-0128) already chooses the files a task is about before the
packet is built. The evidence did not use them.

## Decision

When a task has nominated files, every literal and lexical plan step searches
each nominated file on its own, in nomination order, before it searches the
whole snapshot. Evidence found in nominated files is admitted first, one record
per file per pass in nomination order. Evidence from the whole-snapshot search
follows in its existing order. Caller-declared evidence still leads, and
structural evidence still trails.

Nothing else changes. Every search keeps its limits and its exact-source
verification, a match both searches find is delivered once, and a request
without a nomination builds the same packet as before. A nominated search that
reaches a limit is disclosed as `plan_step_N_nominated_limited`.

Retrieval gains `search_literal_in` and `search_lexical_in`, which search only
named snapshot paths and fail for a path the snapshot does not hold.

## Consequences

On the twenty-two-task astropy corpus:

| | `main` | this decision |
| --- | --- | --- |
| evidence names a reference file | 1/27 | 14/27 |
| evidence records from nominated files | 7 of 43 | 40 of 42 |
| records from changelog, CI, vendored or docs files | 31 | 4 |
| packet bytes, summed over 22 packets | 289,184 | 303,307 |
| nomination recall, map file recall, map items | 22/27, 21/27, 1,475 | unchanged |

Twelve tasks gain a reference file in their evidence and none loses one. A
packet uses about 640 more bytes of its budget on average, and every packet
stays within the 16 KiB it requested.

A packet can run one extra single-file search per nominated file for each
literal or lexical step: at most sixteen files by eight steps. Latency was not
measured on the corpus.

Evidence remains bounded by how many records fit in a packet, which at 4 KiB
excerpts is two. A reference file nominated fourth or lower, or one no planned
search matches, still does not appear.

## Alternatives considered

**One search over all nominated files.** Rejected; measured at 12 of 27. A
search response is cut to the budget from the end of a list in snapshot order,
so the nominated file that sorts first filled it and hid the rest. On
`astropy-12907`, `separable.py` was nominated first and holds
`separability_matrix` seven times, and it never reached the packet.

**Re-rank the whole-snapshot results by nomination.** Rejected: a common
literal reaches its match limit inside files that sort early, so the nominated
files' matches are often never collected to be re-ranked.

**Drop evidence from files that were not nominated.** Rejected: nomination is
bounded and can miss the file a task needs, and the whole-snapshot search is
the fallback when nominated files match nothing.

**Exclude changelogs, CI configuration and vendored code by path.** Rejected: a
hand-written path list is specific to one repository, and the nomination
already says what the task is about without one.
