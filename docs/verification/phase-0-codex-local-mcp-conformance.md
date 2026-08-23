# Phase 0 Codex local MCP conformance record

- Date: 2026-08-23
- Client: Codex CLI `0.149.0-alpha.4.1`
- Classification effect: limited evidence for **Generic local MCP** only; this
  record does not admit Codex as a first-class client.
- Transport: registered local stdio child process with a fixed empty temporary
  workspace, a distinct dedicated temporary cache, a fixed consumer identity,
  and the `local_user` role.

## Result

Passed after correcting two interoperability defects in the local MCP server:

1. Initialization now negotiates the client-requested supported revision
   `2025-06-18` as well as the preferred `2025-11-25` revision.
2. `tools/call` now accepts the standard inert MCP `_meta` object while
   continuing to reject every other unknown envelope field.

The real Codex transport completed these calls in order:

| Operation | Result | Authority outcome |
| --- | --- | --- |
| MCP initialization and tool discovery | Passed | No additional authority |
| `context_session_open` | Structured result, `opened: true` | No additional authority |
| `context_build` with a bounded literal probe | Structured complete packet and session reference | No filesystem or orchestration authority added |
| `context_session_close` | Structured result, `closed: true` | No additional authority |

The temporary workspace remained empty. The literal probe returned no source
evidence and the returned packet explicitly recorded that absence; this was the
expected source-free conformance outcome.

## Deliberate limits

This is not first-class admission evidence. It does not yet prove a maintained
client version range or OS matrix, clean-install workflow, packet equivalence
against the frozen direct-engine corpus through Codex, malformed configuration
handling, project/user configuration scope, or entry-specific removal behavior.
Those requirements remain governed by the [Phase 0 PRD](../product/phase-0-language-and-client-foundation-prd.md)
and [ADR-0018](../decisions/0018-first-class-client-integration-and-compatibility-contract.md).
