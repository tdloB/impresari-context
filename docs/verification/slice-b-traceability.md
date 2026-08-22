# Slice B Structural Intelligence Traceability

## Milestone B1 — Isolated parser, canonical graph, and bounded traversal

Status: implemented and gated. Hosted native-platform evidence is archived
below; independent release review remains open.

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Parser adapter contract | `context-structural` request/response types and length-framed worker protocol | Unit framing, strict Serde, malformed input, and process tests |
| Parser isolation | Pinned short-lived worker receives bounded source bytes, an empty working directory, and a cleared environment | Worker identity and fresh-process tests; security-boundary script |
| TypeScript/JavaScript structure | Pinned Tree-sitter runtime and TS/JS grammars extract declarations, containment, imports, and exports | Language extraction unit test |
| Canonical graph per snapshot | Content-derived graph/node/edge identities bind facts to exact snapshot, path, span, and provenance | Determinism and snapshot-binding unit test |
| Confirmed versus unresolved relationships | Graph edges record resolution; unknowns make unresolved imports/exports and syntax recovery visible | Partial-graph assertions |
| Relative module resolution | Snapshot-only resolver maps supported `./` and `../` TypeScript/JavaScript specifiers to exact file nodes; bare packages and missing files remain unresolved | Two-file resolution and unresolved-state tests |
| Local call evidence | Syntax-confirmed call sites are emitted with exact spans; unambiguous same-file name matches are heuristic targets and every other target remains unresolved | Extraction, graph-resolution, confidence, and unknown-state tests |
| Local reference evidence | Identifier-use syntax excludes declaration-name and direct call-callee positions; a unique same-file declaration is a heuristic target and ambiguous/external names remain unresolved | Extraction, resolution, and explicit unknown-state tests |
| Bounded trace queries | Deterministic outbound traversal enforces node, edge, depth, and serialized-output budgets | Query limit and unresolved-target assertions; engine resource mapping |
| Repository-map views | A bounded graph projection reports transitive directory counts and only manifest-confirmed package boundaries; missing manifests are explicit | Deterministic map and limit tests; closed public schema |
| Isolated graph persistence | A successful graph atomically replaces the prior opaque graph payload in the existing workspace cache; every load is snapshot-scoped, decoded, and content-identity revalidated | Cache replace/scope tests and repository gate |
| Deterministic incremental refresh | Per-file worker results are keyed by lossless path, exact content hash, and all fact-affecting toolchain settings; every reuse is decoded and fully revalidated against the current bounded request | Exact-key cache tests, worker response validation, mutation/rebuild behavior |
| Public contracts | Closed Draft 2020-12 structural graph and query-result schemas | Contract registry check and offline schema compilation |
| Thin CLI adapter | Explicit structural build takes a worker path/hash/empty directory; query consumes a graph file and bounded edge selection | Shared engine gateway, CLI parsing, workspace tests, and Clippy |
| Dependency accountability | Pinned parser/grammar dependencies, expanded threat model, and regenerated SPDX SBOM | Dependency policy, RustSec, license, duplicate, and SBOM gates |
| Frozen structural evaluation | Six original TypeScript cases with a 33% held-out split gate required symbol/edge recall, identity validity, determinism, map availability, and confirmed-target honesty | `structural_evaluation` invoked by the evaluation gate |
| Native platform evidence harness | Tier-A CI runs the full repository gate on macOS/Apple silicon, Linux/x86-64, and Windows/x86-64 and emits bounded OS/architecture/toolchain/filesystem-profile evidence while disclaiming OS-sandbox guarantees | Hosted CI job summaries; release evidence remains gated on successful external runs |

## Open Slice B Work

- Complete independent release review and any maintainer-run filesystem-specific
  release rehearsal selected for the first public binary release.

## Hosted Native Evidence

- Successful full matrix: [GitHub Actions run 32559752496](https://github.com/tdloB/impresari-context/actions/runs/32559752496), commit
  `cea4a36f3d28e84cc7b429702f94d09daa15f126`, 2026-08-22.
- Passed: macOS 14/Apple silicon on Rust 1.98, Windows 2025/x86-64 on Rust
  1.98, Ubuntu 24.04/x86-64 on Rust 1.98, and Ubuntu 24.04/x86-64 on Rust
  1.96 and 1.97.
- This is application-behavior and filesystem-profile evidence, not proof of
  operating-system sandbox confinement.
