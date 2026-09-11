# Declaration Kind and Seed Family — Architecture Requirements and Design

- ARD ID/version: IC-DSF-ARD-153 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-11.
- Governing PRD: [IC-DSF-153](../product/declaration-kind-and-seed-family-prd.md).
- Decision: [ADR-0153](../decisions/0153-record-what-a-declaration-is-and-deliver-a-seeds-family.md).

## Declaration kind

Every declaration fact already carries the tree-sitter `syntax_kind` it came
from. `build_graph` maps it onto the symbol node:

| kind | syntax kinds |
| --- | --- |
| `function` | function and method declarations and definitions, constructors, abstract method signatures, Ruby methods, Swift protocol functions, Haskell functions |
| `type` | classes, interfaces, structs, unions, enums, records, traits, objects, protocols, type aliases and definitions, C# delegates |
| `variable` | Python assignments, JavaScript lexical and variable declarations, JSON and YAML pairs, Haskell bindings |
| `other` | namespaces, modules, TOML tables, and anything unlisted |

`GraphNode` gains `declaration_kind: Option<String>`, serialized only when
present, and `validate_graph` rejects any value outside the four. A node's
identity hashes its snapshot, path and local key rather than its fields, so no
node identity changes. Edge identities hash fact provenance, which now records
graph 1.1.0; selection has not depended on identities since ADR-0151.

The graph schema gives nodes an optional `declaration_kind` enumeration and, with
the query schema, pins version 1.1.0. The conformance fixtures move to 1.1.0. The
valid graph fixture carries a declared function, and
`structural-graph-unknown-declaration-kind.json` proves that an unknown kind is
rejected.

## The family

`context_structural::seed_family(graph, seed, edge_kinds, max_edges)` returns a
`StructuralQueryResult` whose edges are, in order:

```text
enclosing  the edge that declares the seed's container
members    the seed's contains children of kind function or type, source order   (types)
nested     function and type children of the seed (functions) or of each member
bases      the seed's references to types that start before its first contained
           declaration, each followed by that type's members                     (types)
```

A header precedes a type's body, so a reference that starts before the first
thing the type contains is written in its header, which is where a base class
is. Children are ordered by their target's span, and identity breaks only an
exact span tie. Requested edge kinds are honoured, and bases are followed only
when references are permitted. Each edge appears once. Past `max_edges` the
result is truncated and says `seed_family_limit_reached`. Its nodes are the seed
and every endpoint of its edges, so each item resolves.

## Delivery

```text
seeds -> each seed's traversal (16 edges) -> each seed's family (64 edges) -> merge -> output bound -> map
```

`seeded_structural_query` appends every seed's family after every seed's
traversal, each bounded by the structural output limit. Merging keeps the first
occurrence of an edge, so a family edge the traversal already delivered is not
repeated, and the map's collapse (ADR-0152) drops repeats of visible content.
The packet path passes `deliver_family = false`: a packet recovers each edge's
exact source, and a family would multiply excerpts. `MAX_SEED_FAMILY_EDGES` is
64, sized to list the corpus's largest seeded classes whole (`Card` has 39
function and type members and `Table` 44), and is pinned by
`the_seed_traversal_budget_is_pinned`.

## Measurement

Against current `main`, run from the same directory on the twenty-two-task
astropy corpus:

- Map symbol recall rises from 15/34 to 23/34. File recall (22/27) and evidence
  file recall (16/27) are unchanged, and no symbol is lost.
- `main`'s map is an exact prefix of the new map on all twenty-two tasks, and the
  opening evidence is identical.
- Items go from 763 to 1,409, the map in the evaluation harness's item shape from
  69,236 to 125,020 bytes, the whole build result from 1,375,810 to 1,743,588
  bytes, and packet plus map per task from 16,811 to 19,350 bytes.
- The largest map has 154 items, and three maps reach the family limit.

## Verification

- `declarations_record_what_they_declare` fails when every declaration is
  recorded as `other`.
- `a_seed_family_is_its_enclosing_class_members_nested_functions_and_bases` fails
  when kinds are erased or bases are not followed.
- `a_map_names_a_seeded_methods_class_and_a_packet_does_not` fails when the
  engine delivers no family, or delivers one to packets as well.
- The conformance suite accepts the valid graph fixture and rejects the unknown
  kind.
