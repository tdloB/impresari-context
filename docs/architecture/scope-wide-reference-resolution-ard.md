# Scope-Wide Reference Resolution — Architecture Requirements and Design

- ARD ID/version: IC-SWRR-ARD-132 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-04.
- Governing PRD: [IC-SWRR-132](../product/scope-wide-reference-resolution-prd.md).
- Decision:
  [ADR-0132](../decisions/0132-resolve-references-across-the-scope-not-within-a-file.md).

## The graph is a set of islands

```text
        header.py                         card.py
   ┌───────────────────┐            ┌───────────────────┐
   │ Header ──calls──▶ │            │ Card              │
   │ Header ──refs───▶ │  ✗ no target│ Card.fromstring   │
   └─────────┬─────────┘            └─────────┬─────────┘
             │                                │
             └────────── imports ─────────────┘
                  (file node → file node)
```

`add_call` and `add_reference` look their target up in `file.response.facts` —
the declarations of the file the edge starts in. A reference from `Header` to
`Card` finds no `Card` inside `header.py`, so the edge carries no target and is
marked `unresolved`. Traversal will not follow it, correctly: it must never
invent a graph target.

Symbols therefore connect only to symbols in their own file. The single
cross-file edge kind is `imports`, and it joins **file nodes**, not symbols.

## Why no budget could have fixed this

Doubling traversal depth was run as a control before designing anything:

| | map files | map symbols | distinct map files | bytes |
| --- | --- | --- | --- | --- |
| depth 1 | 14 of 27 | 10 of 34 | 68 | 19,293,774 |
| depth 2 | 14 of 27 | 11 of 34 | 68 | 19,511,148 (+1.1%) |

Not one task changed. A second hop through a graph whose cross-file edges
dead-end reaches the same places at a higher price.

The disclosures said as much beforehand and are worth reading together:
`unresolved_traversal_target` in 20 of 21 maps, depth limit in 19, edge limit in
16, seed limit in 17. Every limit saturated at once is not five tuning problems;
it is one structural problem being reported five ways.

## The builder already has what it needs

`build_graph_with_unknowns` takes `Vec<GraphFileInput>` — every file in the
scope — and iterates it twice already, once to create file nodes and once to add
relationships. `add_relationships` is handed `file_nodes` for all files but only
`local_nodes` for the current one.

So a scope-wide declaration index is one pass over data already in hand: no new
read, no new parse, no per-language resolver.

## Resolution order, and why the first rule stays

```text
1. a declaration of that name in the originating file   → resolve (unchanged)
2. exactly one declaration of that name in the scope     → resolve (new)
3. several declarations of that name in the scope        → unresolved + ambiguous
4. none                                                  → unresolved (unchanged)
```

Rule 1 first means every edge that resolves today resolves to the same target
tomorrow. The change is additive: it converts dead ends into edges, and never
redirects a live one. That property is what makes the change safe to measure —
any recall movement is attributable to newly resolved edges rather than to
silently rewired old ones.

Rule 3 is the honesty rule. `__init__`, `get` and `read` are declared in dozens
of scope files, and choosing among them would be inventing a target with extra
steps. Ambiguity is disclosed under its own reason code so a consumer can
distinguish "the scope declares no such name" from "the scope declares several"
— two different reasons a map stops where it does.

## The resolution label does not change

An edge resolved this way stays `heuristic`, the same label a within-file name
match already carries. It is a name match in both cases, not a language-aware
resolution: no import aliasing, no scoping, no shadowing, no dynamic dispatch.
Introducing a fourth resolution value would imply a confidence this does not
have, and would change a vocabulary that contracts and tests already pin.

## An inconsistency this change deliberately leaves alone

`add_call` takes the **first** matching declaration in the file; `add_reference`
requires the match to be **unique**. Two rules for the same question.

The uniqueness rule is the better one, and the cross-file path uses it for both.
Tightening the within-file path would change edges that resolve today, which
would break the additive property above and make a measurement unattributable.
It is recorded here as known and left for its own change.

## Bounds

The scope is already bounded — sixteen nominated files, plus reach where
admitted — so the declaration index is bounded by the graph that contains it. No
new ceiling is introduced, because no new unbounded quantity exists.

## Preserved invariants

`SEC-INV-002`, `SEC-INV-003`, `SEC-INV-007` and `SEC-INV-011` hold unchanged.
Resolution consumes facts the builder already holds, performs no repository
read, writes nothing, executes nothing, and retains no source content. A target
is still never invented: an edge resolves to a declaration that exists in the
graph, or it resolves to nothing and says so.
