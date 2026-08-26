# CI-1 Cursor Guarded Project Admission — Architecture Requirements and Design

- Status: Approved for implementation
- Date: 2026-08-26
- Governing product record: [CI-1 managed connections PRD](../product/client-integration-l1-managed-connections-prd.md)
- Governing decision: [ADR-0053](../decisions/0053-cursor-guarded-project-admission.md)

## Objective

Prove Cursor Agent's project configuration, native approval lifecycle, and
bounded packet delivery without giving its Agent-mode smoke access to ordinary
shell, source, web, user configuration, or project authority.

## Design

```text
explicit empty /private/tmp project + separate cache
  -> owned .cursor/mcp.json install and validation
  -> cursor mcp list
  -> cursor mcp enable exact server
  -> cursor mcp list-tools fixed four-tool set
  -> temporary guarded .cursor/cli.json
  -> Agent-mode four-tool lifecycle + direct packet comparison
  -> cursor mcp disable exact server
  -> owned config and guarded permission-file removal
```

Ask mode is deliberately not used for the live lifecycle because the observed
client blocks dynamic MCP calls there. The temporary Agent-mode permission
contract is the compensating security boundary; it is neither user-project
configuration nor a product-installed policy.

## Invariants

1. The workspace and cache are caller-named children of `/private/tmp`; the
   real project and global Cursor configuration are never resolved.
2. The runner creates, validates, and removes only its ownership-marked MCP
   entry, its fixed test-only permission file, and its exact server approval.
3. Test-only permissions allow only the four named Impresari MCP tools and
   deny shell, source read/write, and web actions.
4. The live call order, packet identity, direct packet equivalence, source
   digest, and exact removal must all succeed or the client is not admitted.
5. Client-owned derived state outside the owned entry is never broadly erased.

## Verification

- Managed-kit unit tests and the malformed preadmission rehearsal validate the
  fixed project configuration and fail-closed inputs.
- `scripts/rehearse-cursor-native-approval.rb` records preview, native
  enable/list-tools/disable, guarded Agent-mode lifecycle, packet equivalence,
  source immutability, and exact project-file cleanup.
- Hosted CI validates Rust, contract, policy, security, documentation, and
  Ruby syntax before the public compatibility claim changes.
