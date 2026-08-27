# Phase 1 Cursor connection-kit record

- Status: First-class for the recorded client/version/OS scope, subject to the
  repository's hosted release gate
- Date: 2026-08-26
- Client: Cursor Agent CLI `3.17.8` (`2026.08.11-e8db854`), macOS aarch64
- Scope: explicit project `.cursor/mcp.json`, exact local approval lifecycle,
  and bounded Agent-mode packet lifecycle

## Evidence completed

| Requirement | Result | Evidence |
| --- | --- | --- |
| Fixed project configuration | Passed | The versioned managed kit renders and validates a fixed absolute local-stdio executable, workspace, separate cache, consumer ID, and role. It rejects environment forwarding and remote fields. |
| Discovery and malformed configuration | Passed | `scripts/rehearse-cursor-preadmission.rb` discovered the isolated entry through `cursor agent mcp list`; malformed temporary `.cursor/mcp.json` loaded no server, did not expose the fixture source, and left it unchanged. |
| Native enable/list-tools/disable | Passed | `scripts/rehearse-cursor-native-approval.rb` starts from an empty explicit `/private/tmp` project, applies the owned entry, calls `cursor agent mcp enable impresari-context`, requires the four fixed tools from `list-tools`, calls `disable` for that exact identifier, and removes only the owned project entry. |
| Bounded Agent-mode lifecycle | Passed | Cursor Agent completed `context_session_open`, `context_build`, `context_packet_resolve`, and `context_session_close` in exactly that order against the disposable project. A test-only project permission file allowed only those four `Mcp(impresari-context:tool)` tokens while denying shell, file read/write, and web actions. |
| Packet equivalence | Passed | The live `context_build` packet exactly matched an independent raw-MCP control packet constructed with the same fixed request, event, purpose, timestamp, literal query, budget, workspace, and launch contract. The resolved packet exactly matched the delivered packet. |
| Source/configuration containment | Passed | The fixture source digest remained unchanged. The test disabled only its named approved server, removed only its owned `.cursor/mcp.json`, removed its test-only `.cursor/cli.json`, and refused to erase unexpected client-owned state. |

## Deliberate limits

Cursor's model chooses tools conversationally, so this is bounded live-client
smoke evidence—not prompt-repeatability or deterministic client conformance.
The deterministic gates are the fixed configuration contract, validation,
malformed-input handling, exact approval/configuration removal, source
immutability, and direct packet comparison.

Cursor Agent **Ask** mode blocked dynamic MCP execution even for the fixed
read-only server. The recorded lifecycle therefore uses Agent mode only inside
the disposable project, with an explicit test-only permission file that allows
only the four named Impresari MCP tools and denies shell, source read/write,
and web operations. It is removed before the rehearsal ends. The normal
product path preserves Cursor's ordinary approval UI; Impresari never installs
that permission file in a user project.

The exact native approval record is not replaced merely by `--approve-mcps`:
the rehearsal also performs the native `enable` and exact `disable` commands.
It never targets a user-level Cursor configuration or a real source project.

## Admission status

The local L1 evidence is complete for Cursor Agent CLI `3.17.8`
(`2026.08.11-e8db854`) on macOS aarch64. The published claim is restricted to
that scope and requires revalidation after a Cursor configuration, approval,
stream, or execution-mode change.

## Roadmap checkpoint

The Master PRD, Phase 1 PRD, ADR-0018, ADR-0035, and client-integration
roadmap were reassessed. Cursor completes the Phase 1 named-client L1 target;
the language roadmap does not change. The next independent L1 target is
GitHub Copilot CLI, followed separately by the VS Code Copilot surface.
