# Startup Usage Guidance PRD

## Document Control

- PRD ID/version: IC-SUG-157 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-12.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Startup Usage Guidance ARD](../architecture/startup-usage-guidance-ard.md).
- Governing decision:
  [ADR-0157](../decisions/0157-tell-a-client-when-to-use-each-tool-at-startup.md).
- Follows: [ADR-0154](../decisions/0154-look-through-local-variables-when-building-a-map.md).

## Problem

The startup instructions named no tool. An agent that does not know the map and
expansion tools reads whole files, and the product's evidence is added to that
reading instead of substituting for it.

## Product Outcome

Every MCP client learns at startup, in a few sentences, how to begin a task with
Impresari Context and when each tool helps, with no claim of control.

## Functional Requirements

1. The initialize response's `instructions` names every tool the server
   advertises and says when to use it.
2. It is under 1,000 characters.
3. It states that results add no orchestration, approval, execution, or
   filesystem authority, and that the client remains free to read files
   directly.
4. It changes no tool, schema, or result.

## Acceptance Criteria

- A test fails when a tool is left unnamed, when the authority statement is
  dropped, or when the text reaches 1,000 characters.
- The full repository gate passes.
- Whether agents substitute map lookups for file reads is measured by a graded
  run, not by this change.

## Non-Goals

- Per-client guidance templates, hooks, or any client configuration.
- Changing tool descriptions or schemas.
