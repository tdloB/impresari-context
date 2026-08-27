# CI-1 Codex User-Home Admission — Architecture Requirements and Design

- Status: Approved for implementation
- Date: 2026-08-26
- Governing product record: [CI-1 managed connections PRD](../product/client-integration-l1-managed-connections-prd.md)
- Governing decision: [ADR-0051](../decisions/0051-codex-user-home-managed-connection.md)

## Objective

Prove Codex's L1 managed connection against the configuration surface its App
Server actually loads, without granting the test access to a user's Codex
home or source workspace.

## Design

```text
explicit empty /private/tmp CODEX_HOME
  -> malformed-config rejection
  -> exact kit install and validation
  -> codex mcp get recognition
  -> direct App Server lifecycle + packet comparison
  -> exact kit removal
  -> codex mcp get absence
```

The deterministic raw-MCP control and App Server use the same fixed managed
consumer contract, source workspace, request, event, purpose, and budget.
Each rehearsal receives fresh child caches so fixed request identifiers never
reuse prior audit state.

## Invariants

1. The active user configuration is always caller-named; no default Codex home
   is resolved by Impresari.
2. Only the ownership-marked `impresari-context` entry may be written or
   removed, and only after explicit `--apply` within the disposable rehearsal.
3. Unrelated source files, client trust, login, approvals, global shell state,
   and client-owned temporary-home runtime metadata are outside Impresari's
   write authority.
4. A malformed home configuration fails before valid installation; a removed
   entry is verified absent through the real client.
5. The engine stays client-neutral; the Ruby rehearsal is development-only.

## Verification

- Unit fixtures cover the TOML kit's rendering, malformed input, exact
  ownership, update/removal, and source immutability.
- The host rehearsal records malformed client rejection, user-home
  recognition, deterministic lifecycle, raw-MCP/engine equivalence, packet
  equality, source immutability, and exact removal.
- Hosted CI validates the Rust, contract, policy, security, and script syntax
  gates before public classification changes.
