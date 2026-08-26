# ADR-0053: Cursor guarded project admission

- Status: Accepted for implementation
- Date: 2026-08-26
- Scope: Cursor Agent L1 configuration, approval, and lifecycle evidence

## Context

Cursor Agent CLI `3.17.8` (`2026.08.11-e8db854`) recognizes project-local
`.cursor/mcp.json` entries and provides `mcp enable`, `list-tools`, and
`disable` commands. A preadmission rehearsal already demonstrated fixed
configuration discovery and malformed-input containment, but did not prove
that the agent would execute and preserve a bounded MCP packet lifecycle.

The observed client blocks dynamic MCP calls in Ask mode. Agent mode can
execute them, but ordinarily exposes a broader client tool surface. The product
needs live Agent-mode evidence without accepting shell, source read/write, web,
or unbounded approval authority.

## Decision

Cursor L1 admission uses an explicit empty caller-named project root under
`/private/tmp`. It applies the exact owned project MCP entry, validates it,
checks discovery, enables only `impresari-context`, lists its fixed tool set,
then disables that same identifier and removes only the owned project entry.

For its bounded Agent-mode smoke, the rehearsal creates a temporary project
`.cursor/cli.json` that allows only the four named
`Mcp(impresari-context:tool)` permissions and denies shell, source read/write,
and web calls. The file is content-checked and removed before cleanup. The
agent must call the four session/packet tools in order, and its packet must
equal a direct raw-MCP control packet built from the same fixed inputs.

## Constraints

- No default user/global Cursor configuration, existing source repository,
  sign-in, extension installation, broad approval, remote MCP, environment
  forwarding, shell, source-write, source-read, web, prompt injection, or
  persistent permission policy is targeted or installed by Impresari.
- Cursor's Agent-mode tool choice is bounded live-client smoke evidence, not
  deterministic prompt-repeatability. The deterministic checks are the fixed
  configuration/permission contract, malformed input handling, exact
  enable/disable/removal, source immutability, and direct packet comparison.
- The exact temporary approved-list entry is disabled; the runner does not
  delete or modify unrelated client-owned approval or runtime state.
- The public first-class scope is restricted to the recorded Cursor CLI and
  macOS architecture until additional evidence is released.

## Consequences

Cursor can be classified First-class for the recorded scope while maintaining a
strictly narrower Agent-mode test environment than a normal project. A Cursor
change to configuration precedence, approval semantics, stream result shape,
or execution modes requires revalidation and can demote the classification.

## References

- [CI-1 managed connections PRD](../product/client-integration-l1-managed-connections-prd.md)
- [Cursor connection-kit record](../verification/phase-1-cursor-connection-kit.md)
- [ADR-0018: client compatibility contract](0018-first-class-client-integration-and-compatibility-contract.md)
- [ADR-0035: managed connection kits](0035-l1-managed-client-connection-kits.md)
