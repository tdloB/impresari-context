# CI-1 VS Code Copilot extension-host admission verification

- Status: local live-client record captured; hosted CI and public L1 promotion pending
- Candidate scope: VS Code `1.134.0`, macOS arm64
- Governing records: [CI-1 PRD](../product/ci-1-vscode-copilot-admission-prd.md), [CI-1 ARD](../architecture/ci-1-vscode-copilot-admission-ard.md), and [ADR-0057](../decisions/0057-vscode-extension-host-admission.md)

## Local contract evidence

The managed configuration now renders a workspace `.vscode/mcp.json` entry
with `type: "stdio"` for VS Code's visible extension-host MCP UI. Its strict validator rejects a missing or incorrect type,
environment forwarding, remote options, input variables, sandboxing, and any
extra server field. The installed MCP child still receives only fixed workspace,
separate cache, consumer, and role arguments.

The disposable runner never launches VS Code or changes its profile. It asks
the operator to review the named temporary workspace in their signed-in VS Code
client, make the trust/enable decision themselves, observe server discovery and
one visible Impresari tool call, and then provide the exact client version as a
confirmation flag before exact cleanup occurs. A model's choice of that tool is
live-smoke evidence only, never a determinism claim.

## Local live-client record

On 2026-08-27, the signed-in VS Code extension host on macOS arm64, version
`1.134.0`, discovered `impresari-context` from the disposable workspace
`.vscode/mcp.json` entry. The operator explicitly started the server and
allowed `context_session_open` for that chat session. Copilot subsequently
invoked `context_session_close`; both visible calls described bounded
process-local sessions that add no authority.

Copilot also attempted a context-packet request, but the request did not match
the strict `context_build` schema. It therefore retrieved no Impresari packet
and read the local probe file instead. This is recorded as a conversational
request-ergonomics limitation, not packet-equivalence evidence and not a claim
that conversational tool choice is repeatable. It does not invalidate the
limited L1 managed-connection observation, whose required live evidence is
server discovery plus one visible Impresari tool invocation.

The exact cleanup command then validated and removed only the owned
`.vscode/mcp.json` entry. Its source-free result recorded
`confirmed_server_discovery: true`, `confirmed_impresari_tool_invocation: true`,
`source_unchanged: true`, and `owned_configuration_removed: true`.

## Quality-gate evidence

On 2026-08-27, the complete local `./scripts/check.sh` gate passed after the
extension-host correction and local live-client record. This includes formatting, zero-warning Clippy,
unit and integration tests, security-boundary and tracked-source-immutability
checks, schema/fixture contracts, SBOM policy checks, evaluation checks, and
Ruby syntax checks for the rehearsal script. Hosted CI remains a required
promotion condition.

## Operator procedure

```text
ruby scripts/rehearse-vscode-copilot-extension-host.rb \
  --prepare-root /private/tmp/impresari-vscode-extension-host-l1-admission
# inspect the source-free preview, then rerun the same command with --apply
# Open the reported workspace in signed-in VS Code, review trust, use MCP: List
# Servers, and observe one Impresari tool call in Agent Chat.
ruby scripts/rehearse-vscode-copilot-extension-host.rb \
  --temporary-root /private/tmp/impresari-vscode-extension-host-l1-admission \
  --vscode-version 1.134.0 \
  --confirmed-discovery --confirmed-tool-invocation --apply
```

Do not use a real source workspace, VS Code user profile, user configuration,
Agent Host `.mcp.json`, MCP sandbox, automatic-approval setting, or shared
project configuration.

## Promotion condition

This document is not an L1 admission record. The local evidence is now
recorded, but hosted CI must pass on the reviewable change before the public
matrix can change classification. A later VS Code L2 guidance increment must
address the observed `context_build` request ergonomics separately; it cannot
retroactively turn this L1 live smoke into packet-equivalence or deterministic
conversational behavior.
