# Codex L2 native-guidance admission

- Status: passed for recorded scope
- Observed: 2026-08-26
- Client: Codex CLI `0.149.0-alpha.4.1`, macOS aarch64
- Governing records: [CI-2 PRD](../product/client-integration-l2-native-guidance-prd.md),
  [ADR-0041](../decisions/0041-native-agent-guidance-artifacts.md), and
  [CI-2 architecture record](../architecture/ci-2-native-guidance-artifacts-ard.md)

## Recorded evidence

An isolated disposable Git project used the exact owned `AGENTS.md` v2
artifact, a one-run local-stdio MCP configuration override, a separate cache,
and a fixed `probe.ts` fixture. The authenticated Codex CLI ran with
`--ephemeral` and `--ignore-user-config`; it opened a bounded Impresari
session and completed an allowed `context_build` for the `orientation` profile
with the hard `4096`-byte budget. The recorded audit event has packet-policy
identity, workspace and snapshot identities, and all enforced limits. No user
Codex configuration, trust setting, source project, or persistent session was
changed.

The v2 guidance records the session lifecycle explicitly:

1. `context_session_open`
2. `context_build`
3. `context_packet_resolve`
4. `context_session_close`

The existing Phase 1 App Server rehearsal remains the deterministic proof for
that complete transport lifecycle and direct packet equivalence. The live L2
smoke establishes that Codex can use the owned project-native artifact together
with the already configured local MCP server; it is not a second transport
conformance harness.

## Contract correction observed during admission

Codex emitted nonnegative integer JSON values for the declared string budget
fields. The MCP adapter now accepts only those integral wire aliases and
canonicalizes them to the core's decimal-string representation before core
validation. Floats, signed values, unknown fields, and invalid policy limits
remain rejected. The core resource policy remains authoritative, so this is a
strict transport-compatibility correction rather than a budget expansion.

The current v2 guidance uses live schema validation rather than duplicating
mutable identifier or policy values. Its render/inspect/validate/remove
lifecycle remains deterministic. Exact owned v1 guidance is identified as
`owned_legacy` solely so it can be safely removed; it cannot validate as v2.

## Limits

On this recorded Codex App Server version, `thread/start` returned no
instruction-source paths for the disposable project. Therefore this record
does **not** claim a separate deterministic signal that the project
instructions were discovered, nor does it claim that a repeated conversational
prompt will choose the same tools. It also does not alter Codex approval,
trust, source-write, execution, network, delivery, memory, or orchestration
authority, and it does not make automatic packet delivery an L2 capability.

Revalidate this record if Codex changes its CLI, App Server, project-instruction
surface, MCP configuration behavior, or platform support.
