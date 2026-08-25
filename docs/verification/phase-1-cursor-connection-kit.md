# Phase 1 Cursor connection-kit record

- Status: Generic local MCP; temporary configuration discovery recorded
- Date: 2026-08-25
- Client: Cursor Agent CLI `3.17.8` (`2026.08.11-e8db854`), macOS aarch64

## Evidence recorded

The user authenticated Cursor Agent CLI through its normal interactive flow.
`scripts/rehearse-cursor-preadmission.rb` then created an isolated temporary
workspace and separate cache. It invokes the versioned managed-kit CLI to
explicitly install the exact owned `.cursor/mcp.json` entry, validates that
entry, asks `cursor agent mcp list` to discover it, then explicitly removes it.
The target file is absent after the removal and the fixture workspace digest is
unchanged. Cursor Agent discovered `impresari-context` on CLI `3.17.8`
(`2026.08.11-e8db854`), macOS aarch64.

The rehearsal did not invoke `cursor agent mcp enable`, `disable`, or an AI
model. No MCP approval was granted and no user or project configuration was
persisted. The connection shape remains bounded: an executable absolute MCP
binary, fixed workspace, separate cache, consumer identity, and local-user
role, with environment forwarding rejected by the Impresari Context validator.

## Classification and remaining gaps

Cursor remains **Generic local MCP**. Configuration discovery does not prove
that Cursor will launch the server, expose the intended tools, or preserve the
evidence packet during a model-directed session.

First-class admission still requires all of the following:

- intentional user approval of an exact, isolated MCP entry and a real-client
  tool lifecycle;
- packet identity/equivalence and source-immutability evidence; and
- maintained version and operating-system coverage.

This evidence is deliberately not substituted with `--approve-mcps` or an
automatic approval flag, because those change Cursor's local approval state.
The [local MCP connection guide](../reference/local-mcp-connection-guides.md#cursor)
contains the user-reviewed setup shape.

## Roadmap checkpoint

No product-roadmap or trust-boundary change is warranted. The addition
confirms that the existing generic-MCP configuration contract is compatible
with the authenticated Cursor CLI, but it does not satisfy a first-class
client conformance criterion.
