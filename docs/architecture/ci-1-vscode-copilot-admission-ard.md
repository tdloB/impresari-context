# CI-1 VS Code Copilot admission ARD

- Status: Accepted implementation design; recorded L1 scope
- Date: 2026-08-26
- Related: [CI-1 VS Code Copilot admission PRD](../product/ci-1-vscode-copilot-admission-prd.md), [ADR-0035](../decisions/0035-l1-managed-client-connection-kits.md), and [ADR-0057](../decisions/0057-vscode-extension-host-admission.md)

## Decision

Keep VS Code adaptation inside the existing managed-connection serializer and
validator. The client-specific extension-host surface is a workspace
`.vscode/mcp.json` file: the same source-free preview, validation, explicit
apply, inspection, and owned-entry removal code remains the only writer.
Workspace-root `.mcp.json` is a separate Agent Host compatibility surface and
is not configured or admitted by this increment. No Agent Host protocol
implementation, extension, hook, profile mutator, or background service is
introduced.

## Contract

```json
{
  "servers": {
    "impresari-context": {
      "type": "stdio",
      "command": "/absolute/path/to/impresari-context-mcp",
      "args": ["--workspace", "${workspaceFolder}", "--cache", "/separate/cache", "--consumer-id", "consumer_vscode_managed", "--role", "local_user"]
    }
  }
}
```

The strict key allowlist rejects all omitted or authority-expanding alternatives.
It deliberately does not request VS Code MCP sandboxing because the currently
documented sandbox behavior auto-approves tools.

## Evidence flow

```text
render/validate -> explicit install in disposable .vscode/mcp.json
                -> operator reviews VS Code trust and discovery
                -> optional bounded tool use is observed
                -> validate source unchanged -> exact owned removal
```

The operator—not Impresari—owns VS Code's trust, enablement, approval, and
sign-in actions. The resulting record is source-free and declares whether the
observation happened; it cannot assert a general model-tool repeatability
guarantee.

The 2026-08-27 live observation confirmed discovery and bounded session-tool
use. A conversational `context_build` attempt was rejected by the strict tool
schema and did not produce a packet. This is intentionally outside the narrow
L1 configuration/lifecycle acceptance boundary; it is a separately tracked L2
native-guidance and request-ergonomics concern, not evidence of automatic
fallback, authority expansion, or packet delivery.

## Alternatives rejected

- Workspace-root `.mcp.json`: read directly by the portable Agent Host, but it
  is not the configuration that the observed VS Code MCP management UI loads.
- User `~/.copilot/mcp-config.json`: a broader persistent Agent Host scope and
  outside this initial disposable extension-host admission.
- VS Code MCP sandboxing: current auto-approval behavior conflicts with the
  no-authority-expansion record.
- An Agent Host protocol adapter or hook: expands scope well beyond an L1
  managed connection.
