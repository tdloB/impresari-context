# Rarity-Weighted Nomination — Architecture Requirements and Design

- ARD ID/version: IC-RWN-ARD-149 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Governing PRD: [IC-RWN-149](../product/rarity-weighted-nomination-prd.md).
- Decision: [ADR-0149](../decisions/0149-weight-nomination-by-identifier-rarity.md).

## Where nomination lost reference files

`nominate_files` admits exact task paths first, ranks every other file holding
an admitted identifier by mentions plus three times declarations, and keeps
sixteen. The three corpus tasks holding the four reference files the snapshot
held but nomination missed had all reached that ceiling, with 43 to 290
candidate files.

Two asymmetries decided which files filled it:

- The identifier index records a file's mentions only for names shaped like
  code. A capitalised word the repository declares, such as `ITRS`, `Header` or
  `WCS`, is admitted as a task identifier through its declaration but is never
  counted as a mention anywhere. It contributes three to its declaring file,
  while a file mentioning four common code-shaped names scores four.
- Every identifier weighed the same, however many files held it.

## Rarity

```text
holders(name) = max(files mentioning name, files declaring name), at least 1
weight(name)  = ⌊log₂(files in snapshot ÷ holders(name))⌋ + 1
score(file)   = Σ 3 × weight(each name it declares) + Σ weight(each name it mentions)
```

In the replay on `astropy-13398`, a name one file holds weighs eleven and a
name thirty files hold weighs six, so the file declaring `ITRS` scores 33 and
enters the sixteen. A name every file holds weighs one, so when every name is
common the rule reduces to the previous count.

The weights come from the index answers nomination already requests, so it
still reads nothing. The logarithm is `u64::ilog2`, and ties still break by
path, so the ranking is deterministic on every platform.

## Dotted module names

`resolve_module_path` applies to a task path the snapshot does not hold that
contains a dot and no slash:

1. Replace the dots with slashes to form a suffix.
2. If any tracked path lies under a directory named by the suffix, the name is
   a package: resolve nothing.
3. A tracked path matches when its path with the extension removed equals the
   suffix or ends in `/` followed by it.
4. Resolve only when exactly one path matches.

A resolved file ranks after the exact task paths and before inferred matches,
under `task_module_path`. In the replay the rule resolved eight module names
across seven tasks. One was a reference file nothing else nominated; three were
reference files already nominated.

## Schema

The disclosure schema moves from 1.0 to 1.1 for the new reason and the changed
ranking. The MCP scope disclosure takes the version from the constant, and the
evaluation adapter reads reason codes as strings and pins no nomination
version.

## Preserved invariants

- At most sixteen files, with exact task paths still first.
- Deterministic: integer arithmetic and a path tie-break.
- No oracle, execution or network authority; the module isolation test is
  unchanged.
- Nomination reads no repository bytes.

## Verification

- `a_rare_declared_name_outranks_files_mentioning_common_ones` fails when every
  weight is one.
- `a_dotted_module_name_nominates_the_file_it_denotes` fails with module
  resolution disabled.
- `a_dotted_name_for_a_package_or_several_files_nominates_none` covers both
  refusals.
- Every existing nomination test passes unchanged.
