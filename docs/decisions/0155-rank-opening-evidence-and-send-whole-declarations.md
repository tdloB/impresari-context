# ADR-0155: Rank Opening Evidence by a Nominated File's Rarest Word and Send Whole Declarations

- Status: Accepted
- Date: 2026-09-12
- Decider: Aaron Boldt, who approved amending ADR-0121's anchor check on 2026-09-12
- Related PRD: [Rank Opening Evidence](../product/rank-opening-evidence-prd.md)
- Architecture: [Rank Opening Evidence](../architecture/rank-opening-evidence-ard.md)
- Follows: [ADR-0154](0154-look-through-local-variables-when-building-a-map.md)
- Amends: [ADR-0121](0121-use-bounded-progressive-structural-disclosure.md) — what preserving an ordinary anchor requires

## Context

The opening packet is the evidence a model reads first. On the twenty-two-task
astropy corpus, `main`'s packet held two items per task, each a 4,096-byte
window centred on a match:

- 21 of the 42 items were junk: a needle with no word in it (`", "`, `'a'`,
  `":-)"`), or a window that opened on the license header.
- The items reached 16 of 27 reference files but held only 40 of the 213 lines
  the accepted fixes change.

Three causes:

1. Plan steps ran in the order the task states its signals. A quoted `", "`
   ran first and chose each nominated file's leading excerpt.
2. A nominated test file names the functions its subject defines, so it matches
   the same words and took packet slots.
3. A centred window cannot see structure. It starts mid-line, and near a file's
   top it is mostly header and imports.

A first attempt ranked every signal by how many snapshot files hold it. It made
the evidence worse: 6 of 27 reference files and 13 of 213 lines. The rarest
words in a code repository are prose (`suddenly`, `experimenting`), found in
changelogs and issue templates. Path signals from tracebacks took the remaining
steps and matched nothing.

## Decision

1. Quoted text with no run of three letters or digits is **not searched**. The
   plan discloses it as the omission `literal_without_word`.
2. A literal or lexical signal that **a nominated file holds** runs first,
   rarest first, counted from the lexical index. Every other signal keeps the
   order the task states it in. Without nominated files the plan is unchanged
   apart from (1).
3. Unless the task asks about tests, evidence from **test and vendored files
   follows** the rest, among the nominated files and the whole-snapshot
   results alike. Nothing is dropped.
4. On a structural build, a search match inside a function or type the task's
   graph declares is sent as **that whole declaration** when it fits the
   per-item excerpt ceiling. Otherwise the centred window is kept. The match
   span and the evidence identity are unchanged.
5. Within each nominated file, matches sent as a whole declaration **lead** the
   file's matches outside any declaration. Nothing is dropped.
6. The map, the structural query, and plans built without task text are
   unchanged.
7. The provider-free progressive gate (ADR-0121) still requires every ordinary
   anchor to be preserved, but no longer byte for byte. The same anchors must
   arrive in the same order, with identical identity, file, span, method and
   kind. An excerpt that differs must be exact source, hold its match, and be
   no larger than the ordinary one. Byte-identical excerpts could not survive
   decision 4, because an ordinary build has no graph to cut with.

## Consequences

Measured against `main` on the twenty-two-task astropy corpus:

| | `main` | this decision |
| --- | --- | --- |
| changed lines in the opening evidence | 40/213 (18.8%) | 83/213 (39.0%) |
| reference files holding a delivered changed line | 6 | 11 |
| reference files in the evidence | 16/27 | 18/27 |
| evidence items, average bytes | 42, 4,039 | 73, 2,059 |
| junk items: no word, or a license-header window | 21/42 | 4/73 |
| tasks with no reference file in the evidence | 7 | 5 |
| map | 1,429 items | identical on every task |
| MCP result bytes | 1,757,688 | 1,770,723 (+0.7%) |

The opening packet stays within its 16,384-byte budget. Smaller excerpts are
what let 73 items fit where 42 did.

Each part was switched off in turn, from the build with decisions 1 to 4:

| switched off | changed lines | what changed |
| --- | --- | --- |
| nothing | 85/213 | |
| rating (1 and 2) | 15/213 | `main`'s plan leads with word-less quotes, and the smaller excerpts carry the wrong matches |
| test and vendored files last (3) | 85/213 | on 5 tasks a test or vendored excerpt, such as `jquery-3.1.1.js` or `six.py`, replaces a source file's |
| whole declarations (4) | 79/213 | every excerpt stays a 4 KB window, so 44 items fit instead of 60 |

The rating is what makes the rest work: without it the package scores below
`main`. Decision 5 trades 8 changed lines on three tasks for 6 on three others.
The lines it gives up are module-level (imports and constants near a file's
top), which only a window there covers. In return it reaches one more
reference file and cuts junk items from 13 to 4.

Evidence-file recall credits an excerpt that is only a license header, so the
changed-line count, not file recall, is the measure this decision was judged by.
Whether the new evidence helps a model's change pass is what a graded run
measures, and that run is still owed.

## Alternatives considered

**Rank every signal by whole-snapshot rarity.** Rejected: measured worse on
every evidence count, for the reasons in Context.

**Weight test files by a factor, as Graft does (0.35×).** Not applicable. The
packet keeps evidence in order and drops from the end, so a weight becomes an
order. Demoting is that order.

**Cut a match outside any declaration to its line.** Deferred. Module-level
matches are imports and constants, and the corpus does not yet show whether a
narrower window helps.

**Treat a literal and a lexical match of one declaration as one delivery.**
Deferred. Provenance is part of the delivery key by an earlier decision, and
changing it deserves its own measurement.
