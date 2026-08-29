# CI-4 Cursor lifecycle-maintenance verification

- Status: passed for the recorded source-free scope
- Verified: 2026-08-29
- Client scope: Cursor Agent CLI `2026.08.11-e8db854`, macOS aarch64,
  native-guidance v2
- Governing records: [CI-4 PRD](../product/client-integration-l4-lifecycle-maintenance-prd.md),
  [CI-4 ARD](../architecture/ci-4-client-lifecycle-maintenance-ard.md), and
  [ADR-0043](../decisions/0043-source-free-client-lifecycle-maintenance.md)

## Implemented contract

The Cursor compatibility manifest binds the exact v2 owned rule path,
ownership marker, artifact SHA-256, client/version/OS/architecture scope,
evidence-record SHA-256, freshness window, lifecycle constraints, and safe
next steps. It uses the same closed receipt contract and authority denials as
the admitted GitHub Copilot CLI and Claude Code scopes.

The checker requires an explicit manifest, absolute target, caller-supplied
client-availability observation, exact client version, OS, architecture, and
assessment date. It does not discover or start Cursor, inspect authentication
or accounts, read a workspace, inherit configuration, spawn a shell, access a
network, mutate an artifact, or retain a background process.

## Deterministic evidence

`ruby scripts/check-cursor-client-lifecycle.rb` independently verifies:

1. the released Cursor rule and L2 evidence record match their manifest
   SHA-256 identities;
2. compatible, stale-evidence, unsupported-platform, unrecorded-version,
   client-unavailable, missing-target, changed-owned-target, and unowned-target
   results fail or pass as specified;
3. malformed manifests are rejected;
4. every receipt denies source read/write, client mutation, process execution,
   networking, and background monitoring;
5. a separate source fixture remains byte-identical; and
6. exact removal deletes only `.cursor/rules/impresari-context.mdc` while
   preserving an unrelated sibling.

The generic manifest and receipt fixtures remain validated by
`scripts/check-contracts.rb`, and the client-specific gate runs from
`scripts/check.sh`.

## Limits and reassessment

This is not a Cursor process-health probe, account or credential check,
installation discovery mechanism, automatic repair service, or L3 delivery
path. The claim applies only to Cursor Agent `2026.08.11-e8db854` on macOS
aarch64 and the exact v2 owned rule. Revalidate after an upstream Cursor Agent,
rule, trust, MCP, or platform change and before the evidence window expires.
