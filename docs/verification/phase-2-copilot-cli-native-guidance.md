# GitHub Copilot CLI L2 native-guidance admission

- Status: passed for recorded scope
- Observed: 2026-08-26
- Client: GitHub Copilot CLI `1.0.80`, macOS aarch64
- Governing records: [CI-2 PRD](../product/client-integration-l2-native-guidance-prd.md),
  [ADR-0041](../decisions/0041-native-agent-guidance-artifacts.md), and
  [CI-2 architecture record](../architecture/ci-2-native-guidance-artifacts-ard.md)

## Recorded evidence

The authenticated Copilot CLI used an isolated caller-named Git project and
`COPILOT_HOME` under `/private/tmp`, a separate cache, a fixed source fixture,
and the exact owned
`.github/instructions/impresari-context.instructions.md` v2 artifact. The
rehearsal explicitly created the required instruction parent directory,
installed and validated the artifact, then started a new Copilot session
without `--no-custom-instructions`. Custom instructions were therefore enabled
only for this disposable session.

Copilot natively discovered the fixed project MCP entry after the exact
temporary workspace-trust transition. Its available tool surface was limited to
the four named Impresari MCP tools; built-in MCP, remote control, automatic
update, and all other tools were disabled. With the owned instruction requested
for the probe file, Copilot completed:

1. `context_session_open`
2. `context_build`
3. `context_packet_resolve`
4. `context_session_close`

The resolved packet matched the delivered packet and an independent direct-MCP
control packet. The rehearsal then removed the exact owned instruction, the
MCP entry, the project instruction directories, and only its temporary trusted
folder value. The source fixture remained byte-identical before removal.

The reusable command is:

```text
ruby scripts/rehearse-copilot-native-project.rb \
  --temporary-root <prepared-private-tmp-root> \
  --native-guidance-smoke
```

Prepare the root first through the script's preview/apply flow. It requires an
already authenticated Copilot CLI and never selects a real Copilot home or a
user source project.

## Limits

GitHub Copilot CLI loads repository custom instructions at new-session start,
but a model's resulting tool choices remain conversational behavior. This is
live recorded-scope smoke evidence, not a claim that repeated prompts select
the same instruction set or MCP tools. It does not change a user's global
Copilot configuration, trust, source-write, execution, network, delivery,
memory, or orchestration authority. VS Code Copilot is a separate client
surface and is not covered by this record.

Revalidate on a Copilot CLI, custom-instructions, trust, project-MCP,
tool-result, or platform change.
