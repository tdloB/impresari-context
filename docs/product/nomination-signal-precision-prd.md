# Nomination Signal Precision PRD

## Document Control

- PRD ID/version: IC-NSP-133 / 1.0.
- Status: Implemented and measured; acceptance criteria met.
- Date: 2026-09-04.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Nomination Signal Precision ARD](../architecture/nomination-signal-precision-ard.md).
- Governing decision:
  [ADR-0133](../decisions/0133-admit-a-bare-declared-word-only-as-a-type-or-marked-code.md).
- Governing objective: [CLAUDE.md](../../CLAUDE.md) — 98% quality at 78% compression.

## Problem

[IC-DAN-131](declaration-aware-nomination-prd.md) admits a task token as a code
identifier when the repository declares it. It raised map file recall from 11 of
27 to 14 of 27 and shipped, and it is far noisier than that number suggested.

Instrumenting a real request shows what a task actually nominates.
`astropy-8707` admitted sixteen identifiers, of which **twelve were prose**:
`does`, `method`, `creates`, `header`, `a`, `string`, `type`, `on`, `but`,
`work`, `that`, `can`. Its top-ranked nominated file was `CHANGES.rst`.

Three defects produce that:

1. The identifier index admits **every** file. A prose line in a changelog that
   opens with a declaration keyword declares a name.
2. Because a changelog can be nominated, package reach expands from the
   repository root, spending fifteen scope slots on `.gitignore`, `setup.py`
   and `appveyor.yml`.
3. The declaring-file bound does not filter prose. Measured, `Header` is
   declared in one file and so are `a`, `but`, `can`, `work` and `string`;
   `fromstring` is declared in four and so are `does` and `that`.

None of this appeared in a recall score, because recall was measured and
nomination content never was.

## Product Outcome

A task's identifiers are the names it is actually about. A word enters on the
repository's say-so only when something beyond the word marks it as code: it
names a type, it is the member or receiver of a dotted access, or the report's
author wrapped it in backticks or quotes.

## Functional Requirements

1. Index only files the product can parse. A file with no admitted structural
   language contributes no declarations and no identifiers.
2. Admit a bare word on the strength of a declaration only when it begins with
   an uppercase letter.
3. Admit a lowercase declared word when the author marked it as code — quoted or
   backticked — or when it appears as the member or receiver of a dotted access.
4. Require a declaration in either case. Marking a word the repository never
   declares admits nothing.
5. Leave the shape rule untouched. A token that passed before still passes.
6. Stay deterministic: identical snapshot and query yield identical signals.
7. Never consult a reference change, accepted patch, or test outcome.

## Acceptance Criteria

- `Header`, `Card`, `Quantity`, `WCS` are admitted bare when declared.
- `does`, `method`, `creates`, `a`, `string`, `work`, `that` are not admitted
  bare, even when declared.
- A backticked lowercase declared name is admitted; the same name bare is not.
- A backticked word the repository does not declare is not admitted.
- A changelog, CI manifest or README never contributes a declaration.
- **Measured, offline, over all twenty-two astropy tasks:** map file recall
  improves by at least two reference files against the 14 of 27 baseline, and no
  reference file currently recalled is lost.
- The full repository gate passes.

## Measured Outcome

| | map files | map symbols | bytes |
| --- | --- | --- | --- |
| baseline (`main`, IC-DAN-131) | 14 of 27 | 10 of 34 | 19,293,774 |
| case rule only | 17 of 27 | 10 of 34 | 19,411,598 |
| **case rule plus authored markup** | **18 of 27** | **11 of 34** | 19,416,100 (**+0.6%**) |

Four reference files gained, **none lost**. Map file recall 52% to 67%.
`astropy-8707` recalls both of its reference files, `io/fits/card.py` included,
for the first time.

The case rule alone cost `astropy-7671`, which is about `minversion` — a
lowercase function named bare seven times and backticked twice. The markup rule
recovers it, which is why both rules ship together.

## Non-Goals

- Admitting a lowercase function named bare and never marked. A report that
  neither dots nor marks a name has offered nothing to separate it from prose.
- A dictionary or stop-word list of prose to exclude. The repository and the
  author's own markup answer the question without one.
- Revisiting ranking. [IC-DAN-131](declaration-aware-nomination-prd.md)'s
  weighted declaration score is unchanged.
