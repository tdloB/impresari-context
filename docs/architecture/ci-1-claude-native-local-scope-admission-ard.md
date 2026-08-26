# CI-1 Claude Code Native Local-Scope Admission — Architecture Requirements and Design

- Status: Approved for implementation
- Date: 2026-08-26
- Governing product record: [CI-1 managed connections PRD](../product/client-integration-l1-managed-connections-prd.md)
- Governing decision: [ADR-0052](../decisions/0052-claude-disposable-local-scope-admission.md)

## Objective

Prove that Claude Code recognizes and precisely removes the fixed local-stdio
connection contract without granting the test access to a user's real Claude
home or source workspace.

## Design

```text
explicit empty /private/tmp HOME + workspace
  -> native claude mcp add --scope local
  -> claude mcp get fixed-contract recognition
  -> claude mcp remove --scope local exact entry
  -> claude mcp get absence

separate strict temporary configuration
  -> malformed-client rejection
  -> bounded model-directed four-tool lifecycle
  -> direct-MCP packet equivalence
```

The two paths are intentionally separate. The native path admits the
configuration lifecycle; the strict temporary path measures the live
conversational client against an allowlisted, bounded MCP operation sequence.

## Invariants

1. The home, workspace, and cache are explicit caller-named children of
   `/private/tmp`; a default real home is never resolved.
2. The native rehearsal starts only with an empty home and workspace and
   registers/removes only the named `impresari-context` entry.
3. Configuration contains an absolute executable, fixed workspace/cache,
   consumer ID, and role; it neither forwards environment variables nor opens
   remote transport.
4. The source fixture digest must remain unchanged across registration,
   inspection, lifecycle use, and removal.
5. The runner does not erase client-owned runtime metadata in its disposable
   home; it proves only that the Impresari entry is absent.
6. Model tool choice is never treated as deterministic proof.

## Verification

- Unit/contract gates validate the fixed Claude configuration, owned managed
  connection lifecycle, malformed inputs, and source immutability.
- `scripts/rehearse-claude-native-local-scope.rb` gives preview, explicit
  preparation, add/get/remove, absence, and immutable-source evidence for the
  recorded CLI/OS scope.
- `scripts/rehearse-claude-code.rb` records strict malformed configuration
  rejection, allowlisted four-tool smoke evidence, and direct-packet
  equivalence.
- Hosted CI validates Rust, contracts, policy, security, documentation, and
  Ruby syntax before public classification changes.
