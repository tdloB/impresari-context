# CI-1 VS Code Copilot admission verification

- Status: implementation contract pending manual live-client evidence
- Candidate scope: VS Code `1.134.0`, macOS arm64
- Governing records: [CI-1 PRD](../product/ci-1-vscode-copilot-admission-prd.md), [CI-1 ARD](../architecture/ci-1-vscode-copilot-admission-ard.md), and [ADR-0056](../decisions/0056-vscode-portable-agent-host-admission.md)

## Local contract evidence

The managed configuration now renders a portable workspace `.mcp.json` entry
with `type: "stdio"`. Its strict validator rejects a missing or incorrect type,
environment forwarding, remote options, input variables, sandboxing, and any
extra server field. The installed MCP child still receives only fixed workspace,
separate cache, consumer, and role arguments.

The disposable runner never launches VS Code or changes its profile. It asks
the operator to review the named temporary workspace in their signed-in VS Code
client, make the trust/enable decision themselves, observe server discovery and
one visible Impresari tool call, and then provide the exact client version as a
confirmation flag before exact cleanup occurs. A model's choice of that tool is
live-smoke evidence only, never a determinism claim.

## Quality-gate evidence

On 2026-08-26, the complete local `./scripts/check.sh` gate passed after the
candidate implementation. This includes formatting, zero-warning Clippy,
unit and integration tests, security-boundary and tracked-source-immutability
checks, schema/fixture contracts, SBOM policy checks, evaluation checks, and
Ruby syntax checks for the rehearsal script. Hosted CI remains a required
promotion condition.

## Operator procedure

```text
ruby scripts/rehearse-vscode-copilot-portable-agent-host.rb \
  --prepare-root /private/tmp/impresari-vscode-l1-admission
# inspect the source-free preview, then rerun the same command with --apply
# Open the reported workspace in signed-in VS Code, review trust, use MCP: List
# Servers, and observe one Impresari tool call in Agent Chat.
ruby scripts/rehearse-vscode-copilot-portable-agent-host.rb \
  --temporary-root /private/tmp/impresari-vscode-l1-admission \
  --vscode-version 1.134.0 \
  --confirmed-discovery --confirmed-tool-invocation --apply
```

Do not use a real source workspace, VS Code user profile, user configuration,
MCP sandbox, automatic-approval setting, or shared project configuration.

## Promotion condition

This document is not an L1 admission record. After the operator procedure and
the complete local/hosted gates, append the source-free observed outcome,
source-immutability digest, owned-removal result, exact VS Code build/OS, and
client limitation. Only then may the public matrix change classification.
