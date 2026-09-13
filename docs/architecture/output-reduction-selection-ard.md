# Output Reduction Selection — Architecture Requirements and Design

- ARD ID/version: IC-ORS-ARD-160 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-13.
- Governing PRD: [IC-ORS-160](../product/output-reduction-selection-prd.md).
- Decision: [ADR-0160](../decisions/0160-keep-the-verdict-and-the-failure-when-reducing-tool-output.md).

## Classifying a line

`line_kind` classifies each line once. First, `without_timestamp` skips a
leading ISO-8601 UTC timestamp found within the line's first 160 bytes. GitHub
Actions logs and `gh run view --log` prefix every line with one.

```text
passing test: "... ok", ✓, PASSED, --- PASS, TAP ok   -> quiet
failure word, failed-test symbol, pytest "E",
  TAP "not ok", Rust "left:" / "right:"               -> failure
warning word, or a type name ending in Warning         -> warning
file.ext:12, file.ext(12,5), File "x", line 12         -> location
anything else                                          -> quiet
```

A word is a run of letters, digits and `_`. It does not count in any of these
cases:
- it sits inside a token that names a file, meaning the token holds a path
  separator or ends in a short lowercase extension;
- it sits inside a command-line flag;
- it follows `0`, `no`, `zero` or `without`, or is followed by `0`.

A type name ending in `Error`, `Exception`, `Failure` or `Warning` also counts,
when the ending follows a lowercase letter or a digit.

Separately, `reports_verdict` marks result lines:
- pytest's `== N passed ==`;
- unittest's `Ran N tests`, `OK` and `FAILED (`;
- cargo's `test result:`;
- jest's `Tests:`;
- go's `ok` and `FAIL` package lines;
- the TAP and node runner's `# fail N`;
- mypy's `Success:`;
- `could not compile`;
- any whole number followed by a result word, such as `2 failed` or
  `39 problems`.

A line or column number, as in `models.py:15: error`, is not a count.

## Choosing lines

`select_lines` offers line indices to a `Picker`. The picker adds a line if its
bytes plus a newline fit what is left of the budget. Otherwise it records that
the budget was reached and moves on. Lines are offered in this order:

1. the last verdict line, then every verdict that reports a failure, latest
   first;
2. the first failure;
3. the last three lines;
4. the first failure's context;
5. every failure and location with `context_lines` of context on either side,
   latest first;
6. every warning and verdict with its context, latest first;
7. the first three lines.

The chosen indices are returned in ascending order and joined with newlines, so
the response is still a subsequence of the offered lines.

## Safety

- **Linear work:** classification is linear in the line length and uses no
  regular expressions, so hostile output cannot cause super-linear work.
- **No panics:** every index goes through checked access.
- **No authority:** the module still holds no process, file or environment
  access. `module_holds_no_execution_or_workspace_authority` checks this.

## Measurement

The engine's own `reduce_host_output` was run on 33 real logs, with 2 lines of
context:
- the 14 logs from the first check;
- 19 new logs from pytest, unittest, jest, vitest, node's test runner, go test,
  go build, tsc, mypy, eslint, cargo build and clang. Each tool ran on a
  throwaway project, failing and, where the tool has one, passing.

"Tail" means keeping only the last bytes of the output within the same budget.

| Budget | Whole answer kept, 20 failing logs (new / tail / first rule) | Result kept, 13 runs of fixed code (new / tail / first rule) | Bytes returned (new / tail) |
| --- | --- | --- | --- |
| 4 KB | 18 / 12 / 6 | 13 / 11 / 8 | 7% / 9% |
| 8 KB | 20 / 18 / 10 | 13 / 13 / 9 | 11% / 13% |
| 16 KB | 20 / 20 / 12 | 13 / 13 / 9 | 13% / 20% |

- **Never worse than the tail:** the new rule never kept fewer facts than the
  tail at the same budget.
- **Failures never lost:** the new rule never lost a failure entirely. The
  first rule lost one in 8 of 60 cases, and the tail in 3.
- **The only misses:** both 4 KB misses are astropy-13033, whose evidence is
  larger than 4 KB. The new rule kept both failing tests' names, every line of
  the target test's assertion, and the verdict. It dropped one or two lines of
  an unrelated leap-second warning and some traceback locations.

## Verification

- `a_late_failure_survives_early_noise_that_would_fill_the_budget` reproduces
  the first rule's defect.
- Budget and priority behaviour:
  - `the_first_error_and_the_latest_survive_a_small_budget`;
  - `the_verdict_survives_when_noise_follows_it`;
  - `assertion_detail_is_kept_even_far_from_a_failure_word`.
- Line classification:
  - `failure_words_count_only_on_their_own_and_never_as_zero_counts`;
  - `a_passing_test_is_quiet_even_when_its_name_holds_a_failure_word`;
  - `verdicts_are_recognized_across_tools`;
  - `source_locations_are_recognized_and_ordinary_colons_are_not`;
  - `timestamped_ci_lines_are_classified_by_their_message`.
- Bounds and edge cases: `no_budget_is_ever_exceeded` and
  `empty_output_returns_nothing_and_claims_nothing`.
- Every earlier test still passes unchanged.
