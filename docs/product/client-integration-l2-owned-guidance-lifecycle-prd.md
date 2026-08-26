# Impresari Context — CI-2a: Owned Native-Guidance Lifecycle PRD

- Status: Approved for implementation
- Date: 2026-08-25
- Authority: Founder-approved client-integration roadmap and autonomous delivery directive
- Governing roadmap: [Client Integration Depth Roadmap](client-integration-roadmap.md)
- Dependency: [CI-2 Native Agent Guidance PRD](client-integration-l2-native-guidance-prd.md)
- Architecture requirements: [CI-2a owned guidance lifecycle ARD](../architecture/ci-2a-owned-guidance-lifecycle-ard.md)

## Objective

Turn CI-2's static, reviewed guidance templates into a narrow local lifecycle:
render, inspect, validate, preview install, explicit install, and exact owned
removal. The lifecycle must install only one released artifact at one fixed
project-relative path per client and must never merge, append to, or overwrite
an existing instruction file.

## Scope

- `client guidance` operations for Codex, Claude Code, Cursor, and GitHub
  Copilot project artifacts.
- Deterministic render output with artifact version, exact relative path,
  content digest, ownership marker, and the fixed template bytes.
- A caller-named existing project root; the target path is derived, not chosen
  by repository content or client discovery.
- Read-only inspect and validate, no-write install preview, explicit `--apply`
  install, and exact-owned `--apply` removal.
- Bounded, regular, UTF-8 target inspection and atomic write/removal.

## Non-goals

- Editing an existing `AGENTS.md`, rule, skill, or Copilot instruction;
  creating project directories; global/user-scope installation; client trust,
  approval, sign-in, enablement, MCP installation, packet delivery, hooks,
  networking, repository execution, or source changes.

## Acceptance criteria

- A valid install target is absent under a caller-provided non-symlink project
  root. Existing, symlinked, malformed, oversized, duplicate, or unowned
  targets are rejected without a write.
- Preview executes all validations but creates no directory or file. Only
  `--apply` can create a missing artifact and only when its parent directories
  already exist.
- Validation accepts only the exact released bytes at the exact path. Removal
  deletes only that exact owned artifact, leaving unrelated files intact; a
  missing artifact is reported rather than treated as removed.
- Tests cover all client paths, render determinism, install/inspect/validate/
  remove round trips, preview/rejection no-write behavior, source-workspace
  immutability, and symlink/conflict/oversize defenses.
- The lifecycle remains a local deterministic capability. An individual client
  reaches L2 only after client/version/OS and opt-in native-surface smoke
  evidence; no lifecycle command changes L1 classification.

## Reassessment checkpoint

After this lifecycle is release-gated, reassess CI-2's artifact paths against
upstream client documentation and reassess the master PRD, compatibility
matrix, and client-integration roadmap. A client whose native surface cannot
preserve exact owned-file semantics remains at its prior level.
