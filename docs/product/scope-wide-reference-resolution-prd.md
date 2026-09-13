# Scope-Wide Reference Resolution PRD

## Document Control

- PRD ID/version: IC-SWRR-132 / 1.0.
- Status: Implemented and measured; file criterion not met, symbol gain substantial.
- Date: 2026-09-04.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Scope-Wide Reference Resolution ARD](../architecture/scope-wide-reference-resolution-ard.md).
- Governing decision:
  [ADR-0132](../decisions/0132-resolve-references-across-the-scope-not-within-a-file.md).
- Governing objective: [CLAUDE.md](../../CLAUDE.md) — 98% quality at 78% compression.

## Problem

A delivered map spans one to seven files, and the reference file a task needs is
frequently a package sibling of a file the map already holds. `astropy-8707`
returns `io/fits/header.py` and never reaches `io/fits/card.py`, though `Header`
references `Card` throughout.

The cause is not a budget. A `calls` or `references` edge resolves its target
against `file.response.facts` — the declarations of **the file the edge starts
in**. A reference from `Header` to `Card` finds no `Card` declaration inside
`header.py`, so the edge is emitted with no target and marked `unresolved`.
Traversal refuses to follow an unresolved edge, correctly, because it must never
invent a target.

The structural graph is therefore a set of **per-file symbol islands**, joined
only by file-to-file `imports` edges for relative modules. `unresolved_traversal_target`
is disclosed in 20 of 21 delivered maps.

Budget cannot reach across an island. Doubling traversal depth was measured as a
control: map file recall 14 of 27 before and 14 of 27 after, **not one task
changed**, the same 68 distinct files corpus-wide, for 1.1% more delivered
bytes. Every other limit is saturated too — depth in 19 of 21 tasks, edges in 16
of 21, seeds in 17 of 21 — and raising any of them walks further through a graph
whose cross-file edges all dead-end.

## Product Outcome

A `calls` or `references` edge resolves to the declaration it names anywhere in
the admitted scope, not only inside its own file, so traversal can cross from a
file to the file it depends on.

Resolution stays honest. A target is resolved only when the scope holds exactly
one declaration of that name; anything else stays unresolved and is disclosed.

## Functional Requirements

1. Resolve a `calls` or `references` target against the declarations of every
   file in the admitted scope, not only the originating file.
2. Prefer a declaration in the originating file when one exists. An edge that
   resolves today must resolve to the same target after this change.
3. Resolve across files only when the scope holds exactly **one** declaration of
   that name. Never choose among several, and never invent a target.
4. Disclose an ambiguous name distinctly from an absent one, so a consumer can
   tell "the scope declares no such name" from "the scope declares several".
5. Keep the resolution vocabulary unchanged. A name match is `heuristic`
   whichever file it lands in, because it is a name match either way and not a
   language-aware resolution.
6. Stay deterministic. An identical graph yields identical edges, in a stable
   order.
7. Add no repository read. Resolution uses facts the builder already holds.
8. Never consult a reference change, accepted patch, or test outcome.

## Acceptance Criteria

- A reference to a name declared once elsewhere in scope resolves to that
  declaration; traversal reaches it.
- A reference to a name declared in several scope files stays unresolved and
  discloses ambiguity.
- A reference to a name declared nowhere in scope stays unresolved exactly as
  today.
- An edge that resolves within its own file today resolves identically after.
- Repeating a build over identical inputs yields an identical graph.
- A static check proves the module reaches no oracle, execution, or network
  surface.
- **Measured, offline, over all twenty-two astropy tasks:** map file recall
  improves by at least two reference files against the 14 of 27 baseline. Any
  reference file lost is reported with its cause, not netted away.
- The full repository gate passes.

## Measured Outcome

Measured twice, because the first verdict was formed against a scope that was
mostly noise.

**Against `main` as it stood:** map file recall 14 of 27 before and after, symbol
recall 10 to 12, for 1.3% more bytes. Delivered maps widened from 68 to 77
distinct files, and `traversal_edge_limit_reached` rose from 16 of 21 tasks to
20 of 21 — the cross-file edges were real and being followed, but they were
being followed through a scope in which twelve of sixteen identifiers were prose
([IC-NSP-133](nomination-signal-precision-prd.md)).

**On the corrected scope, stacked on IC-NSP-133:**

| | map files | map symbols | distinct map files | bytes |
| --- | --- | --- | --- | --- |
| IC-NSP-133 alone | 18 of 27 | 11 of 34 | 77 | 19,416,100 |
| plus this change | 18 of 27 | **12 of 34** | 86 | 19,412,024 |

One reference symbol gained, none lost, and no additional reference file.

**These figures are a correction.** This change was first reported as gaining a
reference file and four symbols. That run also carried an uncommitted
traversal-depth change from an unrelated experiment; on restored settings the
depth accounts for the file and three of the four symbols.

### The criterion that was not met

This PRD requires two reference files against its baseline. Measured on restored
settings the change contributes **none**, and one reference symbol, which is
inside the noise band a twenty-two task corpus supports.

The change is kept for the reason the ADR gives — a graph whose cross-file edges
can never resolve misrepresents its own structure — and not because it earned a
recall bar. It did not.

## Non-Goals

- A language-aware resolver. Import aliasing, scoping, shadowing and dynamic
  dispatch are out; this is name matching over an admitted scope.
- Resolving beyond the scope. A name declared only in an unadmitted file stays
  unresolved; widening the scope is a nomination concern.
- Changing traversal budgets. Depth 2 was measured and bought nothing; budgets
  are revisited only after edges resolve.
- Changing the within-file rule. `calls` takes the first match and `references`
  requires a unique one; that inconsistency is recorded in the ARD and left
  alone so this change stays additive.
