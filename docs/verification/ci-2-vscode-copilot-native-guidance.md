# CI-2 VS Code Copilot native-guidance verification

- Status: implementation prepared; L2 admission pending a new live-client record
- Candidate scope: VS Code `1.134.0`, macOS arm64
- Governing records: [CI-2 PRD](../product/ci-2-vscode-copilot-native-guidance-prd.md), [CI-2 ARD](../architecture/ci-2-vscode-copilot-native-guidance-ard.md), and [ADR-0058](../decisions/0058-vscode-copilot-native-guidance-and-tool-schema-ergonomics.md)

## Why this is a separate gate

The recorded VS Code L1 extension-host session safely discovered, started, and
used bounded session tools, but its conversational `context_build` request did
not match the strict schema. No Impresari packet was returned; the client then
read the local probe file. That is not packet evidence and is not an L2 pass.

The current owned Copilot v3 instruction now makes the two legal build forms
explicit and directs the client to use the live MCP schema for dynamic values.
The MCP tool schema supplies a canonical direct-evidence example with the
complete hard-budget shape. Neither artifact adds a default budget, automatic
approval, source authority, or delivery behavior.

The first v3 live attempt safely failed before packet construction and did not
read the source fixture. The VS Code Copilot log recorded the exact cause:
`object has unsupported top-level schema keyword 'oneOf'`. A second attempt
using an exact caller-supplied request produced the same definition-level
rejection, then closed its session without source access. The candidate schema
now avoids `oneOf`, `anyOf`, `allOf`, and `not` throughout `context_build`;
regression tests preserve the client-supported subset while runtime tests keep
strict request-form validation authoritative.

## Required manual evidence

The disposable runner prepares only an explicit root below `/private/tmp`. It
installs the exact workspace `.vscode/mcp.json` entry and exact-owned v3
instruction, validates both, and preserves the generated probe source.

```text
ruby scripts/rehearse-vscode-copilot-native-guidance.rb \
  --prepare-root /private/tmp/impresari-vscode-l2-guidance
# inspect the source-free preview, then rerun the same command with --apply
# Open only the reported workspace in VS Code, make the user-owned trust and
# tool-approval decisions, and retain the chat tool-result record.
ruby scripts/rehearse-vscode-copilot-native-guidance.rb \
  --temporary-root /private/tmp/impresari-vscode-l2-guidance \
  --vscode-version 1.134.0 \
  --confirmed-discovery --confirmed-guidance-reference \
  --confirmed-session-lifecycle --confirmed-packet-build \
  --confirmed-packet-resolve --apply
```

The operator must confirm all of the following from the live VS Code client:

1. The workspace `.vscode/mcp.json` entry is visible and explicitly started.
2. The owned v3 instruction is shown in chat references or diagnostics for the
   probe request.
3. `context_session_open`, a successful `context_build`, a successful
   same-session `context_packet_resolve`, and `context_session_close` are
   visible. A direct-file fallback is a failed smoke, not a substitute.
4. The cleanup receipt shows that only the owned configuration and guidance
   files were removed and that the source fixture was unchanged.

## Promotion condition

L2 is not admitted until the GitHub Copilot CLI v3 native-guidance smoke, the
manual VS Code packet build/resolve record, the complete local gate, and hosted
CI all pass. A conversational result remains live smoke evidence only; it does
not claim that repeating a prompt repeats the same tool calls.
