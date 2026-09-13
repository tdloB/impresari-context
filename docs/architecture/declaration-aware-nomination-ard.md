# Declaration-Aware Nomination — Architecture Requirements and Design

- ARD ID/version: IC-DAN-ARD-131 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-04.
- Governing PRD: [IC-DAN-131](../product/declaration-aware-nomination-prd.md).
- Decision:
  [ADR-0131](../decisions/0131-admit-and-rank-identifiers-by-declaration.md).

## Shape is the wrong oracle

Deciding whether a word is code by looking at the word is guesswork, and the
guess fails in a specific, systematic direction: it rejects the single-word
class name, which is the commonest shape a class takes.

```text
   is "Header" code?

   by shape          →  no interior separator, no interior capital  →  reject
   by repository     →  `class Header` in astropy/io/fits/header.py →  admit
```

The repository already holds the answer, and preparation already reads every
admitted file to build the identifier index. Recording what each file *declares*
while that read happens costs one more set per file and no additional read.

The shape rule stays. It is the answer when there is no index — a first request
before preparation completes, or a snapshot the index does not cover — and it
still admits `_required_columns` and `__array_ufunc__`, which no `class` or
`def` line will produce.

## Declarations are found lexically

A declaration keyword at the start of a line, after optional modifiers, followed
by a name:

```text
    class Header(_CardAccessor):        →  Header
    def fromstring(cls, data, sep=''):  →  fromstring
    pub fn admitted_paths(&self)        →  admitted_paths
    public static void main(String[])   →  main
```

The keyword set is a union across admitted languages rather than a table keyed
by language: `class`, `def`, `fn`, `func`, `function`, `struct`, `enum`, `trait`,
`interface`, `type`, `impl`, `object`, `record`, `module`, `defmodule`, `defn`,
`sub`, `package`, `namespace`.

A union is deliberately cruder than a per-language grammar, and cheaper in a way
that matters: fifteen languages would otherwise need fifteen tables to maintain,
each a place for drift. The cost of the union is a false positive when one
language's keyword opens a line in another — which admits a name that a task
would still have to mention before it changed anything.

### What the keyword rule does not cover

A C-family method declares a return type rather than a keyword, so
`public static void main(String[] args)` yields no name. Type declarations are
covered in every admitted language — `public class CardAccessor` is found — but
method declarations only where a keyword introduces them, which is Python, Rust,
Go, JavaScript, TypeScript, Ruby, Elixir, Clojure, Kotlin, Scala, Swift and PHP.

Extending to C-family methods means matching "modifiers, a type, a name, an open
parenthesis", which also matches any line calling a function after a variable
declaration. That is a worse trade than the gap, and the gap is recorded here
rather than papered over.

Requiring the keyword at line start is what keeps prose out. Documentation says
"the class Header is constructed from"; it does not begin a line with `class
Header`. The line-start rule is doing the work that token position could not do
in [ADR-0124](../decisions/0124-classify-task-signals-by-code-shape-not-token-position.md),
because a declaration genuinely has a position and a prose mention does not.

## Declaring outranks mentioning

`Header` appears in hundreds of astropy files and is declared in one. Counting a
mention like a definition puts the anchor almost anywhere.

It is not, however, worth *everything*. Measured, ranking declaration as an
absolute tier gained six reference files and lost five: a file declaring one
task identifier displaced a file declaring the central type and mentioning most
of the rest. A task about `Table` anchored on `io/fits/column.py`, which
declares `Column`, over `table/table.py`, which declares `Table`.

So a declaration is weighted, not tiered:

```text
score = mentions + 3 × declarations       ties break by path
```

Three is the smallest weight that keeps a lone declaration ahead of a lone
mention while still letting four mentions outweigh one unrelated declaration.

The split in that measurement is what chose this shape. The six gains came from
**admission** — tasks that previously produced no identifier at all — and the
five losses came from **ranking**. Widening admission and softening ranking are
separable, and only the second needed changing.

This matters beyond nomination. The anchor chosen here is the file seeds
resolve into and the file traversal expands from, so a wrong anchor is not one
bad pick but a wrong starting point for every stage that follows.

## Bounds

| bound | why |
| --- | --- |
| declarations retained per file | a generated file cannot dominate the index |
| identifier length | shared with the existing index ceiling |
| snapshot binding | shared with the index; a stale answer is refused |

A breach is an explicit unknown, following
[ADR-0121](0121-use-bounded-progressive-structural-disclosure.md).

## Admission is not authority

A token is admitted because the *repository* declares it. Repository content is
data (`SEC-INV-003`), and nothing a consumer sends can add a declaration, so
this widens what nomination looks at without widening what any caller can reach
— which is the distinction `SEC-INV-012` draws.

The index continues to hold names and portable paths only, so it cannot become a
second retrieval path around the exact-source provenance rules in `SEC-INV-011`.

## Preserved invariants

`SEC-INV-002`, `SEC-INV-003`, `SEC-INV-007`, `SEC-INV-011` and `SEC-INV-012`
hold unchanged. Declaration extraction reads the workspace during preparation
exactly as the identifier index already does, in the same pass, writes nothing,
executes nothing, and retains no source content.
