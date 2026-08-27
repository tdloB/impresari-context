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

## Implemented command boundary

The first implementation exposes the same versioned L1 contract for `codex`,
`claude`, `cursor`, `copilot`, and `vscode`:

```text
client kit render   <client> <binary> <workspace> <cache>
client kit inspect  <client> <binary> <workspace> <cache> <config-file>
client kit validate <client> <binary> <workspace> <cache> <config-file>
client kit install  <client> <binary> <workspace> <cache> <config-file> [--apply]
client kit update   <client> <old-binary> <old-workspace> <old-cache> <binary> <workspace> <cache> <config-file> [--apply]
client kit remove   <client> <binary> <workspace> <cache> <config-file> [--apply]
```

`install`, `update`, and `remove` are previews unless the caller passes
`--apply`; no
command discovers or targets a default client configuration. The target parent
must already exist and may not itself be a symlink. The target must be absent
or a bounded, UTF-8, regular non-symlink file. JSON modifications are
token-local edits that retain unrelated values; TOML additions and removals
append/remove only the exact ownership-marked Impresari table. Any malformed,
duplicate, conflicting, or unowned Impresari entry fails closed.

`update` requires both the exact prior contract and the desired replacement
contract. It proves that the named target contains the former before it renders
or atomically replaces it with the latter. It never performs a name-based or
automatic overwrite.

## Codex scope correction

Codex CLI/App Server `0.149.0-alpha.4.1` on macOS aarch64 loads the active
user-level Codex-home configuration rather than a repository
`.codex/config.toml` MCP entry. Therefore the Codex L1 target scope is `user`:
the caller must name the target `CODEX_HOME/config.toml`, and the kit must not
discover or automatically write a real user home. The isolated admission
rehearsal may use an explicit empty `/private/tmp` `CODEX_HOME`, where it
proves malformed-configuration rejection, exact install/recognition/removal,
deterministic lifecycle, packet equivalence, and source immutability.

## Claude Code scope admission

Claude Code CLI `2.1.241` on macOS aarch64 exposes a native `local` MCP scope
through `claude mcp add/get/remove --scope local`. The Claude L1 record uses an
explicit empty disposable `HOME` under `/private/tmp`, rather than a default
user home, to prove the fixed entry is accepted, connected, and exactly
removed. Its independent strict temporary-configuration rehearsal records the
model-directed session/packet lifecycle and direct-packet equivalence.

The model-directed step is live smoke evidence only; native add/get/remove,
fixed configuration validation, malformed-input rejection, source
immutability, and entry-specific removal remain deterministic admission gates.

## Cursor scope admission

Cursor Agent CLI `3.17.8` (`2026.08.11-e8db854`) on macOS aarch64 uses an
explicit project `.cursor/mcp.json` entry and a local approved-list transition
through `cursor agent mcp enable/list-tools/disable`. The Cursor L1 record uses
an empty caller-named project root under `/private/tmp`; it applies and removes
only the owned project entry and enables/disables only `impresari-context`.

Cursor Ask mode blocks dynamic MCP calls, including this fixed read-only
server. Its bounded Agent-mode smoke creates a test-only project `cli.json`
that allows only the four named Impresari MCP tools and denies shell, source
read/write, and web actions. The file is verified and removed before the
rehearsal ends. This is an admission guardrail, not a product installer or a
recommended persistent user permission policy.
