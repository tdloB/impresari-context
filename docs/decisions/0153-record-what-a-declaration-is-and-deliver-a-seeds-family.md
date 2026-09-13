# ADR-0153: Record What Each Declaration Is, and Deliver a Seed's Family in the Map

- Status: Accepted
- Date: 2026-09-11
- Related PRD: [Declaration Kind and Seed Family](../product/declaration-kind-and-seed-family-prd.md)
- Architecture: [Declaration Kind and Seed Family](../architecture/declaration-kind-and-seed-family-ard.md)
- Follows: [ADR-0152](0152-collapse-map-items-that-repeat-visible-content.md)

## Context

On the twenty-two-task astropy corpus the map named 15 of the 34 declarations
the accepted fixes changed. Each of the other nineteen was traced through the
engine's own graph, signals, seeds and traversal orders, dumped by an
instrumented build that reproduced `main`'s maps exactly. Eight sat in a seed's
class structure, which a seed's traversal cannot reach:

- three methods of a seeded class that fall past the traversal's sixteen-edge
  limit, in source order;
- the class that contains a seeded method;
- two functions nested inside a seeded class's methods;
- two methods a seeded class inherits from a base class.

The traversal follows only what a seed refers to, one hop out. The graph could
not have supported more: a symbol node did not record what it declared, so a
method was indistinguishable from a local variable. Seventy-four per cent of
symbol nodes on the corpus are not functions or types but assignments, and
following a class's contents without that distinction would list every local of
every method.

## Decision

1. **Graph nodes record what they declare.** A symbol node carries
   `declaration_kind`: `function`, `type`, `variable` or `other`, mapped from the
   syntax kind the parser already reports for every declaration. File nodes carry
   none. The graph contract moves to 1.1.0, and the graph and query schemas, the
   conformance fixtures and the fact provenance move with it. An unknown kind
   fails validation.

2. **The map delivers each seed's family.** For each seed, in order:

   1. the declaration that encloses it, by the edge that declares it, so its
      name is shown;
   2. for a type, its function and type members, in source order;
   3. the functions and types nested one level inside the seed, if it is a
      function, or inside those members;
   4. for a type, the types its header refers to, which is where a base class is
      written, each followed by its function and type members.

   Variables are never family. A family holds at most 64 edges, and a family cut
   there says `seed_family_limit_reached`.

3. **Families follow every seed's own traversal**, so they only add to what the
   map delivered before, and a ceiling that cuts the map cuts family first.

4. **Only the map delivers families.** A packet recovers each edge's exact
   source, and a family of sixty-four members would be sixty-four excerpts. The
   packet path is unchanged.

## Consequences

Measured on the twenty-two-task astropy corpus against current `main`, run from
the same directory:

| | `main` | this decision |
| --- | --- | --- |
| map file recall | 22/27 | 22/27 |
| map symbol recall | 15/34 | 23/34 |
| evidence file recall | 16/27 | 16/27 |
| map items | 763 | 1,409 |
| map in the evaluation harness's item shape | 69,236 bytes | 125,020 bytes |
| whole build result | 1,375,810 bytes | 1,743,588 bytes |

The eight symbols gained are exactly the eight traced to class structure, and
none was lost. All eight sit in files the map already named: the change points
at the right declaration inside a file rather than at a new file.

`main`'s map is an exact prefix of the new map on every task, so the version
change moved no item and families only append. The opening packet's evidence is
unchanged; per task, packet and map together grow from 16,811 to 19,350 bytes
(15.1%). The largest map holds 154 items, under the ceiling of 256, and three
maps reached the family limit. Of the 646 added items, 540 are members and
nested declarations, 100 name an enclosing declaration, and six are base types.
At most about fifty come from seeds that are themselves variables, whose family
is only their enclosing declaration.

Compression has no measured denominator yet. Against the files the accepted
fixes changed, a rough floor on what an agent without this product opens,
delivery goes from 61.6% smaller to 55.8% smaller; against the sixteen nominated
files, from 97.6% to 97.3%. Map symbol recall is a proxy: it shows the map names
the right declarations, not that a model's change passes. The run of recall gains
this decision extends owes a graded correctness run.

Eleven of the nineteen remain: five call a seed through a local variable, five
sit three or more hops away, and one lies past where its file's fact allowance
ran out.

## Alternatives considered

**Raise the per-seed edge limit.** Rejected: a seed's traversal spends added
room on what it already spends most of its room on. Of the 1,371 edges seeds
delivered on `main`, 45% are unresolved and 22% point at local variables, while
the missing methods sit 21st to 33rd among their class's declarations.

**Infer a node's kind from its name or source text in the engine.** Rejected:
only the parser knows whether a name is a function or an assignment, and a guess
in the engine would be a second parser that can disagree with the first.

**Deliver families in packets as well.** Rejected for now, per decision 4.

**Keep seeds off local variables first.** Deferred: 28% of seeds are local
variables, and keeping seeds off them, while crediting a local's calls to its
function, changes selection and is measured on its own.
