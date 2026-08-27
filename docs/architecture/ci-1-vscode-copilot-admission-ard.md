# CI-1 VS Code Copilot admission ARD

- Status: Candidate implementation design
- Date: 2026-08-26
- Related: [CI-1 VS Code Copilot admission PRD](../product/ci-1-vscode-copilot-admission-prd.md), [ADR-0035](../decisions/0035-l1-managed-client-connection-kits.md), and [ADR-0056](../decisions/0056-vscode-portable-agent-host-admission.md)

## Decision

Keep VS Code adaptation inside the existing managed-connection serializer and
validator. The client-specific surface is a workspace-root `.mcp.json` file:
the same source-free preview, validation, explicit apply, inspection, and
owned-entry removal code remains the only writer. No Agent Host protocol
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
render/validate -> explicit install in disposable .mcp.json
                -> operator reviews VS Code trust and discovery
                -> optional bounded tool use is observed
                -> validate source unchanged -> exact owned removal
```

The operator—not Impresari—owns VS Code's trust, enablement, approval, and
sign-in actions. The resulting record is source-free and declares whether the
observation happened; it cannot assert a general model-tool repeatability
guarantee.

## Alternatives rejected

- `.vscode/mcp.json` alone: supported by the extension host but not read
  directly by the portable Agent Host.
- User `~/.copilot/mcp-config.json`: broader persistent scope and outside this
  initial disposable workspace admission.
- VS Code MCP sandboxing: current auto-approval behavior conflicts with the
  no-authority-expansion record.
- An Agent Host protocol adapter or hook: expands scope well beyond an L1
  managed connection.
