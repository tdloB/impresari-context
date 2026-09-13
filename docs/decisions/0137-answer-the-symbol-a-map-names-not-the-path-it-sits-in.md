# ADR-0137: Answer the Symbol a Map Names, Not the Path It Sits In

- Status: Accepted
- Date: 2026-09-06
- Related PRD: [Host Read Substitution](../product/host-read-substitution-prd.md)
- Architecture: [Host Read Substitution](../architecture/host-read-substitution-ard.md)
- Amends: [ADR-0136](0136-substitute-a-host-read-with-declaration-spans.md)

## Context

ADR-0136 decided to answer a host's read offer with the named file's declaration
spans. It shipped, and the substitution ratio it required before any claim of
saving has now been measured through the MCP tool against eight astropy files.

A whole-path answer returns **92.6%** of the source. It is not a substitution.

The cause is structural, not a defect. Python is declaration-dense: a module is
its classes and functions, so returning every declaration returns the file minus
its imports and module docstring. The same measurement gave 91% on TypeScript and
63% on Rust, and the Rust figure is flattered by dropping `///` doc comments,
which is a quality cost the ratio hides rather than a saving. No language makes a
whole-path answer small enough to be worth taking.

The unit was wrong. A map does not point at a file; it points at
`_check_required_columns` in `core.py`. ADR-0136's own architecture record says
that offer "is answered exactly by that method's span" — the implementation
returned all of them.

Measuring the symbol instead, over 494 declarations in the same eight files: the
median answer is **0.7% of the file**, p90 4.1%, mean 2.9%.

Two findings qualify that number, and both are recorded rather than smoothed
over.

**The ratio depends on what the map names.** The maximum observed was 98.3% — a
top-level class whose span is the file. Naming a class in a single-class module
returns the module. The saving comes from naming a *method*, which is what a map
that resolves to a symbol actually does.

**A truncated graph cannot support a claim of absence.** The structural worker
bounds its response by `budget.requested` and, over the ceiling, returns a prefix
of the fact list rather than failing. On `astropy/io/fits/header.py` that dropped
every declaration after roughly line 1,920, including seven whole top-level
classes, from a graph reporting no shortfall of its own. Whole-path substitution
never had to notice: it returns what it holds. A symbol query must answer
"is this symbol here?", and off a prefix it answered "no" for symbols the file
plainly declares. A host trusting that would skip its read and lose the
declaration entirely — the product silently causing the failure it exists to
prevent.

Of 494 declarations, 378 were answered. Every one of the 116 misses fell in a
file whose graph was truncated; each of the four files with a complete graph
answered every symbol it declared (7/7, 10/10, 37/37, 43/43).

## Decision

Accept an optional symbol name on a read offer and answer that declaration alone.

Do not collapse nesting for a named symbol. The whole value of naming one is
reaching a method inside a class, and collapsing to the enclosing declaration
returns the class — the whole-path answer again. Return every declaration
carrying the name, in source order, since a file may declare a name more than
once.

Disclose a truncated graph as `structural_graph_truncated_for_path`, on every
answer built from one, whether or not it found the symbol. An empty answer off a
prefix reports both that unknown and `symbol_not_declared_in_path`, so a host can
tell "this file has no such symbol" from "this graph never reached it."

Keep the whole-path answer. It is the right shape for a host with no symbol, and
its accounting now reports honestly that it is not a saving on a
declaration-dense language.

Do not raise the worker's response ceiling here. Restoring the 116 missing
declarations needs `budget.requested` above 1 MiB, and that budget governs every
structural build in the product, not this hook. It is a separate decision with
its own memory and latency surface.

## Consequences

The substitution becomes worth taking. A host following a map pointer spends
roughly 0.7% of the file rather than 92.6% of it, and the PRD's acceptance
criterion — returned bytes materially smaller than whole-file bytes — is met by
the symbol form and explicitly *not* met by the path form.

The product now declines to claim knowledge it does not have. That costs
coverage: 23.5% of symbols in this sample answer "truncated" instead of returning
bytes, and a host receiving that unknown should read the file. This is a real
reduction in what substitution can do today, taken deliberately over the
alternative of a confident wrong answer.

The ceiling that causes it is now visible and quantified. Raised to 4 MiB in a
throwaway experiment, all 494 symbols answered and no file truncated; that
experiment was reverted rather than merged, and the follow-up is recorded.

Naming a symbol that does not exist is now distinguishable from naming a path
with no declarations, which the previous single `no_declarations_for_path`
category could not express.

No security invariant changes. Selection narrows; nothing new is returned. Every
span still carries an independently computed hash over exactly the bytes
returned, and a symbol name is consumer input that narrows an answer and cannot
broaden one (`SEC-INV-012`). This record grants no execution, network,
publication, or submission authority.

## Still not claimed

A token saving. The evaluation harness still offers an agent six repository tools
and no Impresari call, so no agent can take this offer. The ratio is an offline
property of the answer; whether it makes agents read less remains the blocking
dependency ADR-0136 recorded, and this record does not discharge it.
