# ADR-0124: Classify Task Signals by Code Shape, Not Token Position

- Status: Accepted
- Date: 2026-09-03
- Related PRD: [Deterministic Structural-Seed Selection](../product/deterministic-structural-seed-selection-prd.md)
- Architecture: [Deterministic Structural-Seed Selection ARD](../architecture/deterministic-structural-seed-selection-ard.md)
- Refines: [ADR-0118](0118-select-structural-seeds-from-admitted-task-signals.md)

## Context

Task-signal extraction took the first sixteen raw tokens of a query and only
then classified them into paths and identifiers. It also admitted any token
containing `_`, `-`, or `::` as a code identifier.

Measured against a real bug report, that combination admits nothing usable. A
report opens with a title and an HTML comment block, so the sixteen-token
allowance is spent on prose before the first code signal appears, and the only
token classified as an identifier is `--`, the HTML comment delimiter.

Two further shapes were misclassified. `CamelCase` names carry no separator, so
`TimeSeries` — the subject of the report and the first word of its title — was
not a code identifier at all, which excludes most exported names in Python,
Java, C#, Go, and Swift. Conversely, `1.22.3` and `99.9` satisfied the
`stem.extension` path test, so version numbers competed with real file paths.

The observed consequence was `structural_seed_unavailable` on every such query,
which empties the disclosure map and silently degrades progressive structural
delivery to ordinary retrieval.

## Decision

Reach each ceiling by classified signals rather than raw token position,
scanning a bounded number of tokens. Admit an identifier on code shape:
a separator form, or interior capitalization, and require a leading letter or
underscore so markup runs are excluded. Require a letter in a path's final
component so versions and measurements are not paths.

## Consequences

Signals are recovered from realistic task text rather than only from text whose
first sixteen words happen to be code. The ceilings, determinism, and the rule
that prose cannot become graph authority are unchanged: this record widens what
counts as code shape and moves where the ceiling is applied; it does not admit
lexical or quoted terms as seeds.

Interior capitalization does not recover single-word lowercase names such as
`flux`, which remain indistinguishable from prose by shape alone. This record
grants no new authority and changes no disclosure, custody, or execution
boundary.
