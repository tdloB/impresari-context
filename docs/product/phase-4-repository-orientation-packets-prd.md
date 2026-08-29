# Impresari Context — Phase 4 Delivery Record: Repository-Orientation Packets

- Status: Implemented and accepted after full hosted CI in PR #58
- Date: 2026-08-24
- Approved by: Founder (via the approved roadmap and autonomous delivery directive)
- Roadmap role: Next bounded Phase 4 impact-evidence slice.

## Objective

Provide a deterministic, snapshot-bound orientation packet built from the
existing validated structural repository map plus exact source evidence. It
helps a reviewer or coding client locate bounded entry points and declared
relationships without presenting a generated repository summary as fact.

## Scope

- Accept only a complete, integrity-validated structural graph for the current
  authorized snapshot.
- Use the existing bounded repository-map algorithm and declared item limit.
- Bind graph identity, repository-map identity, selected exact evidence,
  coverage, omissions, policy decision, plan identity, and packet identity.
- Make orientation-map evidence available only for this adapter; ordinary
  orientation profiles retain their existing filename/lexical behavior.
- Expose identical engine, CLI, MCP, schema, and conformance semantics.

## Non-goals

- Generate model summaries, infer architecture or conventions, rank business
  importance, resolve runtime behavior, execute code, or infer test coverage.
- Add Git, history, network, process, compiler, package, language-server,
  source-write, environment, or client-account authority.

## Acceptance criteria

- Foreign, stale, malformed, incomplete, or over-budget graphs fail closed or
  report their bounded omissions without source leakage.
- Equal graph, snapshot, query, policy, and budget inputs yield byte-stable map
  selection, plan identity, evidence, and packet identity.
- Every selected item remains recoverable to exact current source and declared
  structural provenance; omitted or unsupported graph information is explicit.
- Full local and hosted release gates pass before acceptance.
