# Phase 1 Codex connection-kit record

- Date: 2026-08-23
- Client surface: Codex CLI `0.149.0-alpha.4.1`
- OS/architecture scope exercised: macOS aarch64
- Classification effect: **Generic local MCP** remains the published claim.
- Scope: project-scoped `.codex/config.toml` template and the read-only
  `doctor codex-config` validator.

## Evidence completed

| Requirement | Result | Evidence |
| --- | --- | --- |
| Current host contract | Passed | Official Codex documentation specifies local stdio MCP, `.codex/config.toml` project scope, trusted-project gating, `codex mcp get`, and a per-server `default_tools_approval_mode`. |
| Template rendering | Passed | The published TOML entry fixes workspace, cache, consumer ID, and role at process launch and uses `prompt` approval mode. |
| Configuration validation | Passed | `doctor codex-config` parses bounded TOML; requires the exact local-stdio entry, an existing absolute binary path, canonical workspace/cache matches, prompt approval, and no environment forwarding or remote fields. |
| Negative cases | Passed | Unit coverage rejects environment forwarding and malformed TOML. |
| Source immutability | Passed | Unit coverage preserves the source-file bytes and emits a source-free doctor report. |
| CLI discovery surface | Passed | The installed client exposes `mcp list`, `mcp get`, `mcp add`, and `mcp remove`; the kit uses only the read-only `mcp get --json` verification command. |

## Deliberate limits

The kit does not write user or project configuration, trust a project, invoke
`codex mcp add`, launch Codex, or change an existing client entry. The core has
no Codex dependency and the kit adds no model, prompt, network, source-write,
execution, memory, or orchestration authority.

The current client CLI did not load a synthetic untrusted project configuration
during read-only discovery. That behavior is expected from the documented trust
gate and confirms that a first-class assertion requires an isolated trusted
clean-install run rather than an automated configuration mutation.

## Admission status

Do not promote Codex to **First-class** yet. Still required:

- clean-install trusted-project lifecycle evidence through an actual Codex
  child-process launch;
- direct-engine versus Codex-delivered packet-corpus equivalence;
- malformed configuration behavior as rendered by Codex itself;
- defined version/OS matrix beyond the exercised macOS aarch64 client; and
- verified entry-specific removal behavior in project and user scopes.

These gaps are the explicit Phase 1 admission work under the
[Phase 1 PRD](../product/phase-1-language-configuration-and-client-admission-prd.md)
and [ADR-0018](../decisions/0018-first-class-client-integration-and-compatibility-contract.md).

## Roadmap checkpoint

After this slice, the Master PRD, Phase 1 PRD, ADR-0018, and ADR-0023 were
reassessed against the completed evidence. No roadmap, authority-boundary, or
admission-criterion change is warranted: the kit remains Generic local MCP,
and the next Phase 1 work remains JSONC, TOML, YAML, and the remaining
client-admission evidence.
