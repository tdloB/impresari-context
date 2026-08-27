# Phase 1 Codex connection-kit record

- Date: 2026-08-26
- Client surface: Codex CLI `0.149.0-alpha.4.1`
- OS/architecture scope exercised: macOS aarch64
- Classification effect: **First-class** for the recorded client/version/OS
  scope, subject to the repository's hosted release gate.
- Scope: explicit user-level `$CODEX_HOME/config.toml` target, the read-only
  `doctor codex-config` validator, and deterministic App Server transport
  conformance.

## Evidence completed

| Requirement | Result | Evidence |
| --- | --- | --- |
| Current host contract | Passed | The installed CLI exposes `mcp add`, `mcp get`, and `mcp remove`. Its App Server loads the active Codex home configuration; an observed trusted `.codex/config.toml` file was not a runtime MCP configuration source on this client version. |
| Template rendering | Passed | The managed TOML entry fixes workspace, cache, consumer ID, and role at process launch; requires the server; enables it; and uses `prompt` approval mode. |
| Configuration validation | Passed | `doctor codex-config` parses bounded TOML; requires the exact local-stdio entry, an existing executable absolute binary path, canonical workspace/cache matches, prompt approval, enabled/required values when present, and no environment forwarding or remote fields. |
| Negative cases | Passed | Unit coverage rejects environment forwarding and malformed TOML. |
| Source immutability | Passed | Unit coverage preserves the source-file bytes and emits a source-free doctor report. |
| CLI discovery surface | Passed | An isolated active `CODEX_HOME` accepted the exact rendered entry (`codex mcp get impresari-context`) and, after kit removal, did not retain that entry. |
| Live managed lifecycle | Passed | Codex CLI `0.149.0-alpha.4.1` on macOS aarch64 loaded the exact kit entry from an empty disposable `CODEX_HOME`, completed the full four-tool lifecycle against an isolated empty workspace, and removed only the named entry. The delivered packet ID was `sha256:9df0338a3689afdd90f362b11d028f1b928deea32bf04a2341c6fd084ffe1fde`; workspace bytes remained unchanged. |
| Read-only execution behavior | Passed, negative case | `codex exec --sandbox read-only` discovers the MCP server but blocks its tool call because that execution mode sets MCP approval to `never`. This is expected client policy behavior, not a server failure. The successful lifecycle check used Codex's explicit automatic approval mode while the MCP server itself retained the fixed read-only authority contract. |
| Deterministic App Server tool lifecycle | Passed | `scripts/rehearse-codex-app-server.rb` creates an ephemeral read-only thread, lists the dedicated server, and directly invokes `context_session_open`, `context_build`, `context_packet_resolve`, and `context_session_close`. The baseline uses a one-use `-c` definition; managed admission uses a disposable `CODEX_HOME`. Neither path asks a model to select a tool. |
| Packet equivalence through Codex | Passed | The rehearsal creates a temporary TypeScript fixture and fixed startup/request time, proves direct-engine/in-process-MCP equivalence with `doctor mcp`, then requires byte-for-byte equality between a raw MCP child-process packet and the packet delivered through Codex App Server. The resolved session packet must equal the delivered packet; the fixture digest is unchanged. Packet identity intentionally differs between runs because the temporary fixture has a unique workspace identity. |
| Malformed client configuration | Passed | An intentionally malformed `config.toml` in an otherwise empty disposable `CODEX_HOME` was rejected by `codex mcp list` before the valid managed entry was installed. |
| Exact owned-entry removal | Passed | The rehearsal applies the versioned kit, validates it, runs the live lifecycle, removes the exact ownership-marked table, and confirms `codex mcp get impresari-context` no longer succeeds in that isolated home. |

## Deliberate limits

The kit writes a named target only after an explicit `--apply` install or
remove. It does not discover a default target, trust a project, change an
unowned entry, or write the user's Codex home during the rehearsal. The core
has no Codex dependency and the kit adds no model, prompt, network, memory, or
orchestration authority.

The tested temporary home may receive Codex's own local runtime metadata while
the App Server runs. That home is an explicit disposable `/private/tmp` target;
the admission evidence requires removal of Impresari's exact configuration
entry, not deletion of client-owned runtime state. No user home is read or
changed by the rehearsal.

A conversational `codex exec` session remains useful usability evidence, but
is not the release gate: its model may select a different available MCP
operation despite the same prompt. The App Server's explicit
`mcpServer/tool/call` RPC is the deterministic host-transport conformance
surface. The rehearsal uses that surface without supplying a prompt or making
a model request.

## Admission status

The local L1 evidence is complete for Codex CLI `0.149.0-alpha.4.1` on macOS
aarch64: versioned kit, user-level configuration validation, malformed-client
failure, isolated install/client recognition/removal, deterministic tool
lifecycle, packet equivalence, and source immutability. Public promotion is
released only with the hosted gate for this change; the claimed scope remains
that exact client/version/OS combination.

## Roadmap checkpoint

After this slice, the Master PRD, Phase 1 PRD, ADR-0018, ADR-0028, and CI-1
roadmap were reassessed. Codex's managed configuration scope is corrected from
an unsupported project file to the observed user-level home. The next CI-1
admission target is Claude Code; no language roadmap change is needed.
