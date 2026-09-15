# ADR-0165: Offer Failures Before Their Context and Skip Repeats When Reducing Tool Output

- Status: Accepted
- Date: 2026-09-15
- Related PRD: [Output Reduction Selection](../product/output-reduction-selection-prd.md)
- Architecture: [Output Reduction Selection](../architecture/output-reduction-selection-ard.md)
- Amends: [ADR-0160](0160-keep-the-verdict-and-the-failure-when-reducing-tool-output.md)
  decision 2, step 5

## Context

ADR-0160 offers every failure and source location together with its context,
from the end backwards. Offered that way, the context of the latest failures
can spend the budget before an earlier failure's own line is reached. Many
failures that print the same message, such as the cases of one parametrized
test, spend it the same way on copies.

The benchmark harness ran this reducer on the real test output of 22 SWE-bench
tasks at 8 KB, after ADR-0164 (repository-context-eval ADR-0083). Its scorer
marks 257 facts in the failing runs. The rule kept 251, losing 3 failure
details and 3 locations on two tasks, all to the budget. The harness's budget
sweep (repository-context-eval ADR-0085) then compared the rule with two trial
rules on the same captures, at 4, 8, 12 and 16 KB.

## Decision

1. **Offer every failure and location line before any of their context.**
   Step 5 of ADR-0160's order becomes two passes: every failure and location
   line, latest first; then their context, latest first.
2. **Skip a failure or location line identical to an earlier one.** Only its
   first occurrence is offered as evidence, and only that occurrence's context
   is offered. Matching is exact. The verdicts, the first failure and the
   trailing anchors are offered as before. A copy can still be returned as a
   trailing anchor, or when it sits within the context of a line that is
   offered.
3. **Nothing else changes.**
   - The exchange stays 1.1.
   - The response is still whole offered lines in their original order.
   - The budget is never exceeded, and omissions are recorded as before.

## Consequences

- In the harness's sweep at 8 KB, on the same captures:

  | Rule | Facts kept, of 257 | pytest code lines kept, of 59 | Text bytes less than the host cap: 22 tasks / 18 that print tracebacks |
  | --- | --- | --- | --- |
  | ADR-0160 | 251 | 52 | 73.1% / 69.6% |
  | Decision 1 alone | 257 | 50 | 73.1% / 69.6% |
  | Decisions 1 and 2 | 257 | 55 | 74.9% / 71.7% |

  Four of the 22 tasks run pytest with tracebacks turned off, so they print no
  failure detail, and every rule saves about 92% on them. The figure for the
  other 18 tasks is shown so that those four do not flatter the result.
- On ADR-0160's 33-log corpus, measured with the engine:
  - at 4 KB the whole answer is kept for 19 of 20 failing logs, against 18, and
    196 of 198 facts, against 180. No log keeps fewer facts than before;
  - at 8, 12 and 16 KB every fact is kept, as before, in 9.8%, 10.4% and 11.0%
    of the offered bytes, against 11.1%, 12.8% and 13.3%;
  - every run of fixed code keeps its result line.
- Decision 1 alone kept one fact fewer than ADR-0160 on one corpus log at 4 KB.
  Skipping repeats freed the room it needed.
- A copy of a line loses its own context. The first copy keeps its context,
  and lines that name each failing test, such as pytest's `FAILED` summary
  lines, differ from one another and are all still offered.
- Lines that differ only in, say, a number or an address are not repeats, so
  nothing new is ever skipped.

## Alternatives considered

**Keep ADR-0160's order and raise the budget to 12 KB.** Rejected. It keeps all
257 facts, but returns more bytes: 71.5% less than the host cap, against 74.9%.

**Decision 1 alone.** Rejected. It kept every fact at 8 KB, but fewer pytest
code lines than ADR-0160, and one fact fewer on one corpus log at 4 KB.

**Skip lines that are merely similar, such as those differing only in
numbers.** Rejected. It could merge different failures into one, and exact
matching never drops a line that says something new.
