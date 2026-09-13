# Changed-Line Coverage — Architecture Requirements and Design

- ARD ID/version: IC-CLC-ARD-156 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-12.
- Governing PRD: [IC-CLC-156](../product/changed-line-coverage-prd.md).
- Decision: [ADR-0156](../decisions/0156-score-changed-line-coverage-of-the-opening-evidence.md).

## Reading the reference change

`parse_reference_patch` walks the unified diff once. A `+++` header selects the
file (none for `/dev/null`), and a `@@` header resets the insertion anchor as
well as naming the enclosing symbol, as before. Inside a file:

```text
"-" line, non-empty after stripping  -> a changed line
"+" line after a context line        -> that context line is a changed line
" " line, non-empty after stripping  -> becomes the insertion anchor
"---" header                          -> ignored
```

Changed lines are kept per file as a set, so a line touched twice counts once.

## Reading the evidence

`load_delivered` decodes each observed evidence excerpt's `bytes_base64url`
with the pinned `base64` crate and keeps its text, lossily decoded as UTF-8,
per display path. A malformed excerpt is skipped, not fatal.

## Scoring

A changed line counts when it is a substring of any excerpt from the same file.
`CaseScore` gains `changed_lines` and `changed_lines_in_evidence`. `Report`
gains `total_changed_lines`, `total_changed_lines_in_evidence` and
`changed_line_coverage_percent`, which is floored like every other percentage.
`REPORT_SCHEMA_VERSION` becomes 1.2.

## Dependencies

`base64` moves from the evaluation crate's development dependencies to its
dependencies. It is already pinned and locked for the workspace, and
`scripts/generate-sbom.rb` lists locked packages without dependency kinds, so
`Cargo.lock` and `artifacts/sbom.spdx.json` do not change.

## Measurement

On the twenty-two-task astropy corpus the scorer reproduces the
out-of-repository script: 40/213 changed lines for `main` and 85/213 for the
ADR-0155 candidate. Every 1.1 total and per-case field matches the 1.1 report
over the same outputs. `Cargo.lock` is unchanged, and the SBOM check passes
with 192 packages.

## Verification

- `reference_patch_yields_removed_lines_and_insertion_anchors`.
- `changed_lines_count_only_in_evidence_from_the_same_file`, which also checks
  the totals and the percentage.
- `scorer_never_hands_reference_data_to_the_product` still holds.
