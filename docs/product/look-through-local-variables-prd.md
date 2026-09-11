# Look Through Local Variables PRD

## Document Control

- PRD ID/version: IC-LTL-154 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-11.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Look Through Local Variables ARD](../architecture/look-through-local-variables-ard.md).
- Governing decision:
  [ADR-0154](../decisions/0154-look-through-local-variables-when-building-a-map.md).
- Follows: [ADR-0153](../decisions/0153-record-what-a-declaration-is-and-deliver-a-seeds-family.md).

## Problem

A function's local variables were seeds, map items and traversal stops. 28% of
seeds were locals named like task words, and 22% of the items `main`'s map
delivered named a local. Because a call made in an assignment belonged to the
local, 53% of a function seed's resolved calls and references were out of reach
of its traversal.

## Product Outcome

A map names what a reader navigates, which is functions, types, attributes and
module names, and credits a function with the calls it makes, with no loss of
recall and with the omission recorded.

## Functional Requirements

1. A variable is function-local when the nearest enclosing declaration that is
   not a variable is a function.
2. On the map path, seed selection never admits a function local.
3. On the map path, a traversal credits a local's resolved calls and references
   to its function, and does not list relationships to a local or unresolved
   ones a local makes.
4. A traversal that left such relationships out records
   `local_variables_looked_through`.
5. Packets and the public structural query visit locals as before.

## Acceptance Criteria

- On the twenty-two-task astropy corpus, file, symbol and evidence file recall
  are unchanged from ADR-0153, and no task gains or loses a reference symbol or
  file.
- Map items naming a function local fall from 171 to at most a handful, and the
  share naming a function or type rises.
- The opening packet's evidence is identical on every task.
- Tests prove that a traversal looks through locals and says so, that only a map
  does, and that a map never seeds on a local. Each fails when its mechanism is
  disabled.
- The full repository gate passes.

## Non-Goals

- Removing locals from the graph or the worker's facts.
- Changing packets or the public structural query.
- Improving seed selection beyond excluding locals.
