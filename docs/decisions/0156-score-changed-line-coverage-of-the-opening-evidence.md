# ADR-0156: Score Changed-Line Coverage of the Opening Evidence

- Status: Accepted
- Date: 2026-09-12
- Related PRD: [Changed-Line Coverage](../product/changed-line-coverage-prd.md)
- Architecture: [Changed-Line Coverage](../architecture/changed-line-coverage-ard.md)
- Follows: [ADR-0147](0147-count-a-target-file-in-map-recall.md)

## Context

The offline recall report (schema 1.1, ADR-0147) credits the opening evidence
with a reference file when any excerpt from that file is delivered, even one
that holds only the license header. It cannot say whether the evidence held the
code a fix changes.

On the twenty-two-task astropy corpus, `main`'s evidence reached 16 of 27
reference files but held 40 of the 213 lines the accepted fixes change. The two
counts can move apart. One evidence ranking cut both reference files (16 to 6)
and lines (40 to 13). Another kept files level (17) and doubled the lines (85).
Only a script outside the repository measured the lines.

## Decision

1. The recall report, now **schema 1.2**, counts per case the lines the
   reference change touches: each removed line, and for an insertion the
   context line just before it. It also counts how many of those appear,
   stripped of surrounding whitespace, inside an evidence excerpt delivered
   from the same file.
2. It reports both totals and a floored `changed_line_coverage_percent`. Every
   1.1 count is computed exactly as before.
3. The scorer stays offline. It makes no model call, opens no socket, and reads
   only the product's already-written output. To decode excerpts it links the
   workspace's pinned `base64` crate, which was already its test dependency, so
   the locked dependency graph and the SBOM are unchanged.

## Consequences

On the twenty-two-task astropy corpus the report reproduces the
out-of-repository script exactly: 40 of 213 changed lines for `main` (18%) and
85 of 213 for the ADR-0155 candidate (39%). Every 1.1 total and per-case count
is identical to the 1.1 report over the same outputs.

A changed line whose text also occurs elsewhere in the same file is credited by
an excerpt of that other place. Matching by position would need the file at the
base revision, which the scorer does not read. Text needs only the patch and the
output.

## Alternatives considered

**Keep the measure in a script outside the repository.** Rejected. Claims
about evidence would rest on an unversioned tool.

**Count added lines.** Rejected. An added line is not in the old file, so no
evidence can contain it. The line before an insertion is what places it.

**Replace evidence file recall with line coverage.** Rejected. File recall still
answers whether retrieval reached the file at all. The report keeps both.
