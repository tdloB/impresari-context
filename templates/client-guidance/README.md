# Impresari Context native-guidance templates

These are versioned, opt-in source templates for CI-2. Copy only the template
for the client and project scope you intentionally choose. They do not install
or enable an MCP server, change client trust or approval, run commands, read
secrets, or deliver packets automatically.

| Client | Official project surface | Template destination |
| --- | --- | --- |
| Codex | project agent instructions | `AGENTS.md` (only where no unrelated project instructions would be overwritten) |
| Claude Code | project skill | `.claude/skills/impresari-context/SKILL.md` |
| Cursor | project rule | `.cursor/rules/impresari-context.mdc` |
| GitHub Copilot | path-specific repository instruction | `.github/instructions/impresari-context.instructions.md` |

Every template is deliberately small. It asks the agent to use only an already
configured local Impresari MCP server, select an explicit task profile and hard
budget, surface packet identity/reasons/omissions, and continue normally when
the server is unavailable. It grants no additional authority.

The client-specific locations are based on the documented project surfaces for
[Claude Code skills](https://code.claude.com/docs/en/slash-commands),
[Cursor project rules](https://docs.cursor.com/context/rules), and [GitHub
Copilot custom instructions](https://docs.github.com/en/copilot/reference/custom-instructions-support).
The Codex template is intentionally a stand-alone project-instruction template;
it is not installed automatically and must never replace existing project
instructions.

Do not append a template to unrelated instructions. A later managed-artifact
installer may write only these exact owned files after explicit preview and
`--apply`; until then, installation remains user-reviewed.
