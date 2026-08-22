# Slice B Structural Intelligence Traceability

## Milestone B1 — Isolated parser, canonical graph, and bounded traversal

Status: implemented locally; independent release review and native platform
confinement evidence remain open.

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Parser adapter contract | `context-structural` request/response types and length-framed worker protocol | Unit framing, strict Serde, malformed input, and process tests |
| Parser isolation | Pinned short-lived worker receives bounded source bytes, an empty working directory, and a cleared environment | Worker identity and fresh-process tests; security-boundary script |
| TypeScript/JavaScript structure | Pinned Tree-sitter runtime and TS/JS grammars extract declarations, containment, imports, and exports | Language extraction unit test |
| Canonical graph per snapshot | Content-derived graph/node/edge identities bind facts to exact snapshot, path, span, and provenance | Determinism and snapshot-binding unit test |
| Confirmed versus unresolved relationships | Graph edges record resolution; unknowns make unresolved imports/exports and syntax recovery visible | Partial-graph assertions |
| Relative module resolution | Snapshot-only resolver maps supported `./` and `../` TypeScript/JavaScript specifiers to exact file nodes; bare packages and missing files remain unresolved | Two-file resolution and unresolved-state tests |
| Local call evidence | Syntax-confirmed call sites are emitted with exact spans; unambiguous same-file name matches are heuristic targets and every other target remains unresolved | Extraction, graph-resolution, confidence, and unknown-state tests |
| Bounded trace queries | Deterministic outbound traversal enforces node, edge, depth, and serialized-output budgets | Query limit and unresolved-target assertions; engine resource mapping |
| Public contracts | Closed Draft 2020-12 structural graph and query-result schemas | Contract registry check and offline schema compilation |
| Thin CLI adapter | Explicit structural build takes a worker path/hash/empty directory; query consumes a graph file and bounded edge selection | Shared engine gateway, CLI parsing, workspace tests, and Clippy |
| Dependency accountability | Pinned parser/grammar dependencies, expanded threat model, and regenerated SPDX SBOM | Dependency policy, RustSec, license, duplicate, and SBOM gates |

## Open Slice B Work

- Add reference relationships only where syntax and resolver evidence support them.
- Add package/directory repository-map views.
- Persist the derived graph in the isolated replaceable cache.
- Support deterministic incremental graph refresh.
- Produce native macOS, Linux, and Windows confinement/resource evidence without
  overstating application-enforced isolation.
- Extend the evaluation corpus and release thresholds for structural tasks.
