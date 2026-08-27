# ADR-0054: GitHub Copilot CLI trusted-project admission

- Status: Accepted for implementation
- Date: 2026-08-26
- Scope: GitHub Copilot CLI L1 project configuration, workspace-trust, and lifecycle evidence

## Context

GitHub Copilot CLI `1.0.80` on macOS aarch64 recognizes project MCP entries
in `.mcp.json` through `copilot mcp list/get`, but prompt mode intentionally
skips workspace MCP servers until the workspace is trusted. A temporary
additional-MCP configuration can demonstrate tool transport, but it does not
prove that Copilot loads the product's supported project configuration path.

The product needs an auditable project-scope admission without changing a
person's real Copilot home, folder-trust list, user-level server registry, or
source repository.

## Decision

GitHub Copilot CLI L1 admission uses an explicit empty caller-named root under
`/private/tmp` containing an isolated `COPILOT_HOME`, project workspace, and
separate cache. The rehearsal applies and validates only the owned `.mcp.json`
entry in that workspace, then confirms its native discovery with `copilot mcp
list --json` and `copilot mcp get impresari-context --json`.

Because prompt mode cannot interactively request folder trust, the rehearsal
adds exactly that disposable workspace to `trustedFolders` in the isolated
home's `config.json`. It re-reads the file after the client starts and removes
only that exact trusted-folder entry. Client-generated metadata in the
disposable home is preserved rather than broadly deleted.

The native prompt session receives no additional MCP configuration. It disables
built-in MCP servers, remote control, automatic update, and custom
instructions. Its available-tool set is the four named Impresari MCP tools;
automatic approval and path approval apply only to that already restricted
test fixture. The live session must call the four session/packet tools in
order, return successful structured results, resolve the delivered packet, and
match an independent direct MCP control packet.

## Constraints

- No real Copilot home, project, user-level MCP entry, sign-in, remote
  transport, environment forwarding, source mutation, source read/write tool,
  web tool, prompt injection, or background authority is targeted.
- The runner removes only its exact owned `.mcp.json` entry and its exact
  temporary trusted-folder value. It fails closed if either target differs.
- The model-directed sequence is bounded live-client smoke evidence, not a
  claim that repeating a prompt deterministically repeats tool calls.
- First-class support is restricted to the recorded Copilot CLI version and
  macOS architecture until revalidated.

## Consequences

GitHub Copilot CLI can be classified First-class for its recorded scope while
keeping workspace trust explicit, isolated, and reversible. A Copilot change to
project-configuration precedence, folder trust, tool naming, approval
semantics, or JSONL result shape requires revalidation and can demote the
claim. VS Code Copilot remains a distinct, unadmitted client surface.

## References

- [CI-1 managed connections PRD](../product/client-integration-l1-managed-connections-prd.md)
- [GitHub Copilot CLI connection-kit record](../verification/phase-2-copilot-cli-connection-kit.md)
- [ADR-0018: client compatibility contract](0018-first-class-client-integration-and-compatibility-contract.md)
- [ADR-0035: managed connection kits](0035-l1-managed-client-connection-kits.md)
- [GitHub Copilot CLI MCP configuration](https://docs.github.com/en/enterprise-cloud@latest/copilot/how-tos/copilot-cli/customize-copilot/add-mcp-servers)
- [GitHub Copilot CLI folder trust](https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/configure-copilot-cli)
