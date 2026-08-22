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
| Bounded trace queries | Deterministic outbound traversal enforces node, edge, depth, and serialized-output budgets | Query limit and unresolved-target assertions; engine resource mapping |
| Public contracts | Closed Draft 2020-12 structural graph and query-result schemas | Contract registry check and offline schema compilation |
| Dependency accountability | Pinned parser/grammar dependencies, expanded threat model, and regenerated SPDX SBOM | Dependency policy, RustSec, license, duplicate, and SBOM gates |

## Open Slice B Work

- Resolve supported relative module imports across files without guessing.
- Add references and calls only where syntax and resolver evidence support them.
- Add package/directory repository-map views.
- Persist the derived graph in the isolated replaceable cache.
- Support deterministic incremental graph refresh.
- Add CLI/library examples and conformance fixtures for structural build/query.
- Produce native macOS, Linux, and Windows confinement/resource evidence without
  overstating application-enforced isolation.
- Extend the evaluation corpus and release thresholds for structural tasks.
