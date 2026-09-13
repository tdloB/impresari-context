# Stable Structural Selection PRD

## Document Control

- PRD ID/version: IC-SES-151 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Stable Structural Selection ARD](../architecture/stable-structural-selection-ard.md).
- Governing decision:
  [ADR-0151](../decisions/0151-select-structural-context-by-what-it-is.md).
- Refines: [ADR-0145](../decisions/0145-spread-structural-seeds-across-files.md).

## Problem

The structural map an agent receives was chosen partly by hash order. Which
seeds a file contributed, which of a seed's relationships survived its limit,
and what an output ceiling would keep were all decided by identities that hash
the workspace snapshot and the recorded parser version. The same code therefore
produced a different map depending on where it was checked out, after any
commit, and after a relabel of the parser, although no fact had changed.

Relabelling the parser alone replaced 280 of 1,440 map items on the
twenty-two-task astropy corpus and moved map symbol recall from 15 to 13 of 34.
The same build of `main`, checked out in a second directory, delivered 385 of its 1,440 map items differently and moved map symbol recall from 15 to 12 of 34. Hash order also admitted unresolved references ahead of the
confirmed relationships a seed contains.

## Product Outcome

The same code yields the same map wherever it is checked out and whatever parser
version it records, and the map prefers relationships the product resolved.

## Functional Requirements

1. A traversal takes a node's edges most confidently resolved first, then by
   source position, with kind, target path, name and span, and module breaking
   ties.
2. Merging per-seed traversals keeps seed order and traversal order.
3. A seed tie within one file goes to the earliest declaration.
4. Identity breaks a tie only between entries identical in content.
5. Every existing limit is unchanged: eight seeds, three per file, sixteen edges
   per seed at depth one, and the output ceiling.

## Acceptance Criteria

- The same build under two parser labels, run from one directory, delivers
  identical maps on all twenty-two astropy tasks.
- The same build run from two checkout directories delivers identical maps on
  all twenty-two tasks.
- Map file recall stays at 22 of 27 and map symbol recall at 15 of 34 in every
  directory and under either label.
- Tests prove identity independence, resolved-first order, same-named targets,
  seed ties across snapshots, and merge order; each fails with its mechanism
  disabled.
- The full repository gate passes.

## Non-Goals

- Collapsing map items that repeat another item's visible content, which is a
  separate follow-up.
- Ranking relationships by the task's identifiers.
- Changing any limit.
