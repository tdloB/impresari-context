# CI-1 VS Code Copilot admission PRD

- Status: Approved implementation increment; public L1 admission pending
- Date: 2026-08-26
- Roadmap: [Phase 2](phase-2-infrastructure-language-and-agent-expansion-prd.md) and the [Client Integration Depth Roadmap](client-integration-roadmap.md)

## Objective

Replace the VS Code Copilot generic preadmission guide with a candidate
first-class managed-connection path only when the visible VS Code extension
host has demonstrated its documented workspace configuration, explicit trust,
server discovery, bounded tool use, source immutability, and exact removal.

## Scope

- Pin the candidate scope to VS Code `1.134.0` on macOS arm64.
- Render and validate only workspace `.vscode/mcp.json`, the configuration
  that the VS Code extension host reads for its MCP management UI.
- Keep workspace-root `.mcp.json` as a separate Agent Host compatibility
  surface; it has no L1 admission claim from this increment.
- Require a direct `stdio` server entry with the fixed local Impresari MCP
  binary, authorized workspace, separate cache, consumer ID, and role.
- Add a disposable `/private/tmp` rehearsal that prepares and later removes
  only its own configuration after visible signed-in operator evidence.

## Non-goals

- No user-profile or default VS Code configuration writes.
- No automatic trust, enablement, server startup, sign-in, chat submission,
  approval decision, settings sync, sandboxing, or Agent Host service.
- No L2 guidance, L3 delivery, provider proxy, source mutation, repository
  command execution, network authority, or claim about model repeatability.

## Acceptance criteria

1. The configuration validator accepts only `type: "stdio"`, absolute command,
   and the fixed static argument contract; missing/wrong type, environment,
   remote, input, sandbox, and unrecognized fields fail closed.
2. The rendered candidate uses workspace `.vscode/mcp.json`, the surface used
   by the visible `MCP: List Servers` extension-host UI.
3. The rehearsal requires an explicit disposable-root preview and `--apply`,
   proves source immutability, and removes only the exact owned entry.
4. A signed-in operator records VS Code's exact version, reviewed trust choice,
   MCP server discovery, and one visible Impresari tool invocation in the
   disposable workspace.
5. The complete repository gate and hosted CI pass before any public matrix
   change from generic MCP to L1.

## Admission boundary

The code and rehearsal are preparation, not promotion. The compatibility
matrix stays generic until the manual live-client record is reviewed with its
recorded version/OS scope and the full acceptance criteria above.

## Post-step reassessment — 2026-08-27

The extension-host record now confirms the correct workspace configuration
surface, server discovery, bounded session-tool use, source immutability, and
exact owned removal on the pinned client version. A Copilot attempt to build a
packet was rejected by the strict request schema and fell back to an ordinary
local-file read. This does not change CI-1's scope or authorize a broader
client adapter. It establishes a small, separately scoped CI-2 follow-up:
versioned VS Code-native guidance must make a valid bounded packet request
easier to form and must earn its own live record before any L2 claim.
