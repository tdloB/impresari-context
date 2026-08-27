# ADR-0057: VS Code extension-host admission surface

- Status: Accepted for candidate implementation; L1 admission pending
- Date: 2026-08-27
- Scope: VS Code Copilot extension-host workspace MCP configuration
- Supersedes: ADR-0056 for the extension-host L1 candidate only

## Context

The VS Code MCP documentation separates two workspace configuration surfaces.
The visible VS Code `MCP: List Servers` management UI reads
`.vscode/mcp.json`. Workspace-root `.mcp.json` is read natively by the
separate portable Agent Host. The initial candidate wrote only `.mcp.json`, so
the observed extension-host UI did not discover Impresari Context.

## Decision

The candidate VS Code Copilot L1 path uses workspace
`.vscode/mcp.json` with an exact local `stdio` Impresari Context entry. The
shared kit retains its strict `servers.impresari-context` serializer, fixed
workspace/cache/consumer/role arguments, preview-before-write lifecycle, and
exact owned-entry removal.

The candidate remains pinned to VS Code `1.134.0` on macOS arm64. It rejects
sandbox, environment, input, remote, header, development-watch, and
unrecognized configuration authority. The separate Agent Host `.mcp.json`
surface remains generic and cannot inherit extension-host L1 evidence.

## Consequences

- A disposable extension-host rehearsal can now test the configuration
  surface the signed-in VS Code MCP UI actually loads.
- The public matrix remains generic until the renewed real-client evidence,
  exact owned removal, source-immutability evidence, full local gate, and
  hosted CI all pass.
- No user profile, shared repository, Agent Host `.mcp.json`, sandbox,
  automatic approval, background process, or network service is introduced.

## References

- [VS Code MCP servers](https://code.visualstudio.com/docs/agent-customization/mcp-servers)
- [VS Code MCP configuration reference](https://code.visualstudio.com/docs/agents/reference/mcp-configuration)
- [ADR-0056](0056-vscode-portable-agent-host-admission.md)
- [CI-1 VS Code Copilot admission PRD](../product/ci-1-vscode-copilot-admission-prd.md)
