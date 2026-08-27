# CI-1 VS Code Copilot admission PRD

- Status: Approved implementation increment; public L1 admission pending
- Date: 2026-08-26
- Roadmap: [Phase 2](phase-2-infrastructure-language-and-agent-expansion-prd.md) and the [Client Integration Depth Roadmap](client-integration-roadmap.md)

## Objective

Replace the VS Code Copilot generic preadmission guide with a candidate
first-class managed-connection path only when a real VS Code surface has
demonstrated its documented portable configuration, explicit trust, server
discovery, bounded tool use, source immutability, and exact removal.

## Scope

- Pin the candidate scope to VS Code `1.134.0` on macOS arm64.
- Render and validate only workspace-root `.mcp.json`, the configuration that
  the VS Code Agent Host reads directly.
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
2. The rendered candidate uses portable workspace `.mcp.json`, not an
   extension-host-only path.
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
