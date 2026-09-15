# Output Reduction Selection PRD

## Document Control

- PRD ID/version: IC-ORS-160 / 1.2.
- Status: Accepted for implementation.
- Date: 2026-09-15. Version 1.0 was dated 2026-09-13, and 1.1 2026-09-14.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Output Reduction Selection ARD](../architecture/output-reduction-selection-ard.md).
- Governing decisions:
  [ADR-0160](../decisions/0160-keep-the-verdict-and-the-failure-when-reducing-tool-output.md);
  for terminal escape sequences (1.1),
  [ADR-0164](../decisions/0164-remove-terminal-escape-sequences-when-reducing-tool-output.md);
  and for offering failures before their context (1.2),
  [ADR-0165](../decisions/0165-offer-failures-before-their-context-and-skip-repeats-when-reducing-tool-output.md).
- Follows: [ADR-0126](../decisions/0126-answer-host-executed-operations-without-execution-authority.md).

## Problem

Output reduction exists to save the tokens a coding agent spends reading build
and test output, without losing what that output says. The first selection rule
spent its byte budget from the start of the output. It dropped the failure and
the verdict that tools print at the end. On real logs, simply keeping the last
bytes kept the whole answer more often.

(1.1) Tools that color their output wrap its text in terminal escape
sequences, often even when the output is captured. On colored pytest output,
the rules missed the failure detail, the locations and the result line, and
the model received the codes as text.

(1.2) Failures were offered together with their context. A late failure's
context could spend the budget before an earlier failure's own line was
reached, and repeated failure lines spent it on copies. On the real test
output of 22 SWE-bench tasks at 8 KB, the rule lost 6 of 257 facts that way.

## Product Outcome

A reduced log still says which test or check failed, why, where, and what the
run's result was, whether or not the tool colored it. It returns a small share
of the offered bytes, at no model or network cost.

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
   - then failures and locations, from the end backwards. (1.2) Every failure
     and location line comes before any of their context, and a line
     identical to an earlier failure or location is skipped with its context;
   - then warnings;
   - then the start of the output.
4. Keep the in-order line guarantee, the byte bound and the recorded omissions
   unchanged.
5. (1.1) Remove terminal escape sequences from each line before classifying,
   budgeting and returning it, by the grammar in ADR-0164. Remove nothing else.
6. (1.1) Report the escape bytes removed from the returned lines, in exchange
   version 1.1.

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
- (1.1) Tests also cover:
  - colored pytest detail, locations and result lines, classified by their
    text;
  - each kind of escape sequence, and each malformed case, in which only the
    escape character goes;
  - that removal leaves a subsequence with no escape character, and changes
    nothing when repeated;
  - hostile input, which stays linear;
  - a colored failing log that keeps its detail, location and result at 8 KB,
    with the removed bytes counted exactly.
- (1.2) On the 33-log corpus, no log keeps fewer facts than under 1.1 at 4, 8,
  12 or 16 KB, and every failing log still keeps all of its facts at 8 KB.
- (1.2) Tests also cover:
  - every failure line kept ahead of long context;
  - a failure line identical to an earlier one left out with its context,
    while an earlier, different failure is kept.
- The full repository gate passes.

## Non-Goals

- Delivering reduction to a particular client. A host hook remains a separate
  decision.
- Recognizing output in languages other than English, or failures reported only
  through an exit status.
- Changing the exchange schema, except as ADR-0164 records for escape removal.
- Keeping meaning that a tool carries only in color.
