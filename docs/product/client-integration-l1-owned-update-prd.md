# Impresari Context — CI-1a: Owned Managed-Connection Update PRD

- Status: Approved for implementation
- Date: 2026-08-25
- Authority: Founder-approved client-integration roadmap and autonomous delivery directive
- Governing roadmap: [Client Integration Depth Roadmap](client-integration-roadmap.md)
- Dependency: CI-1 managed connection kit and ADR-0035.
- Architecture requirements: [CI-1a owned-update ARD](../architecture/ci-1-owned-managed-connection-update-ard.md)

## Objective

Add a safe explicit update operation for an existing owned local MCP entry
without converting the L1 installer into a broad configuration editor.

## Problem

Current CI-1 install/validate/remove operations bind ownership to an exact
fixed entry. This intentionally prevents replacing an entry merely because it
has the Impresari name. It also means a binary path, workspace, cache, or
released contract change cannot be updated safely with the current command
shape. A blind overwrite would violate the owned-entry boundary.

## Scope

- A previewable `client kit update` operation accepting an explicit prior
  contract and explicit replacement contract for a caller-named target.
- Validation that the target contains exactly the prior owned entry before any
  replacement is considered.
- Atomic, token-local replacement that preserves unrelated TOML/JSON content,
  rejects symlinks/malformed/duplicate/conflicting states, and writes only with
  `--apply`.
- Stable operation receipt containing old/new redacted contract identities,
  exact planned effect, target, ownership result, and source-free status.
- Positive and negative fixtures for all released client serializers.

## Non-goals

- Default target discovery, broad configuration migration, changing client
  trust/sign-in/approval, an update scheduler, environment forwarding, remote
  transports, provider updates, or update of an unowned/ambiguous entry.

## Acceptance criteria

- Preview changes nothing and shows the exact owned entry to replace and exact
  replacement entry.
- `--apply` succeeds only if the current configuration exactly matches the
  caller-declared prior contract; any drift fails closed with no write.
- Unrelated configuration survives byte-for-byte where possible and
  structurally unchanged otherwise; malformed input, duplicate names,
  ownership-marker ambiguity, symlinks, and stale prior contracts are refused.
- An install → update → validate → remove round trip preserves source bytes and
  removes only the resulting owned entry for Codex TOML and all JSON clients.
- The public matrix remains Generic local MCP until per-client live-client and
  platform evidence meet the independent L1 admission contract.

## Reassessment checkpoint

After implementation, reassess the L1 PRD, ADR-0035, kit records, and client
matrix. Do not expand update into automatic migration or a background service.
