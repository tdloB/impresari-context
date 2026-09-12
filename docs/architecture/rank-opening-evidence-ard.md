# Rank Opening Evidence — Architecture Requirements and Design

- ARD ID/version: IC-ROE-ARD-155 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-12.
- Governing PRD: [IC-ROE-155](../product/rank-opening-evidence-prd.md).
- Decision: [ADR-0155](../decisions/0155-rank-opening-evidence-and-send-whole-declarations.md).

## Rating a signal

`WorkspaceCache::lexical_document_counts(terms, within)` counts current-generation
files holding every term: in the whole snapshot, and among the `within` path
units, which are bound as values. Terms pass the same validation as
`lexical_candidates`, so no FTS syntax is accepted.

The engine's `prepare_lexical_index` makes that index describe the current
snapshot, the work a lexical search already did on its first use.
`build_profiled_context_internal` prepares it, maps the nominated display paths
to path units (`nominated_units`), and rates a needle as a `NeedleRarity`:
how many files hold all its words, and whether a nominated file is among them.
Words are counted whole and at most sixteen per needle, so a count ranks
needles; it never predicts a search's result.

## Ordering the plan

`deterministic_plan_with` takes the rating as an oracle. `expand_profile_steps`
keeps the whole query first, then:

```text
quoted text with no 3-letter run  -> left out, counted as literal_without_word
literal or lexical, in a nominated file -> "reaching", sorted by files (stable)
everything else, paths included   -> "rest", in the task's order
steps = reaching then rest, deduplicated, at most eight
```

At most sixteen quoted and thirty-two lexical signals are rated. With no oracle,
no index, or no nominated file, every signal is in "rest", which is `main`'s
order.

## Ordering the evidence

`EvidenceRanking` carries two choices into
`build_planned_context_with_supplemental_internal`:

- `demote_supporting_files`, true for a profiled build whose task text names no
  test word. Evidence from a path under `test`, `tests`, `extern`, `vendor` and
  similar directories, or named like `test_*`, `*_test`, `conftest`,
  `*.test.*` or `*.spec.*`, follows the rest: the nominated scope is reordered
  before the file-first pass, and whole-snapshot results are partitioned
  stably.
- `declarations`, a `DeclarationIndex` built from the seeded build's graph:
  the spans of every `function` and `type` node, by path units.

## Cutting to a declaration

For the first `max_evidence_items` records in delivery order, a literal or
lexical match inside the smallest declaration that holds it and fits
`max_excerpt_bytes_per_item` is re-expanded with `expand_evidence_record` to
exactly `[start, end)` of that declaration. The record is revalidated against
source, keeps its identity and match span, and still passes packet validation.
Any other record keeps its window. Identical cut excerpts are then delivered
once, by the existing rule.

## Within a file

`cut_records` returns each record with whether it was cut. The nominated
records are partitioned stably, cut ones first, before the file-first pass.
That pass keeps each file's order, so within a file a whole declaration leads
the matches outside any declaration, and the rarest step still orders each
group. Whole-snapshot records are cut in order and not partitioned.

## Measurement

Against `main` on the twenty-two-task astropy corpus:

- Changed lines in the opening evidence rise from 40/213 to 83/213.
- Reference files holding a delivered changed line rise from 6 to 11.
- Reference files in the evidence rise from 16/27 to 18/27.
- Items rise from 42 to 73 as their average size falls from 4,039 to 2,059
  bytes. Junk items fall from 21 of 42 to 4 of 73.
- The map is identical on every task. MCP result bytes rise 0.7%.

Switching off the rating drops changed lines to 15/213. Switching off
test-and-vendored demotion leaves them at 85/213 but sends a test or vendored
excerpt on 5 tasks. Switching off whole declarations leaves 79/213, with 44 items instead of 60,
because every excerpt stays a 4 KB window. Declarations leading their
file cost 2 changed lines net and cut junk items from 13 to 4.

## Verification

- `task_signals_are_rated_before_the_step_limit_cuts_them` fails when the
  ordering is removed.
- `signals_no_nominated_file_holds_keep_the_tasks_order` fails when every rated
  signal is ranked.
- `the_rarest_word_a_nominated_file_holds_leads_its_excerpt` fails when the
  ordering is removed or no nominated file is known.
- `lexical_document_counts_are_current_scoped_and_compiled` fails when the
  nominated count is always zero.
- `quoted_text_without_a_word_is_disclosed_not_searched` fails when word-less
  literals are kept.
- `test_files_follow_the_files_a_task_changes` fails when demotion is off or no
  file is a supporting file; `supporting_files_are_tests_and_vendored_copies`
  fails in the second case.
- `a_match_is_sent_as_the_declaration_holding_it` fails when no cut is made.
- `the_smallest_fitting_declaration_holds_a_match` fails when the largest is
  chosen.
- `within_a_file_a_whole_declaration_leads_its_other_matches` fails when cut
  records do not lead their file, or when no cut is made.
- `frozen_provider_free_progressive_structural_gate_passes` now checks ordinary
  anchors by identity, file, span, method and order, and requires any differing
  excerpt to be exact source that holds its match and is no larger than the
  ordinary one (ADR-0155 decision 7, amending ADR-0121).
