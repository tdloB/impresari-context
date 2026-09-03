# Task-Recall-First Context Selection PRD

## Document Control

- PRD ID/version: IC-TRFC-125 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-03.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Task-Recall-First Context Selection ARD](../architecture/task-recall-first-context-selection-ard.md).
- Governing decision:
  [ADR-0125](../decisions/0125-select-ranked-seed-sets-and-traverse-to-definitions.md).
- Governing objective: [CLAUDE.md](../../CLAUDE.md) — 98% quality at 78% compression.

## Problem

Structural selection returns confident, well-attributed, wrong context.

Measured on `astropy__astropy-13033` with an identical task and budget, the
disclosure map returned sixteen items. Every item was in
`astropy/timeseries/sampled.py`. The change the task actually requires is
entirely in `astropy/timeseries/core.py`, in `BaseTimeSeries._check_required_columns`.
Recall on the file that mattered was zero, and neither the target file nor the
target symbol appeared anywhere in the map.

The evidence packet did contain `core.py`. Selection, not retrieval, failed.

Three defects produced this:

1. **One seed.** Exactly one start node is chosen, so the map can only describe
   one neighbourhood. A task whose answer spans a subclass and its parent cannot
   be covered by one anchor.
2. **Ambiguity abandons.** When several nodes match a signal, selection returns
   `structural_seed_ambiguous` and yields nothing. Multiple matches are
   information about where to look, not a reason to stop.
3. **No traversal to definitions.** The task names `TimeSeries`. The graph holds
   `TimeSeries` in `sampled.py` as a reference. Its declaration, and the
   supertype `BaseTimeSeries` that carries the defect, are one and two edges
   away and were never reached.

A quality claim cannot be made until this is measured. There is currently no
mechanism in the repository that scores whether delivered context contains what
a task needed.

## Product Outcome

Selection produces a bounded, ranked **set** of structural seeds rather than a
single node, resolves ambiguity by ranking rather than abandonment, and reaches
the declaration a reference names and the supertypes that declaration extends.

Quality becomes measurable offline. A scoring tool compares delivered context
against a reference change for a corpus of tasks and reports task-relative
recall, with no model call, no network, and no provider cost.

## Functional Requirements

1. Select up to a fixed maximum of ranked structural seeds instead of exactly
   one. The maximum is a closed constant, not caller-supplied.
2. Rank candidates by a total, deterministic order: exact symbol inside an exact
   task path, then exact task file path, then globally unique exact symbol, then
   globally ambiguous exact symbol. Break remaining ties by portable path, then
   by node identity.
3. Admit an ambiguous class rather than discarding it. Retain a bounded number
   of its members in rank order and record the ambiguity as an explicit
   disclosure reason. Yield no seed only when no signal matches any node.
4. From an admitted reference node, reach the declaration it names. From a
   declaration, reach the supertypes it declares. Both stay inside the existing
   bounded traversal depth and resource budgets.
5. Preserve determinism. Identical workspace snapshot, task text, budget, and
   policy must produce an identical map identity.
6. Preserve every existing bound. This requirement adds no new resource
   authority and raises no ceiling.
7. Record every recall-affecting limit as an explicit omission or unknown. A map
   that is partial because a budget bound, and a map that is partial because
   nothing matched, must be distinguishable.
8. Provide an offline scoring tool that, for a corpus of tasks with reference
   changes, reports per task: reference files, reference symbols, files present
   in the delivered context, symbols present, file recall, symbol recall, and
   delivered bytes. It performs no model call and opens no network socket.
9. The scoring tool treats the product as a black box. Reference changes must
   not be readable by the engine, must not enter any product input, and must not
   influence selection. Oracle isolation is a hard validity gate.

## Acceptance Criteria

- On `astropy__astropy-13033`, the delivered context contains
  `astropy/timeseries/core.py` and the symbol `_check_required_columns`.
- The scoring tool runs a corpus offline and emits per-task and aggregate
  recall with no provider credential present in its environment.
- A static check proves the engine has no path to reference-change data.
- Repeating a build with identical inputs yields an identical map identity.
- An ambiguous signal yields ranked seeds plus an explicit ambiguity reason,
  not an empty map.
- The full repository gate passes, including adversarial and fuzz suites.

## Non-Goals

- Any agent loop, provider request, or paid evaluation.
- Hook integration, which is [IC-HEH-126](host-executed-context-hooks-prd.md).
- Cache stability, which is [IC-CSCP-127](cache-stable-context-prefix-prd.md).
- Semantic ranking by a model. Ranking stays lexical and structural.
- Raising any resource ceiling.
