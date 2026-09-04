# Nomination Signal Precision — Architecture Requirements and Design

- ARD ID/version: IC-NSP-ARD-133 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-04.
- Governing PRD: [IC-NSP-133](../product/nomination-signal-precision-prd.md).
- Decision:
  [ADR-0133](../decisions/0133-admit-a-bare-declared-word-only-as-a-type-or-marked-code.md).

## What a recall score could not see

[IC-DAN-131](../product/declaration-aware-nomination-prd.md) was measured by
map file recall alone: 11 of 27 to 14 of 27, shipped. Instrumenting a single
request showed what that number was hiding.

```text
astropy-8707 admitted identifiers
  real  : fromstring  Header
  prose : does  method  creates  header  a  string  type  on  but  work  that  can

astropy-8707 top nominated file
  CHANGES.rst

astropy-8707 reach expansion
  .gitignore  .travis.yml  README.rst  setup.py  appveyor.yml  conftest.py …
```

Twelve of sixteen identifier slots were prose, the anchor was a changelog, and
reach expanded the repository root. Recall still rose, because two real
identifiers were enough to carry it — which is exactly why a recall score alone
is not sufficient evidence that a selection change is working.

## The bound that was never doing the job

IC-DAN-131 excluded prose with a declaring-file bound, on the argument that
prose words are declared everywhere. Measured:

| token | real | files declaring |
| --- | --- | --- |
| `Header` | yes | 1 |
| `a`, `but`, `can`, `work`, `string`, `type` | no | 1 |
| `fromstring` | yes | 4 |
| `does`, `that` | no | 3 |

The distributions overlap completely. No threshold on this count separates the
two, so the bound was never the thing keeping prose out — nothing was.

## What does separate them

Every noise token is a **bare lowercase English word**. Every real one is either
capitalised, or carries positional evidence:

```text
Header              capitalised          → a type name          → admit
Header.fromstring   dotted receiver      → a type name          → admit
Header.fromstring   dotted member        → a method name        → admit
`minversion`        authored markup      → the author says code → admit
work                bare lowercase word  → prose                → reject
```

Case is the discriminator for types, which is what single-word class names are.
For lowercase names, the evidence has to come from somewhere other than the
word, and there are exactly two honest sources: the dot, and the author.

## Authored markup is not token position

[ADR-0124](../decisions/0124-classify-task-signals-by-code-shape-not-token-position.md)
rejected classifying a token by where it sits in a sentence, and that still
holds. A backtick is not a position — it is a decision the report's author made
about that specific word, the same class of evidence as writing out a file path.

The rule stays conservative in both directions: marking without a declaration
admits nothing, so quoting a prose word does not smuggle it in.

## Non-code files declare nothing

The identifier index read every artifact, so `CHANGES.rst` could declare a name
and be nominated for it. Restricting the index to files with an admitted
structural language fixes that, and fixes package reach with it — a changelog
can no longer be the anchor whose package is the repository root.

This also shrinks the index, since a repository's non-code files are excluded
from a structure the product could never parse anyway.

## Bounds

The declaring-file bound is retained. It is no longer load-bearing — case and
markup are — but it remains a cheap guard against a name declared in hundreds of
files, and removing it would be a change this measurement does not justify.

## Preserved invariants

`SEC-INV-002`, `SEC-INV-003`, `SEC-INV-007`, `SEC-INV-011` and `SEC-INV-012`
hold unchanged. Admission narrows rather than widens, reads nothing new, and a
token is still admitted on repository evidence rather than consumer say-so.
