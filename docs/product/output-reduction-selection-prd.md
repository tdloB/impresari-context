# Output Reduction Selection PRD

## Document Control

- PRD ID/version: IC-ORS-160 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-13.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Output Reduction Selection ARD](../architecture/output-reduction-selection-ard.md).
- Governing decision:
  [ADR-0160](../decisions/0160-keep-the-verdict-and-the-failure-when-reducing-tool-output.md).
- Follows: [ADR-0126](../decisions/0126-answer-host-executed-operations-without-execution-authority.md).

## Problem

Output reduction exists to save the tokens a coding agent spends reading build
and test output, without losing what that output says. The first selection rule
spent its byte budget from the start of the output. It dropped the failure and
the verdict that tools print at the end. On real logs, simply keeping the last
bytes kept the whole answer more often.

## Product Outcome

A reduced log still says which test or check failed, why, where, and what the
run's result was. It returns a small share of the offered bytes, at no model or
network cost.

## Functional Requirements

1. Recognize failures, warnings, source locations and verdict lines across
   common build and test tools.
2. Do not count as failures:
   - zero counts;
   - file names;
   - command-line flags;
   - clock times;
   - passing tests whose names hold a failure word.
3. Keep lines in this priority order:
   - the verdict, the first failure, and the end of the output first;
   - then failures and locations, from the end backwards;
   - then warnings;
   - then the start of the output.
4. Keep the exchange schema, the in-order subsequence guarantee, the byte bound
   and the recorded omissions unchanged.

## Acceptance Criteria

- On the 33-log corpus at 8 KB:
  - every failing log keeps all of its scored facts;
  - every run of fixed code keeps its result line.
- At 4, 8 and 16 KB, the reducer never keeps fewer facts than keeping the last
  bytes with the same budget, and never loses a failure entirely.
- Tests cover:
  - the late failure that the first rule dropped;
  - the first and the latest errors under a small budget;
  - a verdict followed by noise;
  - assertion detail;
  - zero counts, file names, command-line flags, clock times and passing test
    names;
  - verdict recognition;
  - empty output;
  - that no budget is ever exceeded.
- The full repository gate passes.

## Non-Goals

- Delivering reduction to a particular client. A host hook remains a separate
  decision.
- Recognizing output in languages other than English, or failures reported only
  through an exit status.
- Changing the exchange schema.
