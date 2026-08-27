# ADR-0056: VS Code portable Agent Host admission

- Status: Superseded by ADR-0057 for the VS Code extension-host L1 candidate; retained for the separate Agent Host surface
- Date: 2026-08-26
- Scope: VS Code Copilot workspace MCP configuration

## Context

VS Code's current MCP reference distinguishes extension-host workspace
`.vscode/mcp.json` from the portable Agent Host configuration it reads natively:
workspace-root `.mcp.json`. The initial VS Code preadmission guide described
only the former and allowed a type-less server entry. Its current documented
stdio contract requires `type: "stdio"`.

## Decision

The candidate L1 path is workspace-root `.mcp.json` with an exact direct local
stdio Impresari Context server entry. It is recorded only for VS Code `1.134.0`
on macOS arm64. The existing shared kit may write this named temporary file
only after explicit preview and `--apply`; all other profile and client state
remains user-owned.

The validator permits only `type`, `command`, and `args`, and requires the
fixed Impresari launch authority. Sandbox, environment, input variables,
remote URLs, headers, development watching, and client auto-approval are
rejected. VS Code sandboxing is deliberately excluded because its documented
server-tool auto-approval would conflict with this L1 boundary.

## Consequences

- A local strict configuration correction is available immediately without
  overstating client admission.
- A signed-in operator must explicitly perform and evidence VS Code trust,
  enablement, discovery, and one tool interaction in a disposable workspace.
- The public matrix cannot promote VS Code Copilot until that client evidence,
  exact removal, source immutability, full quality gate, and hosted CI exist.
- VS Code extension-host and Agent Host modes remain distinct evidence scopes.

## References

- [VS Code MCP configuration reference](https://code.visualstudio.com/docs/agents/reference/mcp-configuration)
- [VS Code Agent Host architecture](https://code.visualstudio.com/docs/agents/concepts/agent-host)
- [CI-1 VS Code Copilot admission PRD](../product/ci-1-vscode-copilot-admission-prd.md)
- [ADR-0035](0035-l1-managed-client-connection-kits.md)

## Supersession

The initial candidate did not appear in the observed `MCP: List Servers` UI:
that UI loads workspace `.vscode/mcp.json`, while this ADR's workspace-root
`.mcp.json` is for the separate Agent Host surface. ADR-0057 records the
extension-host correction. This ADR remains historical context and does not
admit the Agent Host surface.
