# Phase 1 Cursor connection-kit record

- Status: Generic local MCP; temporary configuration discovery recorded
- Date: 2026-08-23
- Client: Cursor Agent CLI `3.17.8` (`2026.08.11-e8db854`), macOS aarch64

## Evidence recorded

The user authenticated Cursor Agent CLI through its normal interactive flow.
`scripts/rehearse-cursor-preadmission.rb` then created an isolated temporary
workspace, separate cache, and project `.cursor/mcp.json` containing only the
fixed Impresari Context local-stdio command and arguments. `cursor agent mcp
list` discovered `impresari_context_conformance`; the fixture workspace digest
was unchanged after the configuration was created and listed.

The rehearsal did not invoke `cursor agent mcp enable`, `disable`, or an AI
model. No MCP approval was granted and no user or project configuration was
persisted. The connection shape remains bounded: an absolute MCP binary,
fixed workspace, separate cache, consumer identity, and local-user role, with
environment forwarding rejected by the Impresari Context validator.

## Classification and remaining gaps

Cursor remains **Generic local MCP**. Configuration discovery does not prove
that Cursor will launch the server, expose the intended tools, or preserve the
evidence packet during a model-directed session.

First-class admission still requires all of the following:

- intentional user approval of an exact, isolated MCP entry and a real-client
  tool lifecycle;
- packet identity/equivalence and source-immutability evidence;
- malformed-configuration behavior and safe removal of only this entry; and
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
