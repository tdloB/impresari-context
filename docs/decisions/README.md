# Architecture Decision Record Index

ADRs record durable product, security, implementation, and governance choices.
They do not override the Master PRD, MVP PRD, Security Threat Model, or Evaluation
PRD; conflicts require an explicit review and superseding record.

| ADR | Decision | Status |
| --- | --- | --- |
| [ADR-0001](0001-independent-core-and-thin-adapters.md) | Independent public core with thin consumer adapters | Accepted |
| [ADR-0002](0002-rust-core-runtime.md) | Stable Rust 2024 core runtime | Accepted |
| [ADR-0003](0003-supported-platform-matrix.md) | Tier A macOS ARM64, Linux x86-64 GNU, and Windows x86-64 MSVC | Accepted |
| [ADR-0004](0004-source-language-and-parser-strategy.md) | UTF-8 lexical MVP; TypeScript/JavaScript first structural family using isolated Tree-sitter workers | Accepted |
| [ADR-0005](0005-hashing-serialization-and-schema.md) | SHA-256, RFC 8785 canonical JSON, JSON Schema 2020-12, byte-authoritative spans | Accepted |
| [ADR-0006](0006-local-cache-and-storage.md) | Per-workspace SQLite derived cache with FTS5 and rollback journaling | Accepted |
| [ADR-0007](0007-context-budget-accounting.md) | UTF-8 serialized bytes as the MVP hard context-budget unit | Accepted |
| [ADR-0008](0008-license-contributions-and-governance.md) | Apache-2.0, DCO 1.1, founder-led initial governance, BoldtHaus Studio, LLC stewardship | Accepted |
| [ADR-0009](0009-path-and-identity-encoding.md) | Lossless native path units, canonical base64url/JCS profile, workspace identity, and exact hash envelopes | Accepted |
| [ADR-0010](0010-structural-worker-protocol-and-isolation.md) | Length-framed, capability-reduced, all-or-nothing structural parsing workers | Accepted |
| [ADR-0011](0011-process-local-session-references.md) | Consumer-scoped in-memory immutable packet references with no durable authority | Accepted |
| [ADR-0012](0012-context-plans-consumer-adapters-and-fallback.md) | Multi-strategy context plans, thin adapters, and fail-closed native-read fallback | Accepted |
| [ADR-0013](0013-extension-contracts-without-code-loading.md) | Pinned extension declarations and metadata-only quarantine without code loading | Accepted |
| [ADR-0014](0014-local-stdio-mcp-transport.md) | Local stdio MCP as a thin authority-neutral transport | Accepted |
| [ADR-0015](0015-release-candidate-builds-and-provenance.md) | Manual release-candidate builds, checksums, provenance, and rehearsal | Accepted |
| [ADR-0016](0016-release-signing-and-attestations.md) | GitHub artifact attestations plus SHA-256 checksums for v0.1.0 | Accepted |
| [ADR-0017](0017-v0.1-release-assurance-policy.md) | Independent review deferred from the v0.1.0 gate to a pre-v1.0.0 assurance target, subject to earlier risk triggers | Accepted |
| [ADR-0018](0018-first-class-client-integration-and-compatibility-contract.md) | Versioned language/client compatibility matrices, opt-in connection kits, and read-only diagnostics | Accepted |
| [ADR-0019](0019-python-structural-language-admission.md) | Pinned Python Tree-sitter structural facts without interpreter or environment authority | Accepted |
| [ADR-0020](0020-strict-json-configuration-evidence.md) | Pinned strict-JSON configuration facts without schema or runtime semantics | Accepted |
| [ADR-0021](0021-go-structural-language-admission.md) | Pinned Go Tree-sitter structural facts without toolchain or module-resolution authority | Accepted |
| [ADR-0022](0022-rust-structural-language-admission.md) | Pinned Rust Tree-sitter structural facts without Cargo or compiler-resolution authority | Accepted |
| [ADR-0023](0023-revised-product-roadmap-sequencing.md) | Align language, client, planner, and enterprise work to the founder-approved five-phase roadmap | Accepted |
| [ADR-0024](0024-deterministic-context-planner.md) | Add a bounded deterministic planner without agent-governance authority | Accepted; approved initial scope implemented |
| [ADR-0025](0025-jsonc-configuration-evidence.md) | Separate bounded JSONC evidence from strict JSON and fail closed on non-strict JSON | Accepted |
| [ADR-0026](0026-toml-configuration-evidence.md) | Pinned TOML syntax facts in the isolated worker without configuration evaluation | Accepted |
| [ADR-0027](0027-yaml-configuration-evidence.md) | Pinned bounded YAML mapping-key facts without alias or consumer semantics | Accepted |
| [ADR-0028](0028-codex-deterministic-mcp-tool-conformance.md) | Deterministic Codex App Server direct-tool conformance without model-directed selection | Accepted |
| [ADR-0029](0029-progressive-client-integration-depth-and-consent.md) | Progressive client integration depth with explicit consent and evidence | Accepted |
| [ADR-0030](0030-java-structural-language-admission.md) | Pinned Java Tree-sitter structural facts without compiler or classpath resolution authority | Accepted |
| [ADR-0031](0031-kotlin-structural-language-admission.md) | Pinned Kotlin Tree-sitter structural facts without compiler or Gradle resolution authority | Accepted |
| [ADR-0032](0032-csharp-structural-language-admission.md) | Pinned C# Tree-sitter structural facts without compiler, MSBuild, or project-resolution authority | Accepted |
| [ADR-0033](0033-structural-impact-planner-admission.md) | Bind structural-impact planner evidence only to validated current-snapshot graphs | Accepted for implementation |
| [ADR-0034](0034-declared-change-set-packets.md) | Bind caller-declared change-set packets only to current snapshot membership and hashes | Accepted for implementation |
| [ADR-0035](0035-l1-managed-client-connection-kits.md) | Use previewable, owned, manifest-driven L1 client connection kits | Accepted for implementation |
| [ADR-0036](0036-declared-associated-test-evidence.md) | Bind caller-declared source-to-test associations only to verified current snapshot artifacts | Accepted for implementation |
| [ADR-0037](0037-repository-orientation-packets.md) | Bind repository orientation only to bounded, validated current structural maps | Accepted for implementation |
| [ADR-0038](0038-incremental-structural-updates.md) | Apply structural changes only through verified explicit current-snapshot update manifests | Accepted for implementation |
| [ADR-0039](0039-convention-and-exemplar-evidence.md) | Bind convention/exemplar context only to caller-declared verified current artifacts | Accepted for implementation |
| [ADR-0040](0040-demand-led-language-admission.md) | Select Phase 5 languages only from documented demand and evaluation evidence | Accepted |

## Change Rules

- Mark an ADR `Superseded by ADR-NNNN`; do not rewrite historical rationale.
- Correct non-semantic errors in place with normal review.
- Changes to trust boundaries, canonical evidence, authorization, hashing,
  schemas, source mutation, execution, network, extensions, hosted deployment,
  durable memory, licensing, or governance require a new ADR.
- Record implementation-specific version pins in manifests and release evidence;
  ADRs define policy and durable compatibility choices.
