# Nominated-File Evidence First — Architecture Requirements and Design

- ARD ID/version: IC-NFE-ARD-148 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Governing PRD: [IC-NFE-148](../product/nominated-evidence-first-prd.md).
- Decision: [ADR-0148](../decisions/0148-search-nominated-files-first-for-packet-evidence.md).

## Where packet evidence came from

A profiled packet plans up to eight search steps from the task text: quoted
literals, then portable paths, then code identifiers, then lexical terms. For
each step the engine searched the snapshot and appended the matches to the
evidence order.

Three orderings decided which of those matches an agent received, and none of
them was about the task:

```text
search_candidates     walks snapshot artifacts, sorted by encoded path units
bound_search_response drops records from the end until the response fits
build_packet          admits in evidence order, drops from the end to fit
```

With 4 KiB excerpts a 16 KiB packet holds two records, so the evidence was the
first two matches of the first productive step in encoded path order.

## The change

The seeded structural request already carries the nomination order, because
seed selection needs it. It is now also passed to packet assembly as a
preferred scope, from `build_profiled_seeded_structural_context_internal`
through `build_profiled_context_internal` to
`build_planned_context_with_supplemental_internal`. Every other caller passes an
empty scope, which builds exactly the packet it built before.

For each literal or lexical step, the engine first searches each nominated file
on its own through `search_internal`, whose new `within` argument narrows a
search to named snapshot paths, and collects those matches. It then runs the
whole-snapshot search as before and collects its matches separately.

```text
evidence order = caller-declared evidence
               + nominated-file matches, one per file per pass, nomination order
               + whole-snapshot matches, in their existing order
               + structural evidence
```

`file_first_by_scope` produces the second segment. The existing delivery key,
path plus extraction method plus kind plus excerpt, is unchanged, so a match
both searches find is admitted once.

## Why one search per file

A search over all nominated files together returns its matches sorted in
snapshot order, and the response bound then cuts from the end. The nominated
file that sorts first fills the response and hides the rest before the engine
can reorder anything. Measured on the corpus, that version reached 12 of 27; on
`astropy-12907` it delivered two test files while `separable.py`, nominated
first and holding `separability_matrix` seven times, never appeared. A search
per file gives each nominated file its own bound, and the regression test
`a_nominated_file_leads_even_when_one_sorting_earlier_fills_the_search_budget`
pins it.

## Retrieval

`search_literal_in` and `search_lexical_in` search only the named snapshot path
units, verify every match against exact source, and return matches in the order
a whole-snapshot search uses. A path the snapshot does not hold fails as stale
state. The lexical cache only narrows a whole-snapshot search to candidate
files; a named scope is already narrow, so the scoped lexical search verifies
the same terms with the same case folding directly.

## Cost

At most sixteen nominated files are searched for each literal or lexical step,
and a plan has at most eight steps, so a packet runs at most 128 additional
single-file searches. Each reads one file already admitted to the snapshot.
Latency was not measured on the corpus.

## Preserved invariants

- Every search keeps its file, match, memory, elapsed and output bounds.
- Every search binds to the current snapshot and fails if the workspace changed
  during planning.
- Evidence is exact source with an unchanged delivery key.
- A nominated search that reaches a limit is disclosed as
  `plan_step_N_nominated_limited`.
- No new authority: the scope is the nomination the product already disclosed
  under ADR-0142, not a caller input.

## Verification

- `nominated_files_lead_the_packet_evidence` fails when the nominated search is
  disabled.
- `a_nominated_file_leads_even_when_one_sorting_earlier_fills_the_search_budget`
  fails against a single search over all nominated files.
- `scoped_search_reads_only_the_named_files` covers both retrieval functions.
