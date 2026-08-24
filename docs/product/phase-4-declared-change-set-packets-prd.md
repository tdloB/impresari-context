# Impresari Context — Phase 4 Delivery Record: Declared Change-Set Packets

- Status: Accepted for implementation after structural-impact planner admission
- Date: 2026-08-24
- Approved by: Founder (via the approved Phase 4 roadmap and autonomous delivery directive)
- Roadmap role: Next bounded impact-evidence slice.

## Objective

Give a caller a safe way to focus a deterministic context packet on files it
declares changed, while verifying every declared file against the current
authorized snapshot. The feature must be useful for code review and handoff
without claiming the engine computed a Git diff or verified history.

## Scope

- Accept a canonical caller-declared manifest with current snapshot identity,
  bounded path entries, and expected current content hashes.
- Verify all entries against one authorized current snapshot before emitting
  packet evidence.
- Bind the manifest, its declaration identity, observed hashes, optional base
  revision assertion, policy decision, budget, and final packet identity.
- Mark `change_set` planner coverage available only for this adapter and label
  it `declared_change_set_current_snapshot_verified`.
- Emit explicit unknowns for an asserted base revision that is absent or does
  not match the existing non-mutating repository metadata.
- Expose identical semantics through the core engine, CLI, MCP, schemas, and
  conformance tests.

## Non-goals

- Execute Git, inspect a working tree, parse a revision diff, access Git
  objects beyond existing bounded metadata, find a merge base, or prove a path
  was changed from a historical revision.
- Treat an asserted base revision as observed historical evidence.
- Add network, process, compiler, language-server, build, package, test-runner,
  environment, source-write, patch-application, or client-account authority.
- Infer associated tests, runtime impact, call reachability, conventions,
  exemplars, configuration-to-code relations, or incremental updates.

## Acceptance criteria

- Equal manifest, snapshot, policy, profile, query, and budget inputs produce
  byte-stable selection, coverage, declaration identity, plan identity, and
  packet identity.
- A foreign, stale, malformed, duplicate/conflicting, out-of-budget, absent,
  or current-hash-mismatched entry fails closed before packet evidence is
  emitted.
- Every selected file is recovered as current exact-source evidence, and all
  asserted information remains labeled as asserted rather than observed.
- Full local and hosted release gates pass before the capability is accepted.
