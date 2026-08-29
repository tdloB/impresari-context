# ADR-0061: Deliver reviewed packets through Claude Code safe-mode print

- Status: Accepted; Claude Code `2.1.241` live admission recorded
- Date: 2026-08-29
- Deciders: Impresari Context maintainers

## Context

Claude Code L1/L2 proves MCP connectivity and native guidance, but neither
guarantees that a reviewed packet enters a specific model turn. Persistent
hooks could inject context at lifecycle points, but they mutate user
configuration, run ambiently, and make per-delivery consent harder to prove.
Claude's programmatic print surface can instead receive one bounded packet in a
fresh process while safe mode disables customizations and project authority.

## Decision

Admit Claude Code `2.1.241` only through an explicit preview/apply adapter using
safe-mode non-interactive print, streaming JSON, an empty tool selection,
disabled slash commands, and no session persistence. Use the existing
authenticated user home in place and, when present, forward the existing
`ANTHROPIC_API_KEY` only to Claude without inspecting or persisting it. Require
the exact echoed input, empty tool/MCP initialization, zero tool-use blocks,
and one successful result. Retain only the bounded receipt.

Do not install a hook or alter Claude configuration for L3. Do not use
`--bare`, because its documented authentication boundary excludes existing
OAuth/keychain authentication and would require a materially different secret
handling design.

## Consequences

- Delivery is explicit, reviewable, version-bound, and independent of model
  tool choice.
- Existing Claude authentication remains under Claude's ownership and is
  neither copied nor read by Impresari.
- Provider network use remains intrinsic and requires explicit authorization
  before live workspace-derived evidence is sent.
- Interactive sessions and future versions remain unadmitted.
- Persistent hook integration may be evaluated later only as a distinct
  configuration and consent boundary.
