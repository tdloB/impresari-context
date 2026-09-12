# Declaration Kind and Seed Family PRD

## Document Control

- PRD ID/version: IC-DSF-153 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-11.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Declaration Kind and Seed Family ARD](../architecture/declaration-kind-and-seed-family-ard.md).
- Governing decision:
  [ADR-0153](../decisions/0153-record-what-a-declaration-is-and-deliver-a-seeds-family.md).
- Follows: [ADR-0152](../decisions/0152-collapse-map-items-that-repeat-visible-content.md).

## Problem

The map named 15 of the 34 declarations the astropy corpus's accepted fixes
changed. Of the nineteen it missed, eight sat in a seed's class structure: a
seeded class's later methods, a seeded method's class, functions nested in a
seeded class's methods, and methods a seeded class inherits. A one-hop traversal
from the seed cannot reach them. The graph could not tell a method from a local
variable, so it could not follow a class's contents without listing every local.

## Product Outcome

A map names the declarations around each seed that make it whole, which are its
enclosing declaration, members, nested declarations and bases, and loses nothing
it named before.

## Functional Requirements

1. Every symbol node records `declaration_kind` as `function`, `type`, `variable`
   or `other`, from the parser's syntax kind. File nodes record none.
2. The graph contract version is 1.1.0. Schemas, fixtures and provenance agree,
   and an unknown kind is rejected.
3. The map delivers, per seed and in this order: the enclosing declaration; a
   type's function and type members; declarations nested one level inside the
   seed or those members; and a type's header-named base types, each followed by
   their members.
4. Variables are never part of a family.
5. A family holds at most 64 edges and reports `seed_family_limit_reached` when
   cut.
6. Families follow every seed's own traversal. The packet path delivers none.

## Acceptance Criteria

- On the twenty-two-task astropy corpus, map symbol recall rises from 15/34 to
  23/34, file and evidence file recall are unchanged, and no symbol is lost.
- `main`'s map is an exact prefix of the new map on every task.
- The opening packet's evidence is identical on every task.
- Tests prove that declarations record their kind, that a family is exactly
  enclosing declaration, members, nested declarations and bases and never a
  variable, and that only a map carries a family. Each fails when its mechanism is
  disabled.
- The full repository gate passes.

## Non-Goals

- Keeping seeds off local variables, or crediting a local's calls to its
  function.
- Changing the per-seed traversal limit or the fact allowance.
- Delivering families in packets.
