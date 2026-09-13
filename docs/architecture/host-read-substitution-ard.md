# Host Read Substitution — Architecture Requirements and Design

- ARD ID/version: IC-HRS-ARD-136 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-05.
- Governing PRD: [IC-HRS-136](../product/host-read-substitution-prd.md).
- Decision:
  [ADR-0136](../decisions/0136-substitute-a-host-read-with-declaration-spans.md).

## The map has no return path

```text
 today                                    with substitution
 ─────                                    ─────────────────
 map: core.py, _check_required_columns    map: core.py, _check_required_columns
        │                                        │
        ▼                                        ▼
 agent needs the bytes                    host offers the read to Impresari
        │                                        │
        ▼                                        ▼
 read_repository_file(core.py)            declaration spans, hashed
   → the whole file                         → the part that matters
```

An agent given a pointer and no way to follow it will use the tool it has. That
is why the treatment arm reads *more* than baseline rather than less: it pays
for the map, then pays for the file.

Substitution closes the loop. The host still performs every operation — it is
the host that reads, and the host that decides — but the answer it gets back is
the declarations rather than the file.

## What a substitution returns

The file's **declarations**, as spans, in source order.

Symbol nodes in the structural graph already carry byte-authoritative spans, so
the answer is recovered rather than computed: the span bounds come from the
graph, and the bytes come from the snapshot the host already authorized.

Declarations are the right unit because they are what a map points at. A map
naming `_check_required_columns` in `core.py` is answered exactly by that
method's span, and the surrounding declarations give the reader enough structure
to know where it sits.

### The path is not the unit; the symbol is

Returning *every* declaration returns the file. Measured through this tool over
eight astropy files, a whole-path answer is 92.6% of the source — Python is its
classes and functions, so the declarations are the module minus its imports. A
host offered that should decline it.

So an offer may name the symbol as well as the path, and then the answer is that
declaration alone: a median of 0.7% of the file across 494 declarations.

Nesting is deliberately **not** collapsed for a named symbol. The whole-path
answer drops enclosed declarations so no byte is returned twice, which is right
when the question is "what does this file declare." It is wrong when the question
is "where is this method": the enclosing class contains it, so collapsing returns
the class, which is the whole-path answer again.

A name may be declared more than once in a file. Every declaration carrying it is
returned, in source order; returning only the first would hide the one the host
is looking for.

### Absence is a claim, and a prefix cannot support it

The structural worker bounds its response and, over the ceiling, returns a prefix
of the fact list rather than failing. The graph records this; the substitution
used to ignore it.

That was survivable while an answer only ever reported what it held. A symbol
query reports what it *does not* hold, and off a prefix it reported symbols as
absent that the file plainly declares — on `astropy/io/fits/header.py`, every
declaration after roughly line 1,920. A host trusting that skips its read and
loses the declaration.

An answer built from a prefix therefore carries
`structural_graph_truncated_for_path`, alongside `symbol_not_declared_in_path`
when it found nothing. The two together say "this graph never reached it"; the
second alone says "this file does not declare it." A host can act on the
difference, and both are cheaper than a confident wrong answer.

## Verifiability is the security model

Every span carries an independently computed content hash over exactly the bytes
returned. The host holds the source, so it can verify each span itself.

This is what lets a hook be safe without being trusted. Impresari cannot smuggle
content: a returned byte that does not verify against the host's own file is
rejected by arithmetic, not by policy. `SEC-INV-011` already requires exact
source to be hash-and-span attested, and this shape inherits it.

## No new authority

The hook reads the workspace exactly as structural extraction already does,
through the same authorized snapshot. It launches no process, opens no socket,
and writes nothing. `SEC-INV-007` stays literally true, which is the property
[ADR-0126](../decisions/0126-answer-host-executed-operations-without-execution-authority.md)
refused to trade away — a control-plane layer that performs reads on the agent's
behalf would need execution authority, and would exchange the product's most
defensible claim for a commodity one.

The response is an offer. It carries no veto, no approval, and no instruction,
so a host that discards every answer is using the product correctly.

## Accounting is not decoration

Each response reports the bytes the whole file would have cost, the bytes
returned, and the spans omitted with a reason.

Without that, neither the host nor a measurement can tell whether substitution
paid. A host deciding per call needs the ratio in front of it; and this project
has already spent a day optimising a proxy without checking the thing the proxy
stood for. The accounting exists so the next claim about token savings is
arithmetic rather than assertion.

## Bounds

| bound | why |
| --- | --- |
| returned bytes | the host sets it; a substitution larger than the file is not one |
| span count | a generated file with thousands of declarations cannot flood a response |
| one path per offer | keeps the exchange attributable and the failure closed |

Reaching a bound is disclosed. A truncated answer that reads as complete would
be worse than no answer, because the host would stop looking.

One bound is not this hook's to set. The worker's response ceiling comes from
`budget.requested`, which governs every structural build in the product; at its
current 1 MiB it truncates the graph for a large file, and 23.5% of the symbols
measured could not be answered because of it. Raised to 4 MiB in a throwaway
experiment, every one of the 494 answered. That change is not made here — a
budget the whole product shares is not something a read hook should move on its
own — and it is recorded as the follow-up rather than absorbed silently.

## What this cannot do yet

Nothing measures it end to end. The evaluation harness exposes six repository
tools to the agent and no Impresari call at all, so an agent cannot take this
offer even when the product makes it.

The substitution *ratio* is measurable offline today — returned bytes against
whole-file bytes, over the files the corpus actually reads — and the PRD requires
that before any claim. The token saving is not measurable until a harness mode
exists that puts this hook in front of an agent, and that is recorded as the
blocking dependency rather than assumed away.

## Preserved invariants

`SEC-INV-002`, `SEC-INV-003`, `SEC-INV-007`, `SEC-INV-011` and `SEC-INV-012`
hold unchanged. The hook transforms data it is handed and data it is authorized
to read, adds no capability, and cannot be steered by the content of a payload.
