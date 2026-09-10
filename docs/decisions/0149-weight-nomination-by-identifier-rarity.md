# ADR-0149: Weight Nomination by Identifier Rarity and Resolve Dotted Module Names

- Status: Accepted
- Date: 2026-09-10
- Related PRD: [Rarity-Weighted Nomination](../product/rarity-weighted-nomination-prd.md)
- Architecture: [Rarity-Weighted Nomination](../architecture/rarity-weighted-nomination-ard.md)
- Refines: [ADR-0131](0131-admit-and-rank-identifiers-by-declaration.md)
- Extends: [ADR-0128](0128-extract-structure-for-nominated-files-not-whole-repositories.md)

## Context

ADR-0131 ranks inferred candidates as mentions plus three times declarations,
with every task identifier counting the same. On a large repository the
identifiers a task names are not equally informative: a name held by one file
identifies that file, while a name held by thirty identifies none of them.

On `astropy-13398` the task names `ITRS`, declared by exactly one file,
alongside names such as `frame_transform_graph` and `matrix_utilities`, held by
thirty and twenty files. Counted equally, the file declaring `ITRS` scored three
and placed eighteenth while the sixteenth nominated file scored four, so it was
never nominated.

Separately, task text names modules the way code imports them. `astropy-14182`
writes `format="ascii.rst"`. The planner classifies `ascii.rst` as a path, the
snapshot holds no such path, and nomination dropped it, so the file it denotes,
`astropy/io/ascii/rst.py`, was never nominated.

## Decision

**Weight each task identifier by its rarity.** An identifier's weight is the
number of times the repository's file count halves before it reaches the files
holding the name, plus one: `⌊log₂(files ÷ holders)⌋ + 1`, where holders is the
larger of the files mentioning and the files declaring it. A file's score is
three times the weight of each identifier it declares plus the weight of each
it mentions. A declaration still counts three mentions of the same name, and
ties still break by path. The logarithm is an integer, so the ranking is
identical on every platform.

**Resolve a dotted module name.** A task path the snapshot does not hold, that
contains a dot and no slash, is read as a path suffix with the file extension
removed. It nominates the one tracked file it denotes, ranked with the paths a
task writes out, under the new reason `task_module_path`. It nominates nothing
when the name is also a directory, which denotes a package, or when several
paths end in it.

The nomination disclosure schema moves from 1.0 to 1.1 for the new reason code
and the changed ranking.

## Consequences

On the twenty-two-task astropy corpus, measured with this change alone:

| | `main` | this decision |
| --- | --- | --- |
| nomination recall | 22/27 | 24/27 |
| map file recall | 21/27 | 22/27 |
| map items | 1,475 | 1,440 |
| evidence names a reference file | 1/27 | 1/27 |

Rarity nominates `itrs.py` for `astropy-13398`; module resolution nominates and
maps `rst.py` for `astropy-14182`. No task loses a reference file. Packet
evidence is unchanged on its own, because it does not consult nomination
without ADR-0148; with ADR-0148 as well, the evidence names 16 of 27 reference
files, against 14 with ADR-0148 alone.

Before the engine change, both rules were replayed offline against the
nomination inputs the engine itself recorded, a replay that reproduced the
engine's nomination on all twenty-two tasks. Rarity alone and module resolution
alone each reached 23 of 27; together 24.

Three reference files remain out of reach of nomination: one is created by the
accepted change, one is a generated parser table, and one is a package
`__init__.py` that registers a new module. The sixteen-identifier task-signal
limit also fills in text order, so a noisy issue can spend slots on names such
as `TypeError` before reaching a rare one; that is a separate follow-up.

## Alternatives considered

**Natural-log inverse document frequency.** It reached the same recall in the
replay, but a floating-point logarithm can differ in its last bit across
platforms and reorder a tie. The integer form keeps nomination reproducible.

**Tune the declaration weight or the weight curve to the corpus.** Rejected:
the corpus is small and is where the problem was found, so a tuned constant
would fit it rather than the product. The rule keeps ADR-0131's factor of three
and adds no new constant.

**Raise `MAX_NOMINATED_FILES`.** Rejected: ADR-0128 measured the curve flat
beyond sixteen while graph cost keeps rising, and a larger ceiling admits the
same common-name files ahead of the rare one.

**Resolve a dotted name to a package's `__init__` file.** Rejected: a package
holds many files and its `__init__` is rarely the one a task needs. Without the
directory rule, `io.registry` resolved to `docs/io/registry.rst` while the
package `astropy/io/registry` exists.
