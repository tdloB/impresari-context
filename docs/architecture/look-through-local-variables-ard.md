# Look Through Local Variables — Architecture Requirements and Design

- ARD ID/version: IC-LTL-ARD-154 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-11.
- Governing PRD: [IC-LTL-154](../product/look-through-local-variables-prd.md).
- Decision: [ADR-0154](../decisions/0154-look-through-local-variables-when-building-a-map.md).

## Which variables are local

`context_structural::function_locals(graph)` maps each function-local variable to
the function it belongs to. A variable node (ADR-0153's `declaration_kind`) climbs
its `contains` parents past any enclosing variables; if the first declaration
that is not a variable is a function, the variable is that function's local.
Class attributes and module-level names are not locals. The climb is bounded by
the node count, which only guards a malformed graph, because containment nests.

## Looking through

`LocalVariables` is `Visit` or `LookThrough`. `query_graph` visits, and
`query_graph_with` takes the choice. `LocalVariables::looked_through(graph)`
yields the locals to look through: all of them, or none when visiting.

When building each node's outgoing edges, looking through reads every local as
part of its function:

```text
edge from a local, resolved      -> the function's own edge (its identity and span unchanged)
edge from a local, unresolved    -> left out
edge to a local                  -> left out
any other edge                   -> unchanged
```

The traversal then runs exactly as before, in the same order and within the same
limits. A looked-through edge still names the local as its source, so the result
includes that node and every item resolves. The nodes for which anything was
left out are collected. When the traversal expands one of them within its depth,
it records `local_variables_looked_through`, and the map carries that among its
omissions.

## Where it applies

The engine's `StructuralDelivery` is `Map` or `Packet`. `seeded_structural_query`
takes it, and it replaces the flag ADR-0153 used for families:

- `Map` selects seeds with `LocalVariables::LookThrough`: `is_named_symbol` no
  longer admits a function local. Seed traversals then run with `LookThrough`
  and every seed's family is appended.
- `Packet` selects and traverses with `Visit` and delivers no family.
- The public `query_structure` visits locals.

## Measurement

Against the ADR-0153 map, run from the same directory on the twenty-two-task
astropy corpus:

- File (22/27), symbol (23/34) and evidence file (16/27) recall are unchanged, no
  task gains or loses a reference symbol or file, and the opening evidence is
  identical.
- Items go from 1,409 to 1,429, and packet plus map per task from 19,350 to 19,433
  bytes.
- Items naming a function or type rise from 67.0% to 75.9%. Items naming a
  function local fall from 171 to 6.
- 19 of 22 maps carry `local_variables_looked_through`.

## Verification

- `a_traversal_can_look_through_a_functions_locals` fails when:
  - no variable counts as local;
  - a local's unresolved edges are kept;
  - edges into locals are delivered;
  - the omission is never recorded.
- `a_map_looks_through_a_seeds_locals_and_a_packet_does_not` fails when:
  - no variable counts as local;
  - the map path visits locals;
  - edges into locals are delivered;
  - the omission is never recorded.
- `a_map_never_seeds_on_a_functions_local_variable` fails when no variable counts
  as local, or when seed selection ignores locals.
