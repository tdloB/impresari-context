# Impresari Context — CI-2: Native Agent Guidance PRD

- Status: Approved for implementation
- Date: 2026-08-24
- Authority: Founder-approved client-integration roadmap and autonomous delivery directive
- Governing roadmap: [Client Integration Depth Roadmap](client-integration-roadmap.md)
- Architecture requirements: [CI-2 native guidance ARD](../architecture/ci-2-native-guidance-artifacts-ard.md)
- Dependency: CI-1 managed connection kits remain the first-class (L1) promotion gate.

## Objective

Provide small, client-native, opt-in guidance artifacts that help Codex, Claude
Code, Cursor, and GitHub Copilot users request bounded, evidence-grade context.
The artifacts must improve discoverability without gaining authority over client
configuration, tool approval, source code, network access, or automatic packet
delivery.

## Scope

- One versioned, original guidance artifact per officially supported client
  surface: Codex project instructions, Claude Code skill/instruction surface,
  Cursor project rule, Copilot CLI/VS Code instruction or agent surface.
- A common content contract: when to request a packet, supported task profiles,
  how to cite packet identity and reason codes, and how to handle an omission
  or unavailable MCP server.
- Preview, validate, explicit install, inspect, and exact owned-artifact
  removal, preserving unrelated instruction content.
- Fixed artifact version, target scope, platform/client-version record, source
  immutability proof, malformed-artifact behavior, and opt-in round-trip test.

## Non-goals

- Any automatic context injection, lifecycle hook, background process,
  prompt-rewriting proxy, model routing, memory, remote service, shell/profile
  mutation, project trust/approval change, or general third-party client setup.
- A claim that conversational tool choice is deterministic.
- L3 guided context delivery and L4 lifecycle maintenance; those retain their
  own roadmap gates.

## Content contract

Every artifact must say that Impresari Context produces source-grounded,
snapshot-bound evidence—not repository execution, hidden semantics, or policy
authority. It must request an explicit profile and bounded budget, preserve the
user's task wording, make packet identity and omissions visible, and instruct
the user to continue without packet delivery when the server is unavailable.

Artifacts may reference the exact released MCP tools only. They may not embed
secrets, auto-approve tools, ask a client to trust a workspace, alter unrelated
instructions, or direct a client to write source files.

## Acceptance criteria

- Rendering is deterministic for a client/scope/version and contains an
  ownership marker.
- Installation and removal occur only after an explicit apply action against a
  caller-named target; diagnostics and previews are read-only.
- The installer rejects symlinks, ambiguous or unowned artifacts, duplicate
  ownership markers, malformed content, unsupported scope, and oversized
  targets; it preserves unrelated content byte-for-byte where the host format
  permits exact preservation.
- The live MCP tool schema exposes the fixed resource-policy fingerprint and
  request/event identifier grammar required for a valid packet request; an
  artifact must not copy mutable protocol values that would compromise exact
  owned-artifact removal on correction.
- A round trip proves install, inspect, validate, and owned-only removal,
  source-workspace immutability, and no authority expansion.
- Each client is promoted to L2 only after its own supported client/version/OS
  evidence and one opt-in native-surface smoke record. L2 never promotes a
  client to L1 by itself.

## Rollout and degradation

CI-2 begins only for clients whose official surface can carry a versioned,
owned project artifact without requiring global settings. If a client surface
cannot preserve exact ownership/removal, it remains at L1 or L0. A missing,
invalid, or rejected artifact degrades to ordinary manual MCP use and reports
no misleading health claim.

## Reassessment checkpoint

After each client artifact and admission record, reassess the master PRD,
client-integration roadmap, compatibility matrix, and this PRD. Minor
clarifications may proceed autonomously; new authority or automated delivery
requires a separate decision.
