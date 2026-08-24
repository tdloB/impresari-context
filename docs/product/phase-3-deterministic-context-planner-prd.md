# Impresari Context — Phase 3: Deterministic Context Planner PRD

- Status: Approved; initial implementation in progress
- Date: 2026-08-23
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)

## Objective

Add a small deterministic intelligence layer that selects evidence through
explicit rules and preserves Impresari Context's client-neutral authority model.

## Inputs

- Declared task profile and query.
- Exact workspace snapshot.
- Policy, budget, and supported structural-evidence inventory.

## Outputs

- Explicit retrieval plan and exact packet identity.
- Reason code for every selected item.
- Coverage report, omitted candidates, and budget-exclusion reasons.

## Initial profiles

`orientation`, `implementation`, `bug_investigation`, `change_review`,
`security_review`, `test_selection`, and `configuration_change`.

## Non-goals

- Agent governance, task routing, model calls, execution, approvals, durable
  memory, hidden scoring, or language support that has not passed admission.

## Acceptance criteria

- Equivalent declared inputs produce the same plan, reasons, omissions, and
  packet identity.
- Unsupported evidence classes are reported explicitly rather than inferred.
- Every selected or omitted candidate remains recoverable to exact evidence or
  a stable rule and budget reason.

## Initial delivery boundary

The initial implementation exposes all seven declared profiles through the
core engine, CLI, and existing `context_build` MCP tool. Each build returns a
canonical plan identity, ordered reason-coded steps, evidence-class coverage,
profile omissions, and the exact packet identity. The planner deliberately
uses only the existing bounded exact-path, filename, literal, and lexical
retrievers. Structural relationship, change-set, associated-test, and
configuration-to-code evidence are reported as unavailable until a separately
admitted planner adapter can supply them.
