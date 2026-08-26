# ADR-0052: Claude Code disposable local-scope admission

- Status: Accepted for implementation
- Date: 2026-08-26
- Scope: Claude Code L1 configuration and admission evidence

## Context

Claude Code CLI `2.1.241` exposes native local-stdio MCP lifecycle commands:
`claude mcp add`, `claude mcp get`, and `claude mcp remove` with a `local`
scope. The existing strict temporary-configuration rehearsal establishes that
an authenticated conversational client can complete the bounded four-tool
Impresari lifecycle and receive a packet identical to direct MCP output. It
cannot, however, make a conversational model choose tools repeatably.

The product needs configuration recognition and exact removal evidence without
reading or changing a person's actual Claude configuration.

## Decision

Claude Code L1 admission uses two complementary, explicit disposable paths:

- a strict temporary MCP configuration for the bounded model-directed packet
  lifecycle; and
- an empty caller-named `/private/tmp` home for native `local` scope
  `add/get/remove` recognition and exact entry removal.

The native rehearsal starts only with an empty disposable home and workspace.
It registers the fixed local-stdio contract, verifies the CLI reports a
connected stdio entry with the expected executable, removes only
`impresari-context`, verifies absence, and proves source bytes remain
unchanged. It never resolves a default real home.

## Constraints

- No automatic real-home discovery/write, project-shared configuration write,
  sign-in, trust grant, approval grant, environment forwarding, remote MCP,
  source mutation, prompt injection, or hidden hook is allowed.
- The model-directed tool sequence is bounded live-client smoke evidence, not
  deterministic conformance. Deterministic L1 gates are the fixed connection
  contract, configuration validation, malformed-input rejection, direct packet
  comparison where observable, source immutability, and exact removal.
- Client-owned metadata in the explicit disposable home is outside Impresari's
  deletion authority. The required cleanup is removal of the exact named MCP
  entry only.
- First-class scope is restricted to the observed Claude Code CLI version and
  macOS architecture until additional evidence is released.

## Consequences

The public Claude Code integration can be classified First-class for its
recorded scope while accurately distinguishing the deterministic configuration
and packet checks from non-deterministic model tool selection. A client update
that changes native configuration behavior requires revalidation and can
demote the classification.

## References

- [CI-1 managed connections PRD](../product/client-integration-l1-managed-connections-prd.md)
- [Claude Code connection-kit record](../verification/phase-1-claude-code-connection-kit.md)
- [ADR-0018: client compatibility contract](0018-first-class-client-integration-and-compatibility-contract.md)
- [ADR-0035: managed connection kits](0035-l1-managed-client-connection-kits.md)
