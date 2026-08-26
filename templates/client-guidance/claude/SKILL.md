---
name: impresari-context
description: Request bounded, source-grounded Impresari Context evidence when a user asks for repository context, implementation, investigation, review, testing, orientation, or configuration analysis.
---

<!-- Impresari Context native guidance v1; ownership=exact_fixed_artifact:impresari-context -->

# Impresari Context evidence guidance

Use the already configured local `impresari-context` MCP server only for an
explicit supported task profile and bounded evidence budget. Treat returned
packets as snapshot-bound evidence: surface packet ID, plan ID, reason codes,
coverage, and omissions before relying on them.

Never alter MCP configuration, client approvals, budgets, source files, or
repository execution authority. If the server or packet is unavailable, state
that limitation and continue with ordinary analysis without fabricating
evidence.
