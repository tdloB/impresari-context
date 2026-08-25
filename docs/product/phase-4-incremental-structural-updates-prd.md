# Impresari Context — Phase 4 Incremental Structural Updates Delivery Record

- Status: Proposed for implementation
- Date: 2026-08-25
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)
- Related ADR: [ADR-0038](../decisions/0038-incremental-structural-updates.md)

## Objective

Allow a caller to explicitly replace a bounded set of structural-file results
when it has already built a new snapshot, without treating the request as a
filesystem watcher, Git diff, or background synchronization service.

## In scope

- An explicit update manifest containing the expected prior graph identity,
  current snapshot identity, and a bounded set of current artifact parser
  results.
- Exact validation of artifact membership and content hashes against the
  current authorized snapshot.
- Deterministic graph replacement and identity recomputation after the update.
- Explicit changed, removed, unresolved, and limited outcomes.
- Shared engine, CLI, MCP, schema, test, and evaluation coverage.

## Out of scope

- File watching, polling, background workers, durable synchronization, Git
  history or diff discovery, source writes, compiler/language-server calls,
  package resolution, and network access.
- Claims that an update describes all changes outside its declared current
  snapshot or that it captures runtime behavior.

## Acceptance criteria

- The engine fails closed for stale graphs, stale snapshots, duplicate paths,
  hash mismatches, malformed worker results, and budget violations.
- All retained graph content remains snapshot-bound and canonical; the new
  graph identity changes only according to the verified declared update.
- A caller can identify exactly which declared artifacts were replaced or
  removed and which limits/unknowns apply.
- Full local quality, security, schema, evaluation, and hosted CI gates pass.
