# Phase 1 Codex connection-kit record

- Date: 2026-08-23
- Client surface: Codex CLI `0.149.0-alpha.4.1`
- OS/architecture scope exercised: macOS aarch64
- Classification effect: **Generic local MCP** remains the published claim.
- Scope: project-scoped `.codex/config.toml` template, the read-only
  `doctor codex-config` validator, and deterministic App Server transport
  conformance.

## Evidence completed

| Requirement | Result | Evidence |
| --- | --- | --- |
| Current host contract | Passed | Official Codex documentation specifies local stdio MCP, shared host configuration, `.codex/config.toml` trusted-project scope, `codex mcp get`, and a per-server `default_tools_approval_mode`. |
| Template rendering | Passed | The published TOML entry fixes workspace, cache, consumer ID, and role at process launch and uses `prompt` approval mode. |
| Configuration validation | Passed | `doctor codex-config` parses bounded TOML; requires the exact local-stdio entry, an existing absolute binary path, canonical workspace/cache matches, prompt approval, and no environment forwarding or remote fields. |
| Negative cases | Passed | Unit coverage rejects environment forwarding and malformed TOML. |
| Source immutability | Passed | Unit coverage preserves the source-file bytes and emits a source-free doctor report. |
| CLI discovery surface | Passed | The installed client exposes `mcp list`, `mcp get`, `mcp add`, and `mcp remove`; the kit uses only the read-only `mcp get --json` verification command. |
| Live local lifecycle | Passed, limited scope | Codex CLI `0.149.0-alpha.4.1` on macOS aarch64 launched the existing fixed-stdio entry and completed `context_session_open` and `context_build` against an isolated empty workspace. The delivered packet ID was `sha256:919b7bc7cec5466a48e3a6fbd701573f2783768dc6fec03171b472b7cfd77818`; workspace bytes remained unchanged. |
| Read-only execution behavior | Passed, negative case | `codex exec --sandbox read-only` discovers the MCP server but blocks its tool call because that execution mode sets MCP approval to `never`. This is expected client policy behavior, not a server failure. The successful lifecycle check used Codex's explicit automatic approval mode while the MCP server itself retained the fixed read-only authority contract. |
| Deterministic App Server tool lifecycle | Passed | `scripts/rehearse-codex-app-server.rb` starts a local Codex App Server with a one-use `-c` MCP definition, creates an ephemeral read-only thread, lists the dedicated server, and directly invokes `context_session_open`, `context_build`, `context_packet_resolve`, and `context_session_close`. It never asks a model to select a tool or changes a Codex configuration file. |
| Packet equivalence through Codex | Passed | The rehearsal creates a temporary TypeScript fixture and fixed startup/request time, proves direct-engine/in-process-MCP equivalence with `doctor mcp`, then requires byte-for-byte equality between a raw MCP child-process packet and the packet delivered through Codex App Server. The resolved session packet must equal the delivered packet; the fixture digest is unchanged. Packet identity intentionally differs between runs because the temporary fixture has a unique workspace identity. |

## Deliberate limits

The kit does not write user or project configuration, trust a project, invoke
`codex mcp add`, launch Codex, or change an existing client entry. The core has
no Codex dependency and the kit adds no model, prompt, network, source-write,
execution, memory, or orchestration authority.

The current client CLI did not load a synthetic untrusted project configuration
during read-only discovery. That behavior is expected from the documented trust
gate and confirms that a first-class assertion requires an isolated trusted
clean-install run rather than an automated configuration mutation.

A conversational `codex exec` session remains useful usability evidence, but
is not the release gate: its model may select a different available MCP
operation despite the same prompt. The App Server's explicit
`mcpServer/tool/call` RPC is the deterministic host-transport conformance
surface. The rehearsal uses that surface without supplying a prompt or making
a model request.

## Admission status

Do not promote Codex to **First-class** yet. Still required:

- malformed configuration behavior as rendered by Codex itself;
- defined version/OS matrix beyond the exercised macOS aarch64 client; and
- verified entry-specific removal behavior in project and user scopes; and
- an isolated trusted-project clean-install record for the published
  `.codex/config.toml` template.

These gaps are the explicit Phase 1 admission work under the
[Phase 1 PRD](../product/phase-1-language-configuration-and-client-admission-prd.md)
and [ADR-0018](../decisions/0018-first-class-client-integration-and-compatibility-contract.md).
The deterministic transport decision is recorded separately in
[ADR-0028](../decisions/0028-codex-deterministic-mcp-tool-conformance.md).

## Roadmap checkpoint

After this slice, the Master PRD, Phase 1 PRD, ADR-0018, and ADR-0023 were
reassessed against the completed evidence. The roadmap is unchanged. Codex now
has deterministic host-transport and packet evidence, but remains Generic
local MCP until its remaining configuration-scope, removal, and platform
admission criteria are met. Claude Code and Cursor still require a user-owned
authentication step before their real-client lifecycle admissions can begin.
