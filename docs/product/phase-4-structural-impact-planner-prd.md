# Impresari Context — Phase 4 Delivery Record: Structural Impact Planner

- Status: Implemented and accepted after full hosted CI in PR #49
- Date: 2026-08-24
- Approved by: Founder (via the approved Phase 4 roadmap)
- Roadmap role: First impact-evidence slice after Java, Kotlin, and C# admission.

## Objective

Make the deterministic context planner able to use an already validated,
snapshot-matched structural graph for bounded relationship and impact evidence.
The slice must improve planner coverage without inventing source relationships,
change history, test association, or semantic behavior.

## Scope

- Accept only a complete, integrity-validated structural graph whose workspace
  snapshot equals the current authorized snapshot.
- Select explicit bounded graph-traversal candidates using declared node,
  edge-kind, node-count, edge-count, depth, and byte budgets.
- Return graph and traversal identities, reason codes, confirmed/heuristic/
  unresolved state, and every traversal-limit or unavailable-state reason.
- Make `structural_relationship` planner coverage available only when this
  exact adapter is active; preserve the existing unavailable state otherwise.
- Expose the same semantics through the core engine, CLI, MCP transport, and
  conformance tests.

## Non-goals

- Git commands, revision-diff parsing, working-tree interpretation, or any
  claim that a structural graph identifies a change set.
- Compiler, language-server, build, package, project, test-runner, network,
  process, environment, or source-write authority.
- Reachability, impact certainty, call dispatch, aliasing, inheritance,
  dependency injection, reflection, or runtime inference.
- Associated-test, configuration-to-code, convention, exemplar, or incremental
  update evidence. Each requires a later separately admitted slice.

## Acceptance criteria

- A graph from another workspace, a stale snapshot, malformed graph, unknown
  edge kind, or exhausted traversal/packet budget fails closed or is explicitly
  reported without evidence leakage.
- Equal declared inputs produce byte-stable traversal selection, coverage,
  omission reasons, plan identity, and packet identity.
- Every selected relationship remains recoverable to an exact graph edge and
  source span with parser, grammar, resolver, and graph provenance.
- The full local gate plus hosted macOS, Linux, Windows, fuzzing, static
  analysis, and dependency-security checks pass before acceptance.
