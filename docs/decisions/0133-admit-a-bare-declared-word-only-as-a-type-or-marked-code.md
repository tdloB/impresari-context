# ADR-0133: Admit a Bare Declared Word Only as a Type or Marked Code

- Status: Accepted
- Date: 2026-09-04
- Related PRD: [Nomination Signal Precision](../product/nomination-signal-precision-prd.md)
- Architecture: [Nomination Signal Precision](../architecture/nomination-signal-precision-ard.md)
- Corrects: [ADR-0131](0131-admit-and-rank-identifiers-by-declaration.md)

## Context

ADR-0131 admits a task token as a code identifier when the repository declares
that name in few enough files to identify one. It bounded that at eight
declaring files, arguing that prose words are declared everywhere and so would
be excluded.

Instrumenting a real request refuted the argument. On `astropy-8707`:

| token | a real identifier | files declaring it |
| --- | --- | --- |
| `Header` | yes | 1 |
| `a`, `but`, `can`, `work`, `string`, `type` | no | 1 each |
| `fromstring` | yes | 4 |
| `does`, `that` | no | 3 |

The count does not separate a name from a word. Twelve of that task's sixteen
identifier slots held prose — `does`, `method`, `creates`, `header`, `a`,
`string`, `type`, `on`, `but`, `work`, `that`, `can`.

Two further defects compounded it. The identifier index admitted **every** file,
so a prose line in `CHANGES.rst` opening with a keyword declared a name; that
changelog then ranked first in the nomination, and package reach expanded from
its package — the repository root — spending fifteen scope slots on
`.gitignore`, `setup.py` and `appveyor.yml`.

None of this was visible in the recall number. ADR-0131 measured 11 of 27 to 14
of 27 and shipped, because the measurement looked at recall and never at what
was being nominated.

## Decision

Index only files the product can parse. A changelog, a CI manifest and a README
declare nothing.

Admit a **bare** word on the repository's say-so only when it names a type —
when it begins with an uppercase letter. That is the single-word class name
ADR-0131 exists for: `Header`, `Card`, `Quantity`, `Table`, `WCS`, `HDUList`.

Admit a lowercase declared word when evidence beyond the word itself marks it as
code: as the member or receiver of a dotted access, or wrapped in backticks or
quotes by the report author.

Retain the declaring-file bound. It is not the instrument this ADR relies on,
but it remains a cheap guard against a name declared in hundreds of files.

## Consequences

Map file recall rises from 14 of 27 to 18 of 27, with four reference files
gained and none lost, for 0.6% more delivered bytes. `astropy-8707` recalls both
of its reference files, `io/fits/card.py` included, for the first time.

The case rule alone cost one file: `astropy-7671` is about `minversion`, a
lowercase function named bare seven times. The markup rule recovers it, because
the report backticks the name twice.

Authored markup is not token position. ADR-0124 rejected classifying a token by
where it sits in a sentence, and that still holds. A backtick is a decision the
report's author made about that word, and marking alone remains insufficient —
the repository must still declare the name, so quoting a prose word admits
nothing.

A lowercase function named bare and never marked is not admitted. That is a real
gap, accepted because the alternative measured twelve prose tokens per task, and
because a report that never marks a name and never dots it has given nothing to
distinguish it from prose.

No security invariant changes. This record grants no execution, network,
publication, or submission authority.
