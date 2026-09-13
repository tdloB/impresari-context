# Rank Opening Evidence PRD

## Document Control

- PRD ID/version: IC-ROE-155 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-12.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Rank Opening Evidence ARD](../architecture/rank-opening-evidence-ard.md).
- Governing decision:
  [ADR-0155](../decisions/0155-rank-opening-evidence-and-send-whole-declarations.md).
- Follows: [ADR-0154](../decisions/0154-look-through-local-variables-when-building-a-map.md).

## Problem

The opening packet held two 4,096-byte windows per task, and half of them were
junk. They were chosen by whatever the task quoted first, often `", "` or
`'a'` matching a license header. Nominated test files took slots from the files
a fix changes. Each window was cut around the match, not around the code that
holds it.

## Product Outcome

Opening evidence shows the code a task is about: the definitions that hold the
rarest words a nominated file shares with the task, from the files a fix
changes before their tests. It stays within the same byte budget, with no loss
of map content.

## Functional Requirements

1. Quoted text with no run of three letters or digits is not searched, and the
   plan discloses how many were left out (`literal_without_word`).
2. Literal and lexical signals that a nominated file holds run first, ordered
   by how few snapshot files hold them. Other signals keep the task's order.
3. The ordering is deterministic for a snapshot and recorded by the plan's
   identity.
4. Unless the task asks about tests, test and vendored files' evidence follows
   other files' evidence. None is removed.
5. A search match inside a declared function or type the task's graph holds is
   sent as that whole declaration when it fits the per-item excerpt ceiling;
   otherwise it keeps its centred window.
6. Within each nominated file, whole-declaration matches lead the file's other
   matches.
7. The map, structural queries, and plans without task text behave as before.

## Acceptance Criteria

- On the twenty-two-task astropy corpus against `main`, changed-line coverage
  and evidence file recall do not fall, junk items fall, and the map is
  identical on every task. Measured: 40 to 83 of 213 changed lines, 16 to 18 of
  27 reference files, junk 21/42 to 4/73.
- An arm with each part switched off shows what that part contributes.
- Tests prove the ordering, the disclosure, the demotion and the cut, and each
  fails when its mechanism is disabled.
- The full repository gate passes.

## Non-Goals

- Changing the map, seeds or traversal.
- Cutting module-level matches, or merging literal and lexical deliveries.
- Weighting evidence by a score; the packet stays ordered.
