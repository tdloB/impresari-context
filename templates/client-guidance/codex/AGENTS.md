# Impresari Context evidence guidance

Use an already configured local `impresari-context` MCP server only when the
user requests repository context or an evidence-backed implementation,
investigation, review, test-selection, orientation, or configuration-change
task.

- Ask for or state one explicit supported profile and a bounded evidence budget.
- Treat every packet as snapshot-bound evidence. Show its packet ID, plan ID,
  reason codes, coverage, and omissions when relying on it.
- Do not infer unsupported runtime behavior, alter MCP configuration, bypass
  client approvals, execute repository code, or expand the requested budget.
- If the MCP server or packet is unavailable, say so briefly and continue with
  ordinary repository analysis; do not fabricate evidence.
