# Impresari Context — Phase 4 Delivery Record: Declared Associated-Test Evidence

- Status: Implemented and accepted after full hosted CI in PR #57
- Date: 2026-08-24
- Approved by: Founder (via the approved Phase 4 roadmap and autonomous delivery directive)
- Roadmap role: Next bounded impact-evidence slice.

## Objective

Let a caller include explicitly associated current test artifacts in a
deterministic test-selection packet while verifying every source and test entry
against the same authorized current snapshot. The result must be useful for
review and handoff without claiming a test was discovered, executed, covers a
source artifact, or passes.

## Scope

- Accept a canonical caller-declared association manifest containing bounded
  source/test entry pairs, current snapshot identity, and expected current
  hashes.
- Verify every declared source and test file against the current authorized
  snapshot before exact-source evidence is recovered.
- Bind the manifest, association identity, observed current hashes, policy
  decision, budget, plan identity, and packet identity.
- Mark `associated_test` planner coverage available only for this adapter and
  label it `declared_associated_test_current_snapshot_verified`.
- Expose identical semantics through the core engine, CLI, MCP, schemas, and
  conformance tests.

## Non-goals

- Infer associations from names, paths, imports, frameworks, or repository
  conventions.
- Execute a test runner, inspect results, calculate coverage, resolve build
  targets, or claim behavioral adequacy.
- Add Git, history, network, process, compiler, package, language-server,
  environment, source-write, or client-account authority.

## Acceptance criteria

- Equal manifest, snapshot, policy, query, and budget inputs produce
  byte-stable association identity, selected evidence, plan identity, and
  packet identity.
- A foreign, stale, malformed, duplicate, self-associated, out-of-budget,
  absent, or current-hash-mismatched entry fails closed before evidence is
  emitted.
- Exact evidence records remain current-source evidence; the association stays
  explicitly caller-declared rather than observed behavior.
- Full local and hosted release gates pass before acceptance.
