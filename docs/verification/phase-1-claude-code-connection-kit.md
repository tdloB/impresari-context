# Phase 1 Claude Code connection-kit record

- Date: 2026-08-26
- Client surface: Claude Code CLI `2.1.241`
- OS/architecture scope exercised: macOS aarch64
- Classification effect: **First-class** for the recorded client/version/OS
  scope, subject to the repository's hosted release gate.
- Scope: local-stdio guide, read-only configuration validator, disposable
  native local-scope lifecycle, and temporary real-client MCP rehearsal.

## Evidence completed

| Requirement | Result | Evidence |
| --- | --- | --- |
| Client availability and authenticated host path | Passed | Claude Code CLI `2.1.241` on macOS ARM64 completed the bounded host-side model request used by the disposable rehearsal. The sandbox cannot access the client credential store; the host result is the admission evidence. |
| Fixed local-stdio configuration | Passed | The versioned managed kit renders and validates the fixed executable local-stdio command, workspace, separate cache, consumer ID, and role without environment forwarding. |
| Native local-scope registration and removal | Passed | `scripts/rehearse-claude-native-local-scope.rb` starts with an explicit empty Claude home under `/private/tmp`, registers the named local entry with `claude mcp add --scope local`, verifies Claude Code reports it as a connected stdio server with the fixed executable contract, removes only that entry, and confirms `claude mcp get` no longer recognizes it. |
| Real temporary-config lifecycle | Passed | `scripts/rehearse-claude-code.rb` explicitly uses managed install, validate, and exact removal against a disposable `--mcp-config`, starts Claude with `--strict-mcp-config`, makes built-in tools unavailable, and permits only the minimal four-tool session/packet lifecycle. |
| Complete operation sequence and packet equivalence | Passed | Claude called `context_session_open`, `context_build`, `context_packet_resolve`, and `context_session_close` in order. The rehearsal requires each non-error tool result, requires Claude's resolved packet to equal its delivered packet, and proves that packet exactly equals an independent direct MCP packet from the same fixture. |
| Source and configuration containment | Passed | The fixture source digest remained unchanged; the temporary managed configuration target was removed after the run; and `claude mcp get impresari-context` confirmed no persistent registration was created. |
| Client-rendered malformed configuration | Passed | `scripts/rehearse-claude-code.rb --malformed-config-only` gives Claude Code a malformed disposable MCP configuration under `--strict-mcp-config`. The client rejects it before any model call or MCP startup, and the fixture source digest remains unchanged. |

## Deliberate limits

Claude Code's exposed client path is model-directed. Restricting built-in tools
and preapproving the exact MCP tools makes a bounded real-client lifecycle
rehearsal possible, but does not make model tool selection deterministic. This
evidence is therefore not a substitute for a direct client RPC conformance
surface like Codex App Server.

The model-directed rehearsal requires an authenticated user's ordinary Claude
Code account and makes a bounded model request only after strict temporary
configuration is accepted. It is an opt-in local maintenance check, not a
hosted CI test and not a persistent configuration installer. Client credential
access is not available inside the restricted test sandbox, so the host
execution is the authoritative live-client record.

The native-registration rehearsal is preview-first: `--prepare-root` reports
the exact disposable root, home, workspace, and cache; `--apply` creates only
those empty directories under `/private/tmp`; and `--temporary-root` is then
allowed to register and remove only `impresari-context` in that named
disposable home. It never discovers, reads, or modifies the user's real Claude
home. Claude may keep its own runtime metadata in the disposable home; the
rehearsal deliberately removes only Impresari's exact named entry.

## Admission status

The local L1 evidence is complete for Claude Code CLI `2.1.241` on macOS
aarch64: versioned kit, fixed configuration validation, malformed strict
client configuration rejection, native isolated add/get/remove, bounded
real-client lifecycle with packet equivalence, and source immutability. The
published claim remains restricted to this exact version/OS scope and requires
revalidation after an upstream client/configuration change.

Claude Code's model-directed tool selection is recorded as live-client smoke
evidence, not as a repeatable client-conformance requirement. The
deterministic gates are the managed connection contract, fixed authority,
validation, malformed-input handling, packet evidence where observable, and
exact removal.

These requirements remain governed by the [Phase 1 PRD](../product/phase-1-language-configuration-and-client-admission-prd.md),
[ADR-0018](../decisions/0018-first-class-client-integration-and-compatibility-contract.md),
and [ADR-0052](../decisions/0052-claude-disposable-local-scope-admission.md).

## Roadmap checkpoint

The Master PRD, Phase 1 PRD, ADR-0018, ADR-0035, and the client-integration
roadmap were reassessed. The native disposable local-scope record closes the
remaining Claude Code L1 evidence gap without changing the language roadmap or
granting Impresari write access to an actual user configuration. Cursor's
user-owned IDE enablement remains the next distinct Phase 1 admission boundary.
