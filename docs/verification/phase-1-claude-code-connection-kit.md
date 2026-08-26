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
| Client availability and authenticated health | Passed | `claude --version`, `claude auth status`, and `claude doctor` recorded a healthy native Claude Code `2.1.241` installation, first-party authentication, and macOS ARM64 platform. |
| Fixed local-stdio configuration | Passed | The versioned managed kit renders and validates the fixed executable local-stdio command, workspace, separate cache, consumer ID, and role without environment forwarding. |
| Real temporary-config lifecycle | Passed | `scripts/rehearse-claude-code.rb` explicitly uses managed install, validate, and exact removal against a disposable `--mcp-config`, starts Claude with `--strict-mcp-config`, makes built-in tools unavailable, and preapproves only the four fixed MCP operations. |
| Complete operation sequence | Passed | Claude called `context_session_open`, `context_build`, `context_packet_resolve`, and `context_session_close` in the required order; every tool result was present and non-error. |
| Source and configuration containment | Passed | The fixture source digest remained unchanged; the temporary managed configuration target was removed after the run; and `claude mcp get impresari-context` confirmed no persistent registration was created. |
| Client-rendered malformed configuration | Passed | `scripts/rehearse-claude-code.rb --malformed-config-only` gives Claude Code a malformed disposable MCP configuration under `--strict-mcp-config`. The client rejects it before any model call or MCP startup, and the fixture source digest remains unchanged. |

## Deliberate limits

Claude Code's exposed client path is model-directed. Restricting built-in tools
and preapproving the exact MCP tools makes a bounded real-client lifecycle
rehearsal possible, but does not make model tool selection deterministic. This
evidence is therefore not a substitute for a direct client RPC conformance
surface like Codex App Server.

The rehearsal uses the authenticated user's ordinary Claude Code account and
makes a bounded model request. It is an opt-in local maintenance check, not a
hosted CI test and not a persistent configuration installer.

## Admission status

Do not promote Claude Code to **First-class** yet. Still required:

- direct-engine versus Claude-delivered packet-equivalence evidence;
- version/OS matrix beyond the exercised macOS aarch64 client; and
- verified entry-specific removal behavior in local, project, and user scopes.

Claude Code's model-directed tool selection is recorded as live-client smoke
evidence, not as a repeatable client-conformance requirement. The deterministic
gates remain the managed connection contract, fixed authority, validation,
malformed-input handling, packet evidence where observable, and exact removal.

These requirements remain governed by the [Phase 1 PRD](../product/phase-1-language-configuration-and-client-admission-prd.md)
and [ADR-0018](../decisions/0018-first-class-client-integration-and-compatibility-contract.md).

## Roadmap checkpoint

The Master PRD, Phase 1 PRD, ADR-0018, and revised roadmap were reassessed.
No roadmap or authority-boundary change is warranted. The real lifecycle closes
the prior “CLI not installed/authenticated” blocker, but the generic
classification remains accurate. Cursor's user-owned sign-in remains the next
manual Phase 1 admission boundary.
