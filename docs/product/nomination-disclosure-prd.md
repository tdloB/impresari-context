# Nomination Disclosure PRD

## Document Control

- PRD ID/version: IC-ND-142 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-07.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Nomination Disclosure ARD](../architecture/nomination-disclosure-ard.md).
- Governing decision:
  [ADR-0142](../decisions/0142-disclose-the-files-a-scoped-graph-was-built-over.md).
- Completes: [IC-SSSE-128](seed-scoped-structural-extraction-prd.md) requirements
  6 and 9.

## Problem

A scoped structural graph is dense but partial. The consumer receives a map and
cannot tell which files that map could ever have named, so it cannot tell a
confident, well-attributed, wrong answer from a right one.

IC-SSSE-128 anticipated exactly this and required the disclosure. It was never
implemented. The nomination is computed on every scoped build, used to seed and
to scope the graph, and then discarded.

Two consequences, both measured:

1. A consumer cannot distinguish a sixteen-file scoped graph from a
   whole-repository one. The map reads as complete.
2. **Nomination recall cannot be measured.** The governing architecture names it
   the leading indicator — a file never nominated can never be mapped, so it
   bounds map recall from above — and three consecutive investigations into map
   recall stuck at 19 of 27 could not rule nomination in or out for any of the
   eight tasks it misses.

## Product Outcome

Every scoped structural map states which files it was scoped to, how many were
considered, why each was admitted, and what limited the nomination.

A whole-repository fallback says so explicitly rather than omitting the field,
so the two cases are never confused by absence.

## Functional Requirements

1. A progressive structural map discloses its nomination: the admitted files in
   rank order, the ground each was admitted on, how many distinct task
   identifiers each carries, how many candidates were considered, and the
   nomination's own shortfall reasons.
2. The disclosure states plainly whether coverage is scoped to nominated files.
   A whole-repository graph reports `scoped_to_nominated_files: false` and an
   empty file list.
3. The disclosure is derived, never supplied. No consumer input reaches it, and
   it grants no capability a consumer did not already hold.
4. The disclosure names only paths the map may already name. It discloses no
   file the consumer could not otherwise see.
5. Nomination recall becomes measurable offline by comparing the disclosed file
   list against a reference change, with no product involvement.

## Acceptance Criteria

- A scoped build's map carries the nomination; a whole-repository build's map
  carries the explicit unscoped form.
- A test proves the disclosure reflects the nomination actually used, and fails
  if the map is emitted without it.
- The disclosure adds no read, no capability, and no consumer-supplied input.
- The full repository gate passes, including adversarial and fuzz suites.

## Non-Goals

- Changing nomination, seeding, or traversal. This record only discloses what
  already happens.
- Improving nomination quality. Measuring it first is the point.
- Any provider request or paid evaluation.
