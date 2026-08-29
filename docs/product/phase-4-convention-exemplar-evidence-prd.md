# Impresari Context — Phase 4 Convention and Exemplar Evidence Delivery Record

- Status: Implemented and accepted after full hosted CI in PR #60
- Date: 2026-08-25
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)
- Related ADR: [ADR-0039](../decisions/0039-convention-and-exemplar-evidence.md)

## Objective

Enable reviewable convention and exemplar context packets from a caller’s
explicit selections, while preserving the distinction between a caller’s
assertion and exact current-source evidence.

## In scope

- A bounded caller declaration of named convention labels and exact current
  artifact hashes.
- Verification of every artifact against the authorized current snapshot.
- Exact source recovery and a canonical declaration identity in a deterministic
  planner packet.
- Explicit coverage and omission states for convention/exemplar evidence.
- Shared engine, CLI, MCP, schema, test, and evaluation support.

## Out of scope

- Inferring, ranking, recommending, scoring, or enforcing conventions.
- Repository mining, embeddings, LLM analysis, ownership/maintainer claims,
  runtime claims, Git history, source mutation, execution, or network access.

## Acceptance criteria

- Labels are bounded opaque caller data and never promoted into observed facts.
- All paths and hashes are current-snapshot verified; malformed, duplicate,
  stale, or over-budget declarations fail closed.
- The packet preserves exact evidence plus explicit caller assertion without
  calling it a discovered convention or representative sample.
- Full local and hosted security, quality, contract, evaluation, and CI gates pass.
