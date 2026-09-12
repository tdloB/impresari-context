# ADR-0154: Look Through a Function's Local Variables When Building a Map

- Status: Accepted
- Date: 2026-09-11
- Related PRD: [Look Through Local Variables](../product/look-through-local-variables-prd.md)
- Architecture: [Look Through Local Variables](../architecture/look-through-local-variables-ard.md)
- Follows: [ADR-0153](0153-record-what-a-declaration-is-and-deliver-a-seeds-family.md)

## Context

The graph records a function's local variables as declarations and attributes a
call made in an assignment to the local, not the function. Traced through the
engine's own graph on the twenty-two-task astropy corpus:

- 28% of seeds were function locals: task words that happened to name one, such
  as `table`, `array` and `value`.
- 22% of the items `main`'s map delivered named a function local.
- 53% of a function seed's resolved calls and references sat one hop behind a
  local, out of reach of a one-hop traversal.

ADR-0153 made each declaration record what it is, which makes a local variable
identifiable.

## Decision

1. A variable is **function-local** when the nearest declaration enclosing it
   that is not itself a variable is a function. Class attributes and
   module-level names are not: they are part of a type's or a module's shape.
2. On the map path, **no seed is a function local.**
3. On the map path, a traversal **looks through** locals. A resolved call or
   reference made in a local's assignment is its function's own. Relationships
   to a local itself, and unresolved ones a local makes, are not listed.
4. A traversal that leaves such relationships out records
   `local_variables_looked_through`, so the omission is never silent. A
   structural query that visits locals, which is the default and the packet
   path, still returns them.
5. Packets and the public structural query are unchanged.

## Consequences

Measured on the twenty-two-task astropy corpus against the ADR-0153 map, run from
the same directory:

| | ADR-0153 | this decision |
| --- | --- | --- |
| map file recall | 22/27 | 22/27 |
| map symbol recall | 23/34 | 23/34 |
| evidence file recall | 16/27 | 16/27 |
| map items | 1,409 | 1,429 |
| map in the evaluation harness's item shape | 125,020 bytes | 127,040 bytes |
| opening packet and map, per task | 19,350 bytes | 19,433 bytes |
| items naming a function or type | 67.0% | 75.9% |
| items naming a function local | 12.1% (171) | 0.4% (6) |
| items unresolved, showing their source's name | 11.5% | 11.4% |

No task gained or lost a reference symbol or file, and the opening packet's
evidence is unchanged. What changes is what the map names: items that point at
something a reader navigates rise from about two-thirds of the map to three
quarters. `main`, before ADR-0153, stood at 39%.

The map did not shrink. The freed seed slots (the seed limit was reached in 14
of 22 maps, down from 18) went to other candidates, and some of those are weak.
On one task an ambiguous `Table` class seed took a freed slot, and its traversal
and family added 73 items. (Corrected 2026-09-12: this sentence first blamed a
declaration in vendored `extern/configobj` code, which added one item.)
Sixteen maps shrank or stayed the same size, and six grew. The omission
`local_variables_looked_through` appears on 19 of the 22 maps.

Map symbol recall is a proxy. Whether a map that names more real declarations
helps a model's change pass is what a graded correctness run measures, and the
run of changes this decision extends still owes one.

## Alternatives considered

**Stop recording locals in the graph at all.** Deferred. It changes the worker's
facts and what every graph consumer sees. It would also free fact allowance,
which today cuts 24% of the corpus's declarations, so it deserves its own
measurement.

**Look through locals on the packet path too.** Deferred. A packet recovers
each edge's exact source, and the corpus does not measure packets.

**Drop a local's unresolved relationships without saying so.** Rejected. An
omission a consumer can detect is recoverable; one it cannot see is not.
