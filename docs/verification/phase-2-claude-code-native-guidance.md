# Claude Code L2 native-guidance admission

- Status: passed for recorded scope
- Observed: 2026-08-26
- Client: Claude Code CLI `2.1.241`, macOS aarch64
- Governing records: [CI-2 PRD](../product/client-integration-l2-native-guidance-prd.md),
  [ADR-0041](../decisions/0041-native-agent-guidance-artifacts.md), and
  [CI-2 architecture record](../architecture/ci-2-native-guidance-artifacts-ard.md)

## Recorded evidence

An isolated temporary project used the exact owned project skill at
`.claude/skills/impresari-context/SKILL.md`, a separate temporary MCP JSON
configuration, cache, and a fixed `probe.ts` fixture. Claude Code reported the
`impresari-context` skill and slash command as loaded, connected only to the
configured local MCP server, then model-directed the bounded four-tool
lifecycle:

1. `context_session_open`
2. `context_build` with the `orientation` profile and a hard budget
3. `context_packet_resolve`
4. `context_session_close`

The delivered packet and resolved packet were identical. The completed packet
was current and complete, carried packet/plan identity, coverage, and explicit
unknowns, and added neither filesystem nor orchestration authority. The
rehearsal removed the exact owned skill and MCP entry afterward; the source
fixture remained byte-identical.

The reusable command is:

```text
ruby scripts/rehearse-claude-code.rb --native-guidance-smoke
```

It requires an already authenticated Claude Code CLI. It uses a disposable
temporary project, strict temporary MCP configuration, no session persistence,
and an allowlist containing only the four Impresari lifecycle tools. It does
not create a persistent MCP registration. Client-controlled session state is
outside Impresari's authority and is not a product claim.

## Contract correction observed during admission

The original MCP tool schema described `request_id`, `event_id`, and
`policy_profile` only as arbitrary strings. That invited a conversational
client to form inputs that the core correctly rejected. The current schema now
exposes the v1 identifier pattern and fixed policy-profile fingerprint. These
constraints live in the live tool schema rather than in the exact-owned v1
skill, preserving safe removal of existing owned artifacts.

## Limits

This is a client-specific live smoke record, not proof that the same prompt
always leads to the same model tool calls. It does not alter Claude approval,
trust, source-write, execution, network, delivery, memory, or orchestration
authority; it does not make automatic packet delivery an L2 capability.
