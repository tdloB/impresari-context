# Output Reduction Selection — Architecture Requirements and Design

- ARD ID/version: IC-ORS-ARD-160 / 1.2.
- Status: Accepted for implementation.
- Date: 2026-09-15. Version 1.0 was dated 2026-09-13, and 1.1 2026-09-14.
- Governing PRD: [IC-ORS-160](../product/output-reduction-selection-prd.md).
- Decisions:
  [ADR-0160](../decisions/0160-keep-the-verdict-and-the-failure-when-reducing-tool-output.md);
  for terminal escape sequences (1.1),
  [ADR-0164](../decisions/0164-remove-terminal-escape-sequences-when-reducing-tool-output.md);
  and for offering failures before their context (1.2),
  [ADR-0165](../decisions/0165-offer-failures-before-their-context-and-skip-repeats-when-reducing-tool-output.md).

## Removing terminal escape sequences (1.1)

`reduce_host_text` splits the offered text into lines first, as `str::lines`
does, so a host counts the same lines in the bytes it offered. Each line then
goes through `remove_terminal_escapes`. Classification, the budget and the
returned text all use the result.

```text
ESC [, params 0x30-0x3F, intermediates 0x20-0x2F, final 0x40-0x7E    control sequence
ESC ] P X ^ _, then up to a bell or ESC \ on the same line           string control
ESC, intermediates 0x20-0x2F, final 0x30-0x7E                        such as ESC ( B
ESC, one byte 0x30-0x7E                                              such as ESC 7
ESC before anything else, or an unfinished sequence                  the ESC alone
```

- Every sequence starts and ends on an ASCII byte, so what is left is whole
  characters.
- A newline never belongs to a sequence. The Claude Code adapter relies on
  this when it removes escapes from a stream it keeps whole.
- Text without an escape character is returned borrowed and unchanged.
- `escape_bytes_removed` sums, over the returned lines, each offered line's
  length less its returned length. A host can recompute it from its own lines.

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
5. every failure and location line, latest first, except one identical to an
   earlier failure or location line (1.2);
6. the context of those lines, `context_lines` on either side, latest first
   (1.2);
7. every warning and verdict with its context, latest first;
8. the first three lines.

Until 1.2, steps 5 and 6 were one step: every failure and location with its
context, latest first. A late failure's context could then spend the budget
before an earlier failure's own line was reached, and repeated lines spent it
on copies (ADR-0165). `fresh_evidence` finds the lines for step 5 by exact
match against every earlier failure or location line.

The chosen indices are returned in ascending order and joined with newlines.
From 1.1 each returned line is the offered line without its terminal escape
sequences, so the response is whole offered lines in their original order, and
every byte in it is one the host supplied.

## Safety

- **Linear work:** classification is linear in the line length and uses no
  regular expressions, so hostile output cannot cause super-linear work.
- **Linear removal (1.1):** a scan that starts at an escape character stops at
  the next escape character, a newline, or its terminator. Each byte is
  therefore read a bounded number of times.
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

**Terminal escape sequences (1.1).** The benchmark harness's provider-free base
run of 2026-09-14 ran this reducer on 22 SWE-bench tasks. Ten print colored
pytest output. On those, the classification above missed pytest's detail, its
locations and its result line. None of the 33 corpus logs holds an escape
character. Rebuilt from `main` and from 1.1, the engine's measurement returned
byte-identical selections and identical counts for all 33 logs at 4, 8 and
16 KB. The harness's re-run measures the colored tasks.

**Failures before their context (1.2).** Measured the same way, on the same 33
logs:

| Budget | Whole answer kept, 20 failing logs (1.2 / 1.1 / tail) | Facts kept, of 198 (1.2 / 1.1) | Result kept, 13 runs of fixed code (1.2 / 1.1) | Bytes returned (1.2 / 1.1) |
| --- | --- | --- | --- | --- |
| 4 KB | 19 / 18 / 12 | 196 / 180 | 13 / 13 | 7.4% / 7.5% |
| 8 KB | 20 / 20 / 18 | 198 / 198 | 13 / 13 | 9.8% / 11.1% |
| 12 KB | 20 / 20 / 20 | 198 / 198 | 13 / 13 | 10.4% / 12.8% |
| 16 KB | 20 / 20 / 20 | 198 / 198 | 13 / 13 | 11.0% / 13.3% |

No log kept fewer facts under 1.2 than under 1.1, at any budget. The
harness's budget sweep on 22 SWE-bench tasks agrees. At 8 KB, 1.2 kept 257 of
257 facts against 251, and returned 74.9% fewer text bytes than the host cap
against 73.1%. Over the 18 tasks that print tracebacks, the figures are 71.7%
against 69.6%.

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
- Escape removal (1.1):
  - `terminal_escape_sequences_are_removed_whole_and_nothing_else_is`;
  - `removal_leaves_a_subsequence_without_escapes_and_is_stable`;
  - `hostile_escapes_are_removed_in_linear_time`.
- Colored output (1.1):
  - `a_colored_line_is_classified_by_its_text`;
  - `a_colored_failure_keeps_its_detail_location_and_result_without_codes`;
  - `text_reduction_matches_the_exchange`, which now also runs a colored log.
- Failures before their context (1.2):
  - `every_failure_line_comes_before_any_context`;
  - `a_failure_line_identical_to_an_earlier_one_is_left_out_with_its_context`.
- Every earlier test still passes unchanged.
