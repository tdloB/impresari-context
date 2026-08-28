# CI-2 VS Code Copilot native-guidance ARD

- Status: Approved implementation design; L2 admission pending live evidence
- Date: 2026-08-27
- Governing product record: [CI-2 VS Code Copilot native-guidance PRD](../product/ci-2-vscode-copilot-native-guidance-prd.md)
- Governing decisions: [ADR-0041](../decisions/0041-native-agent-guidance-artifacts.md), [ADR-0045](../decisions/0045-owned-native-guidance-artifact-lifecycle.md), and [ADR-0058](../decisions/0058-vscode-copilot-native-guidance-and-tool-schema-ergonomics.md)

## Decision

Keep Copilot guidance as one exact-owned project file at
`.github/instructions/impresari-context.instructions.md`. VS Code discovers
that supported workspace instruction location; the same static v3 artifact is
also reusable by GitHub Copilot CLI. The guidance never configures MCP or
implements a packet itself. It points the model to the live MCP `context_build`
input schema and describes only a canonical ordering and mutually exclusive
request forms.

The MCP tool description supplies the dynamic contract that must never be
frozen into static guidance: the current policy-profile fingerprint, decimal
budget grammar, identifier pattern, and allowed request fields. This keeps the
instruction stable while the transport remains the single normative source for
request validation.

VS Code Copilot `1.134.0` accepts a narrower tool-schema subset and rejects
top-level `oneOf` before tool arguments are formed. The published
`context_build` schema is therefore a flat closed object: required common
fields, optional form-specific fields, explicit descriptions, and canonical
decimal-string budget fields. The server remains the authoritative exclusive
grammar. Its deny-unknown deserialization and exhaustive form match reject
mixed or incomplete direct, planner, structural, change-set, associated-test,
and orientation requests before engine work. Flattening the client schema does
not make an invalid combination valid.

## Canonical evidence lifecycle

```text
operator enables owned guidance in disposable workspace
    -> model opens a bounded local session
    -> model builds one packet using either explicit steps OR profile + query
    -> model resolves the returned packet ID in that same session
    -> model closes the session
    -> operator records only actual tool results
    -> exact owned guidance/configuration removal
```

For a direct file or term request, the preferred path is one-to-eight explicit
`steps`; that form excludes `profile`, `query`, and every specialised
structural declaration. For planner-backed evidence, the preferred path is one
supported `profile` and `query`; the forms cannot be combined. Every path still
requires caller-generated request/event identifiers, an RFC 3339 occurrence
time, and the complete current hard budget from the live schema.

## Security and failure boundary

- Static guidance contains no secret, executable command, shell construct,
  configuration write, approval instruction, or literal policy fingerprint.
- The client owns trust, server start, tool approval, and chat submission.
- A failed build yields no packet. The model may continue ordinary analysis,
  but it must clearly distinguish it from packet-backed evidence.
- The L2 rehearsal operates only in a caller-named `/private/tmp` root and
  proves exact configuration and guidance removal before reporting a record.
- This design has no automatic packet injection or delivery; CI-3 remains the
  separately consented planner-backed delivery gate.

## Alternatives rejected

- **Implicit server budgets:** would hide resource selection and weaken the
  hard-budget evidence contract.
- **A VS Code extension or prompt-rewriting proxy:** would broaden authority,
  add a persistent integration surface, and duplicate transport logic.
- **Embedding current policy values in guidance:** would make an exact-owned
  static artifact stale when a live protocol contract changes.
- **Treating a local-file fallback as a packet:** would misrepresent evidence
  provenance and defeat the product's security claim.
