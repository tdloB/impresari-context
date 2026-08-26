# Phase 2 GitHub Copilot CLI connection-kit record

- Status: First-class for the recorded client/version/OS scope, subject to the
  repository's hosted release gate
- Date: 2026-08-26
- Client: GitHub Copilot CLI `1.0.80`, macOS aarch64
- Scope: project `.mcp.json`, isolated workspace trust, and bounded prompt-mode
  packet lifecycle

## Evidence completed

| Requirement | Result | Evidence |
| --- | --- | --- |
| Fixed project configuration | Passed | The versioned managed kit renders and validates a local transport with an absolute executable, fixed workspace, separate cache, consumer ID, role, and exact four-tool allowlist. It rejects environment forwarding and remote fields. |
| Malformed configuration | Passed | `scripts/rehearse-gemini-copilot-preadmission.rb --malformed-copilot-config-only` requires the client to reject malformed disposable configuration before a tool call and verifies the fixture source is unchanged. |
| Native project discovery | Passed | `scripts/rehearse-copilot-native-project.rb` starts from an empty caller-named `/private/tmp` project, installs the owned `.mcp.json` entry, and requires `copilot mcp list --json` plus `copilot mcp get impresari-context --json` to report the project-local executable contract. |
| Isolated folder trust | Passed | Prompt mode natively loaded the workspace server only after its exact workspace path appeared in the isolated `COPILOT_HOME/config.json` `trustedFolders` list. The runner removes only that exact list item afterward and preserves Copilot-generated metadata in the disposable home. |
| Bounded prompt lifecycle | Passed | With no `--additional-mcp-config`, Copilot completed `context_session_open`, `context_build`, `context_packet_resolve`, and `context_session_close` in that order against the native project entry. Built-in MCP, remote control, automatic update, and custom instructions were disabled; the model's available tool set contained only the four named Impresari MCP tools. |
| Packet equivalence | Passed | Each live tool result was successful and structured. The resolved packet exactly equaled Copilot's delivered packet, and that packet exactly equaled an independent raw-MCP control packet from the same fixed fixture, request, timestamp, budget, and launch contract. |
| Source/configuration containment | Passed | The fixture source digest remained unchanged. The runner removed only the owned project MCP entry and the exact temporary trusted-folder value; it never read or changed the user's real Copilot home or source project. |

## Deliberate limits

Copilot's model selects tools conversationally, so the prompt sequence is
bounded live-client smoke evidence—not prompt-repeatability or deterministic
client conformance. The deterministic gates are fixed configuration validation,
native project discovery, exact trust/configuration lifecycle, malformed-input
handling, source immutability, and direct packet comparison.

The rehearsal uses `--allow-all-tools` and `--allow-all-paths` only after
`--available-tools` reduces the model-visible surface to the four named
Impresari MCP tools. It disables built-in MCP, remote control, automatic
update, and custom instructions. It exposes no shell, source read/write, web,
or user-project authority and does not recommend these flags for persistent
user configuration.

The rehearsal is preview-first: it reports the explicit `/private/tmp` root;
`--apply` creates only its isolated home, workspace, and cache; and the native
run touches no default Copilot home. Copilot may retain its own metadata in the
disposable home; Impresari removes only its exact trust entry and owned project
server entry.

## Admission status

The local L1 evidence is complete for GitHub Copilot CLI `1.0.80` on macOS
aarch64: versioned kit, malformed-client rejection, isolated native
project/trust discovery, bounded four-tool lifecycle with direct packet
equivalence, and exact owned-entry/trust removal. The published claim remains
restricted to this client/version/OS scope and requires revalidation after a
Copilot configuration, trust, tool, approval, or result-stream change.

VS Code Copilot is a separate client surface and remains Generic local MCP
until it independently meets the L1 evidence contract.

## Roadmap checkpoint

The Master PRD, Phase 2 PRD, ADR-0018, ADR-0035, and client-integration
roadmap were reassessed. Copilot CLI completes its independent CI-1 L1 target;
the approved language roadmap does not change. Its distinct CI-2 result is
recorded in the [Copilot CLI L2 native-guidance record](phase-2-copilot-cli-native-guidance.md).
The VS Code Copilot extension remains a separate L1 client surface.
