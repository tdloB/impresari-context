# Repeat Read Notice PRD

## Document Control

- PRD ID/version: IC-RRN-158 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-12.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Repeat Read Notice ARD](../architecture/repeat-read-notice-ard.md).
- Governing decision:
  [ADR-0158](../decisions/0158-tell-a-session-it-already-holds-an-unchanged-read.md).
- Amends: [ADR-0136](../decisions/0136-substitute-a-host-read-with-declaration-spans.md).

## Problem

Read substitution sends a full answer every time. A session that reads the same
unchanged file twice pays for the same declarations twice.

## Product Outcome

A repeated read of unchanged source costs a short notice instead of the answer,
and the full answer stays one call away.

## Functional Requirements

1. The server remembers, per session, each delivered answer's identity and
   request, never its bytes.
2. A repeat offer whose newly computed answer is identical returns a notice
   naming the earlier request, the answer identity, and the bytes withheld.
3. `repeat: true` returns the full answer. A changed source, a different symbol,
   or another session always gets the full answer.
4. Closing a session forgets it. Memory per session is bounded and never evicts.
5. The notice adds no authority and says how to receive the answer again.

## Acceptance Criteria

- Tests prove a repeat of an unchanged answer returns the notice with no span
  bytes, and that `repeat`, a changed source and another session each get the
  full answer.
- Tests prove a closed session is forgotten, that nothing is recorded past the
  bound, and that nothing already remembered is evicted.
- The full repository gate passes.

## Non-Goals

- Skipping the source read on a repeat.
- Notices for context builds, evidence expansion, or map lookups.
- Measuring how often agents repeat reads; that belongs to a graded run.
