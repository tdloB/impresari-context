# CI-1 GitHub Copilot CLI Trusted-Project Admission — Architecture Requirements and Design

- Status: Approved for implementation
- Date: 2026-08-26
- Governing product record: [CI-1 managed connections PRD](../product/client-integration-l1-managed-connections-prd.md)
- Governing decision: [ADR-0054](../decisions/0054-copilot-cli-trusted-project-admission.md)

## Objective

Prove GitHub Copilot CLI's native project configuration and bounded packet
delivery without placing a server in a real Copilot home, permanently trusting
a source project, or exposing ordinary client tools to the model smoke.

## Design

```text
explicit empty /private/tmp root
  -> isolated COPILOT_HOME + workspace + separate cache
  -> exact trustedFolders entry for that workspace only
  -> owned .mcp.json install and validation
  -> copilot mcp list/get project discovery
  -> prompt-mode four-tool lifecycle + direct packet comparison
  -> owned project-entry removal
  -> exact trustedFolders-entry removal
```

The prompt receives no `--additional-mcp-config`; the session must load the
project entry natively. It disables built-in MCP, remote control, automatic
update, and custom instructions. `--available-tools` exposes only the four
Impresari tools; `--allow-all-tools` and `--allow-all-paths` are restricted to
that known disposable fixture and do not make shell, file, web, or built-in
tools available.

## Invariants

1. The home, project, and cache are explicit caller-named children of
   `/private/tmp`; default client configuration and real source are never
   resolved.
2. The managed kit owns only the Impresari `.mcp.json` entry. The runner
   validates native `list/get` recognition and removes only that entry.
3. Workspace trust is explicit and isolated. Cleanup re-parses the client
   configuration and deletes only the exact temporary workspace value, while
   preserving client-generated metadata.
4. The live call order, successful structured results, delivered/resolved
   packet identity, direct packet equivalence, and source digest must all pass.
5. Any malformed configuration, unexpected trusted-folder state, unexpected
   tool, packet mismatch, or cleanup mismatch fails the admission.

## Verification

- Managed-kit unit and contract checks validate the fixed local project
  configuration, owned lifecycle, and source immutability.
- `scripts/rehearse-gemini-copilot-preadmission.rb` records malformed
  temporary configuration rejection and a bounded temporary-config control.
- `scripts/rehearse-copilot-native-project.rb` records preview, isolated
  workspace trust, native project discovery, four-tool smoke, direct packet
  equivalence, source immutability, owned-entry removal, and entry-level trust
  cleanup for the recorded CLI/OS scope.
- Hosted CI validates Rust, contracts, policy, security, documentation, and
  Ruby syntax before the public classification changes.
