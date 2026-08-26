# Phase 1 Claude Code connection-kit record

- Date: 2026-08-25
- Client surface: Claude Code CLI `2.1.241`
- OS/architecture scope exercised: macOS aarch64
- Classification effect: **Generic local MCP** remains the published claim.
- Scope: local-stdio guide, read-only configuration validator, and temporary
  real-client MCP lifecycle rehearsal.

## Evidence completed

| Requirement | Result | Evidence |
| --- | --- | --- |
| Client availability and authenticated host path | Passed | Claude Code CLI `2.1.241` on macOS ARM64 completed the bounded host-side model request used by the disposable rehearsal. The sandbox cannot access the client credential store; the host result is the admission evidence. |
| Fixed local-stdio configuration | Passed | The versioned managed kit renders and validates the fixed executable local-stdio command, workspace, separate cache, consumer ID, and role without environment forwarding. |
| Real temporary-config lifecycle | Passed | `scripts/rehearse-claude-code.rb` explicitly uses managed install, validate, and exact removal against a disposable `--mcp-config`, starts Claude with `--strict-mcp-config`, makes built-in tools unavailable, and permits only the minimal four-tool session/packet lifecycle. |
| Complete operation sequence and packet equivalence | Passed | Claude called `context_session_open`, `context_build`, `context_packet_resolve`, and `context_session_close` in order. The rehearsal requires each non-error tool result, requires Claude's resolved packet to equal its delivered packet, and proves that packet exactly equals an independent direct MCP packet from the same fixture. |
| Source and configuration containment | Passed | The fixture source digest remained unchanged; the temporary managed configuration target was removed after the run; and `claude mcp get impresari-context` confirmed no persistent registration was created. |
| Client-rendered malformed configuration | Passed | `scripts/rehearse-claude-code.rb --malformed-config-only` gives Claude Code a malformed disposable MCP configuration under `--strict-mcp-config`. The client rejects it before any model call or MCP startup, and the fixture source digest remains unchanged. |

## Deliberate limits

Claude Code's exposed client path is model-directed. Restricting built-in tools
and preapproving the exact MCP tools makes a bounded real-client lifecycle
rehearsal possible, but does not make model tool selection deterministic. This
evidence is therefore not a substitute for a direct client RPC conformance
surface like Codex App Server.

The rehearsal requires an authenticated user's ordinary Claude Code account and
makes a bounded model request only after strict temporary configuration is
accepted. It is an opt-in local maintenance check, not a hosted CI test and
not a persistent configuration installer. Client credential access is not
available inside the restricted test sandbox, so the host execution is the
authoritative live-client record.

The rehearsal also offers a preview-first `--prepare-project-root` mode. It
creates only an empty disposable `workspace` and separate `cache` under
`/private/tmp`; it neither creates nor removes Claude Code configuration. That
keeps the required local-scope registration and exact removal decision fully
user-owned.

## Admission status

Do not promote Claude Code to **First-class** yet. Still required:

- verified user-reviewed local-scope installation and entry-specific removal;
  and
- a maintained published version/OS scope if the recorded `2.1.241` macOS
  aarch64 scope changes.

Claude Code's model-directed tool selection is recorded as live-client smoke
evidence, not as a repeatable client-conformance requirement. The deterministic
gates remain the managed connection contract, fixed authority, validation,
malformed-input handling, packet evidence where observable, and exact removal.

These requirements remain governed by the [Phase 1 PRD](../product/phase-1-language-configuration-and-client-admission-prd.md)
and [ADR-0018](../decisions/0018-first-class-client-integration-and-compatibility-contract.md).

## Roadmap checkpoint

The Master PRD, Phase 1 PRD, ADR-0018, and revised roadmap were reassessed.
The host-side live lifecycle and direct-packet comparison close the temporary
configuration evidence gap. The generic classification remains accurate until
the user-reviewed local-scope install/remove record exists. Cursor's user-owned
enablement remains the next distinct Phase 1 admission boundary.
