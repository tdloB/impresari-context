# Phase 3 deterministic context planner evidence

- Status: Initial implementation in progress
- Governing records: [Phase 3 PRD](../product/phase-3-deterministic-context-planner-prd.md) and [ADR-0024](../decisions/0024-deterministic-context-planner.md)

## Implemented evidence

`TaskProfile` accepts the seven founder-approved profiles. A profile build is
bound to the engine's current snapshot and the same policy decision used for
packet construction. It returns a canonical `deterministic-context-plan` with
ordered retrieval steps, per-step reason codes, a complete availability matrix,
and explicit profile omissions. The companion `profiled-context-packet` returns
the immutable packet and reports any evidence count omitted by the hard packet
budget.

The CLI provides `context profile-build`; MCP retains the existing
`context_build` tool and accepts either its previous explicit `steps` form or a
mutually exclusive `profile` plus `query` form. No new MCP tool, client
configuration, model call, background process, filesystem write to source, or
authority is introduced.

## Verification

- Engine tests prove identical declared inputs yield identical plan and packet
  identities, with unavailable change-set evidence visible.
- CLI tests prove profile output is machine readable and the source workspace
  remains unchanged.
- MCP tests prove profile output includes both plan and packet and returns
  `false` for orchestration and filesystem authority additions.
- The schema registry contains `deterministic-context-plan.schema.json`.

## Deliberate limits

The initial planner selects only existing exact-path, filename, literal, and
lexical retrieval. It reports structural relationships, change sets, associated
tests, and configuration-to-code references as unavailable. Those classes are
not inferred from parser availability and require individual future admission.
