# CI-4 VS Code Copilot lifecycle-maintenance verification

- Status: passed for the recorded source-free scope
- Verified: 2026-08-29
- Client scope: VS Code `1.134.0`, macOS arm64, native-guidance v3 and
  operator-confirmed Ask-mode L3 delivery
- Governing records: [CI-4 PRD](../product/client-integration-l4-lifecycle-maintenance-prd.md),
  [CI-4 ARD](../architecture/ci-4-client-lifecycle-maintenance-ard.md), and
  [ADR-0043](../decisions/0043-source-free-client-lifecycle-maintenance.md)

## Implemented contract

The VS Code compatibility manifest binds the exact v3 owned instruction path,
ownership marker, artifact SHA-256, client/version/OS/architecture scope,
CI-3f evidence SHA-256, freshness window, lifecycle constraints, and safe next
steps. Its additional delivery contract fixes Ask mode, the admitted
`code_chat_ask_prompt_stdin_context_v1` protocol, required exact operator
confirmation, no source-workspace exposure, no delivery inference from launcher
exit, and zero added authority.

The shared checker requires an explicit manifest, absolute owned target,
caller-supplied client availability, exact client version, OS, architecture,
and assessment date. It does not discover or start VS Code, inspect
authentication or accounts, read a workspace, inherit configuration, spawn a
shell, access a network, mutate an artifact, deliver a packet, or retain a
background process.

## Deterministic evidence

`ruby scripts/check-vscode-client-lifecycle.rb` independently verifies:

1. the released v3 instruction and CI-3f L3 evidence match their manifest
   SHA-256 identities;
2. the closed L3 protocol and authority facts match exactly;
3. compatible, stale-evidence, unsupported-platform, unrecorded-version,
   client-unavailable, missing-target, changed-owned-target, and unowned-target
   states pass or fail as specified;
4. malformed manifests and delivery-contract drift are rejected;
5. every receipt denies source read/write, client mutation, process execution,
   networking, and background monitoring;
6. a separate source fixture remains byte-identical; and
7. exact removal deletes only the owned instruction while preserving an
   unrelated sibling.

The client-specific gate runs from `scripts/check.sh` and the public
compatibility manifest is asserted against the same released manifest.

## Limits and automatic withdrawal

This is not a VS Code process-health probe, account or credential check,
installation discovery mechanism, automatic repair service, or delivery path.
Any unrecorded version or platform is `unknown`/`unsupported`; unavailable,
missing, changed, or unowned state is `degraded`; and dates after 2026-11-27 are
`stale_evidence`. Each state withdraws the recorded L3/L4 compatibility claim
until the exact scope is revalidated. Rehearse again after a VS Code, Copilot
Chat, instruction-format, prompt/stdin, MCP, authentication-boundary, UI, or
platform change.
