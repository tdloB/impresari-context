# ADR-0160: Keep the Verdict and the Failure When Reducing Tool Output

- Status: Accepted
- Date: 2026-09-13
- Related PRD: [Output Reduction Selection](../product/output-reduction-selection-prd.md)
- Architecture: [Output Reduction Selection](../architecture/output-reduction-selection-ard.md)
- Follows: [ADR-0126](0126-answer-host-executed-operations-without-execution-authority.md)

## Context

ADR-0126 lets a host offer output it has already produced and receive a bounded
selection of its lines, with omissions recorded. The first selection rule kept
any line holding one of twelve marker words or a `name:digits` reference, a few
lines around each, and the first and last three lines. It then spent the byte
budget from the start of the output forwards.

An offline check ran that rule on 14 real logs: SWE-bench grader runs for three
tasks, and the Windows `cargo test` failure from #305. It scored four facts per
failing log: which test failed, why, where, and the result line. At the same
byte budget, keeping only the last bytes kept the whole answer in 16 of 24
cases. The rule kept it in 4 and lost the failure entirely in 7. There were four
causes:

1. Tools print their verdict and their latest failure at the end, and the
   budget ran out before the end.
2. Markers fired on `0 failed`, on the file name `warnings.rst`, and on clock
   times read as `name:digits`, so every line of a timestamped CI log looked
   important.
3. Python traceback frames, `path(line,col)` locations, and result lines such
   as `OK` and `15 passed` were not recognized.
4. Assertion detail more than two lines from a marker was cut.

## Decision

1. Classify each line once as one of four kinds:
   - a passing test;
   - a failure: a failure word standing alone and not a zero count, a
     failed-test symbol, pytest's `E` detail, TAP's `not ok`, or a Rust
     assertion operand;
   - a warning;
   - a source location: `file.ext:12`, `file.ext(12,5)`, or Python's
     `File "x", line 12`.

   Separately, mark whether the line reports a run's verdict. A leading
   ISO-8601 timestamp is skipped before classifying. Words inside file names or
   command-line flags do not count. A passing test is never a failure, even
   when its name holds a failure word.
2. Offer lines to the budget in this order:
   1. the verdict: the last verdict line, and every verdict that reports a
      failure;
   2. the first failure;
   3. the last three lines;
   4. the first failure's context;
   5. every failure and location with its context, from the end backwards;
   6. warnings and the other verdicts, the same way;
   7. the first three lines.

   A line that does not fit is skipped, so a later, shorter line can still fit.
3. Nothing else changes:
   - the exchange schema stays 1.0;
   - the response is still an in-order subsequence of the offered lines;
   - the budget is never exceeded;
   - omissions are recorded as before.

## Consequences

The rule was measured on 33 real logs: the original 14, plus failing and passing
runs of pytest, unittest, jest, vitest, node's test runner, go test, go build,
tsc, mypy, eslint, cargo build and clang.

| At 8 KB | New rule | Keeping the last bytes | First rule |
| --- | --- | --- | --- |
| Whole answer kept, 20 failing logs | 20 | 18 | 10 |
| Result line kept, 13 runs of fixed code | 13 | 13 | 9 |
| Share of offered bytes returned | 11% | 13% | 11% |

At 4, 8 and 16 KB the new rule never keeps fewer facts than keeping the last
bytes, and never loses a failure entirely.

Recognition is lexical and in English. A tool that reports failure only through
its exit status, or in another language, is covered by the anchors alone. The
corpus does not include every tool. It is the evidence for this rule, not proof
that the rule fits all output.

## Alternatives considered

**Keep only the last bytes.** Rejected. It lost Django's result, which the
runner prints before its set-up chatter. At 4 KB it kept the whole answer in 12
of 20 failing logs; the new rule kept it in 18.

**Spend the budget from the end backwards only.** Rejected. A compiler's first
error is often the cause of the rest. tsc printed the root cause last and mypy
printed it first, so both ends are kept.

**Ask a model to choose the lines.** Rejected. Reduction must stay
deterministic and free, and unable to introduce text.
