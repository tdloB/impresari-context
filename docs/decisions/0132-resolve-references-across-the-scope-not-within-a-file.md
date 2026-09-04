# ADR-0132: Resolve References Across the Scope, Not Within a File

- Status: Accepted; lands after ADR-0133, which its measurement depends on
- Date: 2026-09-04
- Related PRD: [Scope-Wide Reference Resolution](../product/scope-wide-reference-resolution-prd.md)
- Architecture: [Scope-Wide Reference Resolution](../architecture/scope-wide-reference-resolution-ard.md)
- Extends: [ADR-0125](0125-select-ranked-seed-sets-and-traverse-to-definitions.md)

## Context

ADR-0125 extended bounded traversal so a reference reaches the declaration it
names and a declaration reaches its supertypes. It does, within one file.

`add_call` and `add_reference` resolve a target against `file.response.facts` —
the declarations of the file the edge starts in. A reference from `Header` to
`Card` finds no `Card` inside `header.py`, so the edge is emitted with no target
and marked `unresolved`, and traversal will not follow it.

The structural graph is consequently a set of per-file symbol islands joined
only by file-to-file `imports` edges for relative modules. Delivered maps span
one to seven files, and `unresolved_traversal_target` is disclosed in 20 of 21
of them.

This was mistaken for a budget problem. Every limit saturates at once — depth in
19 of 21 tasks, edges in 16, seeds in 17 — which reads as a system that needs
more room. Doubling traversal depth was run as a control and moved nothing: map
file recall 14 of 27 before and after, not one task changed, the same 68
distinct files corpus-wide, for 1.1% more delivered bytes. Five saturated limits
were one structural problem reported five ways.

## Decision

Resolve a `calls` or `references` target against the declarations of every file
in the admitted scope, not only the originating file.

Prefer a declaration in the originating file, so every edge that resolves today
resolves to the same target after. Across files, resolve only when the scope
holds exactly one declaration of that name; disclose ambiguity under its own
reason code and never choose among several. Keep `heuristic` as the resolution
label, because a name match is a name match wherever it lands.

Do not raise traversal budgets. Depth 2 was measured and bought nothing;
budgets are a question to revisit once edges resolve, not before.

## Consequences

Traversal can cross from a file to the file it depends on, which is the
precondition for a map spanning the files a change actually touches. Whether it
does so usefully is a measurement this decision requires rather than asserts:
the PRD sets two reference files against a 14 of 27 baseline, and any file lost
is to be reported with its cause rather than netted away.

Preferring the originating file keeps the change additive. It converts dead ends
into edges and never redirects a live one, so recall movement is attributable to
newly resolved edges rather than to silently rewired old ones.

Refusing ambiguity keeps the graph honest at some cost in reach. `__init__`,
`get` and `read` are declared in dozens of scope files and will stay unresolved.
Choosing among them would be inventing a target with extra steps, which is the
property `SEC-INV-011` and ADR-0125's traversal rule both depend on.

This is name matching over an admitted scope, not a resolver. Import aliasing,
scoping, shadowing and dynamic dispatch are all unhandled, and the `heuristic`
label says so. A future language-aware resolver would supersede this; nothing
here forecloses it.

The known inconsistency between `add_call` taking the first within-file match
and `add_reference` requiring a unique one is left in place. Uniqueness is the
better rule and the cross-file path uses it, but tightening the within-file path
would change edges that resolve today and make this change's measurement
unattributable.

No security invariant changes. Resolution consumes facts the builder already
holds, adds no repository read, and this record grants no execution, network,
publication, or submission authority.

## Measured outcome

Measured against `main` as it stood, this changed no reference file at all: 14
of 27 before and after, symbols 10 to 12. The cross-file edges were live —
delivered maps widened from 68 to 77 distinct files and the traversal edge limit
began biting in 20 of 21 tasks rather than 16 — but they were being followed
through a scope in which twelve of sixteen admitted identifiers were prose.

Re-measured on the scope
[ADR-0133](0133-admit-a-bare-declared-word-only-as-a-type-or-marked-code.md)
corrects, the same change gains a reference file and four reference symbols: 18
of 27 to 19 of 27, and 11 of 34 to 15 of 34, for 1.1% more bytes.

One file is inside this corpus's noise band and the PRD's two-file criterion is
recorded as unmet. Four symbols is not: it is the measure that moves when a map
reaches the right declarations rather than merely the right files, which is what
resolving a cross-file edge does.

The sequencing matters more than either number. A selection change measured
against a noisy scope will read as neutral whatever its merit, because the noise
dominates what selection has to work with. This decision was judged neutral once
on exactly that basis, and the judgement was wrong.
