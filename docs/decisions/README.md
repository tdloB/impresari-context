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
| [ADR-0008](0008-license-contributions-and-governance.md) | Apache-2.0, DCO 1.1, founder-led initial governance, owner/counsel gate | Accepted with gate |
| [ADR-0009](0009-path-and-identity-encoding.md) | Lossless native path units, canonical base64url/JCS profile, workspace identity, and exact hash envelopes | Accepted |
| [ADR-0010](0010-structural-worker-protocol-and-isolation.md) | Length-framed, capability-reduced, all-or-nothing structural parsing workers | Accepted |
| [ADR-0011](0011-process-local-session-references.md) | Consumer-scoped in-memory immutable packet references with no durable authority | Accepted |

## Change Rules

- Mark an ADR `Superseded by ADR-NNNN`; do not rewrite historical rationale.
- Correct non-semantic errors in place with normal review.
- Changes to trust boundaries, canonical evidence, authorization, hashing,
  schemas, source mutation, execution, network, extensions, hosted deployment,
  durable memory, licensing, or governance require a new ADR.
- Record implementation-specific version pins in manifests and release evidence;
  ADRs define policy and durable compatibility choices.
