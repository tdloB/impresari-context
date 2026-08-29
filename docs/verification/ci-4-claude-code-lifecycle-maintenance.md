# CI-4 Claude Code lifecycle-maintenance verification

- Status: passed for the recorded source-free scope
- Verified: 2026-08-29
- Client scope: Claude Code CLI `2.1.241`, macOS aarch64, native-guidance v2
- Governing records: [CI-4 PRD](../product/client-integration-l4-lifecycle-maintenance-prd.md),
  [CI-4 ARD](../architecture/ci-4-client-lifecycle-maintenance-ard.md), and
  [ADR-0043](../decisions/0043-source-free-client-lifecycle-maintenance.md)

## Implemented contract

The Claude compatibility manifest binds the exact v2 owned skill path,
ownership marker, artifact SHA-256, client/version/OS/architecture scope,
evidence-record SHA-256, freshness window, lifecycle constraints, and safe
next steps. It uses the same closed receipt contract and authority denials as
the admitted GitHub Copilot CLI scope.

The checker requires an explicit manifest, absolute target, client-availability
observation, exact client version, OS, architecture, and assessment date. It
does not discover or start Claude Code, inspect authentication or accounts,
read a workspace, inherit configuration, spawn a shell, access a network,
mutate an artifact, or retain a background process.

## Deterministic evidence

`ruby scripts/check-claude-client-lifecycle.rb` independently verifies:

1. the released Claude skill and L2 evidence record match their manifest
   SHA-256 identities;
2. compatible, stale-evidence, unsupported-platform, unrecorded-version,
   client-unavailable, missing-target, changed-owned-target, and unowned-target
   results fail or pass as specified;
3. malformed manifests are rejected;
4. every receipt denies source read/write, client mutation, process execution,
   networking, and background monitoring;
5. a separate source fixture remains byte-identical; and
6. exact removal deletes only `.claude/skills/impresari-context/SKILL.md`
   while preserving an unrelated sibling.

The generic manifest and receipt fixtures remain validated by
`scripts/check-contracts.rb`, and both client-specific lifecycle gates run from
`scripts/check.sh`.

## Limits and reassessment

This is not a Claude process-health probe, account or credential check,
installation discovery mechanism, automatic repair service, or L3 delivery
path. The claim applies only to Claude Code CLI `2.1.241` on macOS aarch64 and
the exact v2 owned skill. Revalidate after an upstream Claude Code, skill,
trust, MCP, or platform change and before the evidence window expires.
