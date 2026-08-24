# Impresari Context — CI-1: First-Class Managed Connections PRD

- Status: Approved for implementation
- Date: 2026-08-24
- Authority: Founder-approved client-integration roadmap and autonomous delivery directive
- Governing roadmap: [Client Integration Depth Roadmap](client-integration-roadmap.md)

## Objective

Promote Codex, Claude Code, Cursor, and GitHub Copilot from generic local MCP
guides to evidence-backed L1 managed connections. The shared capability must
make installation, inspection, validation, update, and owned-entry removal
predictable without silently changing a user’s third-party configuration.

## Scope

- Versioned client/scope manifests for Codex, Claude Code, Cursor, Copilot CLI,
  and VS Code Copilot, each with exact local-stdio command/argument policy.
- Preview/render, validate, explicit install, inspect, and exact owned-entry
  removal operations with stable machine-readable receipts.
- Strict target containment, symlink refusal, bounded configuration size,
  atomic write behavior, unrelated-entry preservation, and ownership markers.
- Per-client malformed configuration, round-trip removal, source immutability,
  platform/version, and disposable real-client lifecycle evidence.
- Public compatibility promotion only after each individual client meets L1.

## Non-goals

- Silent configuration edits, project trust, sign-in, MCP approval, global shell
  changes, remote transport, environment forwarding, provider proxying, model
  routing, persistent memory, source mutation, or background hooks.
- Native instructions/skills/rules (L2), automatic packet delivery (L3), or
  deep lifecycle health integration (L4); they require separate admissions.

## Acceptance criteria

- Every operation previews exact target/owned-entry effects and fails closed on
  ambiguity, unowned state, malformed configuration, unsupported client/scope,
  or unsafe path conditions.
- A successful install/validate/remove round trip changes only the target’s
  owned entry and preserves unrelated configuration and the source workspace.
- Each client record has a version/OS scope, lifecycle smoke evidence, packet
  equivalence where its protocol allows it, malformed-case coverage, and a
  source-free degradation path.
- Client classifications are promoted one at a time only after full local and
  hosted gates plus the relevant live-client record pass.
