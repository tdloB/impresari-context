# ADR-0062: Deliver reviewed packets through Cursor Agent ask-mode print

- Status: Accepted; recorded-scope L3 admitted
- Date: 2026-08-29
- Deciders: Impresari Context maintainers

## Context

Cursor L1/L2 proves managed local MCP configuration and native project rules,
but neither guarantees delivery into one model turn. Cursor documents print
mode for automation and stream-JSON evidence, while also warning that
non-interactive mode has write and shell tools.

## Decision

Admit only Cursor Agent `2026.08.25-3e8eec8` through an explicit preview/apply
adapter using ask mode, sandboxing, stdin prompt delivery, and one empty
disposable workspace. Pass `--trust` only for that newly created empty runtime;
never trust or expose the source workspace. Verify Cursor authentication with a silent `status`
preflight whose output is discarded and whose boolean result is carried in the
receipt. Reject every observed tool call and require exact prompt, cwd, and
terminal evidence.

Do not install hooks, use a cloud worker, trust a source workspace, expose source,
approve MCP, enable force or auto-review, retain sessions, or infer safety from
model instructions alone.

## Consequences

- Delivery remains explicit, bounded, version-specific, and independent of
  model tool selection.
- Cursor retains intrinsic provider networking and existing authentication;
  Impresari handles neither credential nor response content.
- The integration is narrower than Cursor's general automation surface by
  design. Any tool use or upstream contract drift withdraws the L3 claim.
