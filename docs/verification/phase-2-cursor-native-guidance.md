# Cursor L2 native-guidance admission

- Status: passed for recorded scope
- Observed: 2026-08-26
- Client: Cursor Agent CLI `3.17.8` (`2026.08.11-e8db854`), macOS aarch64
- Governing records: [CI-2 PRD](../product/client-integration-l2-native-guidance-prd.md),
  [ADR-0041](../decisions/0041-native-agent-guidance-artifacts.md), and
  [CI-2 architecture record](../architecture/ci-2-native-guidance-artifacts-ard.md)

## Recorded evidence

The authenticated Cursor Agent CLI used an isolated caller-named project under
`/private/tmp`, a separate cache, a fixed source fixture, and the exact owned
`.cursor/rules/impresari-context.mdc` v2 artifact. The rehearsal created the
required rule parent directory, explicitly installed and validated the rule,
then used Agent mode with a project permission file that allowed only the four
Impresari MCP lifecycle tools and denied shell, source read/write, and web
actions.

Cursor discovered the fixed local-stdio server, enabled only
`impresari-context`, listed the four expected tools, and completed the
model-directed lifecycle after being asked to apply the owned project rule:

1. `context_session_open`
2. `context_build`
3. `context_packet_resolve`
4. `context_session_close`

The resolved packet matched the delivered packet and the direct-MCP control
packet. The rehearsal then disabled only the named Cursor server and exactly
removed the owned rule, MCP entry, permission file, and source fixture. The
workspace was empty after cleanup; source immutability was verified before and
after removal.

The reusable command is:

```text
ruby scripts/rehearse-cursor-native-approval.rb \
  --temporary-root <prepared-private-tmp-root> \
  --native-guidance-smoke
```

Prepare the root first through the script's preview/apply flow. It requires an
already authenticated Cursor Agent CLI and never targets a user project or
global Cursor configuration.

## Limits

Cursor project rules are a native persistent guidance surface, but the model's
decision to select an agent-requested rule and its tool calls remain
conversational behavior. This is live recorded-scope smoke evidence, not a
claim that repeating a prompt produces the same rule selection or tool calls.
Ask mode blocks dynamic MCP execution and is not an L2 lifecycle surface.
Nothing in this admission changes a user IDE's trust, approval, source-write,
execution, network, delivery, memory, or orchestration authority.

Revalidate on a Cursor CLI, project-rules, approval, output-stream, MCP, or
platform change.
