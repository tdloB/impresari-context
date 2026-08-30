# CI-4 Codex lifecycle-maintenance verification

- Status: passed for the recorded source-free scope
- Verified: 2026-08-30
- Client scope: Codex CLI `0.149.0-alpha.4.1`, macOS aarch64, native-guidance v2
- Governing records: [CI-4 PRD](../product/client-integration-l4-lifecycle-maintenance-prd.md),
  [CI-4 ARD](../architecture/ci-4-client-lifecycle-maintenance-ard.md), and
  [ADR-0043](../decisions/0043-source-free-client-lifecycle-maintenance.md)

## Implemented contract

The Codex compatibility manifest binds the exact v2 owned `AGENTS.md` path,
ownership marker, artifact SHA-256, client/version/OS/architecture scope,
L2 evidence-record SHA-256, freshness window, lifecycle constraints, and safe
next steps. It uses the shared closed receipt and authority denials.

The checker requires an explicit manifest, absolute target, caller-supplied
client availability, exact client version, OS, architecture, and assessment
date. It does not discover or start Codex, inspect a Codex home, authentication,
accounts, or sessions, read repository source, spawn a shell, access a network,
mutate an artifact, or retain a background process.

## Deterministic evidence

`ruby scripts/check-codex-client-lifecycle.rb` independently verifies:

1. the released Codex template and L2 evidence record match their manifest
   SHA-256 identities;
2. compatible, stale-evidence, unsupported-platform, unrecorded-version,
   client-unavailable, missing-target, changed-owned-target, and unowned-target
   results fail or pass as specified;
3. malformed manifests are rejected;
4. every receipt denies source read/write, client mutation, process execution,
   networking, and background monitoring;
5. a separate source fixture remains byte-identical; and
6. exact removal deletes only the owned `AGENTS.md` while preserving an
   unrelated sibling.

The generic manifest and receipt fixtures remain validated by
`scripts/check-contracts.rb`, and the Codex-specific gate runs from
`scripts/check.sh`.

## Limits and reassessment

This is not Codex process health, installation or version discovery, an account
or credential check, automatic repair, background monitoring, proof of repeated
conversational instruction selection, or maintenance of the separately admitted
Codex App Server `0.150.0-alpha.8` L3 delivery path. The L4 claim applies only
to Codex CLI `0.149.0-alpha.4.1` on macOS aarch64 and the exact v2 owned
`AGENTS.md`. Revalidate after an upstream Codex CLI, project-instruction, MCP,
or platform change and before the evidence window expires.
