# CI-4 GitHub Copilot CLI lifecycle-maintenance verification

- Status: passed for the recorded source-free scope
- Verified: 2026-08-27
- Client scope: GitHub Copilot CLI `1.0.80`, macOS aarch64, native-guidance v3
- Governing records: [CI-4 PRD](../product/client-integration-l4-lifecycle-maintenance-prd.md),
  [CI-4 ARD](../architecture/ci-4-client-lifecycle-maintenance-ard.md), and
  [ADR-0043](../decisions/0043-source-free-client-lifecycle-maintenance.md)

## Implemented contract

The initial compatibility manifest binds the exact v3 owned guidance path,
ownership marker, artifact SHA-256, client/version/OS/architecture scope,
evidence-record SHA-256, freshness window, lifecycle constraints, and safe
next steps. The receipt schema closes every object and fixes all authority
fields to `denied`.

The checker requires an explicit manifest, absolute target, client-availability
observation, exact client version, OS, architecture, and assessment date. It
does not discover a client, read a workspace or account, inherit configuration,
spawn a process or shell, access a network, mutate an artifact, or retain a
background process.

## Deterministic evidence

`ruby scripts/check-client-lifecycle.rb` verifies:

1. the released template and evidence record match their manifest SHA-256
   identities;
2. compatible, stale-evidence, unsupported-platform, unrecorded-version,
   client-unavailable, missing-target, changed-owned-target, and unowned-target
   results fail or pass as specified;
3. malformed manifests are rejected;
4. every receipt denies source read/write, client mutation, process execution,
   networking, and background monitoring;
5. a separate source fixture remains byte-identical; and
6. exact removal deletes only the matching owned artifact while preserving an
   unrelated sibling.

The contract fixtures are also validated by `scripts/check-contracts.rb`, and
the lifecycle regression check is part of `scripts/check.sh`.

## Limits and reassessment

This is not a Copilot process-health probe, account check, installation
discovery mechanism, repair service, or L3 delivery path. No VS Code Copilot,
Codex, Claude Code, Cursor, Gemini, or other client receives an L4 claim from
this record. Revalidate after an upstream Copilot CLI, instruction, trust, MCP,
or platform change and before the manifest evidence window expires.
