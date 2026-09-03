# ADR-0125: Select Ranked Seed Sets and Traverse to Definitions

- Status: Accepted
- Date: 2026-09-03
- Related PRD: [Task-Recall-First Context Selection](../product/task-recall-first-context-selection-prd.md)
- Architecture: [Task-Recall-First Context Selection](../architecture/task-recall-first-context-selection-ard.md)
- Refines: [ADR-0118](0118-select-structural-seeds-from-admitted-task-signals.md), [ADR-0124](0124-classify-task-signals-by-code-shape-not-token-position.md)

## Context

Structural selection returned sixteen well-attributed items for
`astropy__astropy-13033`, all in `astropy/timeseries/sampled.py`. The change the
task requires is entirely in `astropy/timeseries/core.py`, in
`BaseTimeSeries._check_required_columns`. Recall on the target file was zero.
The evidence packet did contain `core.py`, so retrieval worked and selection did
not.

ADR-0118 chose exactly one seed and treated a tie as no seed. That was a
defensible first increment: one anchor is easy to reason about, and refusing to
guess between candidates avoids inventing authority. Measurement shows both
halves are too strict.

One anchor cannot describe a task whose answer spans a subclass and the parent
it inherits from. And refusing on ambiguity discards the most common useful
signal — that a name occurs in several related places — while returning nothing
at all, which is strictly worse than returning a ranked list with the ambiguity
disclosed.

Separately, the repository has no way to know any of this. Nothing scores
whether delivered context contains what a task needed, so selection quality has
been invisible.

## Decision

Return a bounded, ranked **set** of structural seeds. Rank by exact symbol in an
exact task path, then exact task file path, then globally unique exact symbol,
then globally ambiguous exact symbol, with deterministic tie-breaks. Retain a
bounded slice of an ambiguous class and disclose the ambiguity, rather than
yielding nothing. Yield no seed only when nothing matches.

Extend bounded traversal so a reference reaches the declaration it names and a
declaration reaches the supertypes it extends.

Add an offline scoring tool that measures task-relative recall against reference
changes, driving the product as a black box. The engine has no path to reference
data, enforced by a static check.

The seed maximum is a closed constant, never caller-supplied.

## Consequences

Maps become wider and more likely to contain the target. That costs bytes, which
the governing objective explicitly permits: 78% compression is the target, not
99%, and the difference is budget to spend on being right.

Selection quality becomes measurable at zero cost, on every commit, without a
provider credential — so recall regressions surface like test failures instead
of like benchmark surprises.

A caller-supplied seed maximum is rejected because a caller able to widen
selection could steer it, and steering is oracle authority.

No security invariant changes. `SEC-INV-003`, `SEC-INV-007`, `SEC-INV-009`, and
`SEC-INV-011` hold unchanged; the seed set is larger, the authority is not. This
record grants no execution, network, publication, or submission authority.
