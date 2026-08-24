# Dependency Policy

Every dependency addition requires a narrow purpose, current registry metadata,
license and MSRV review, feature minimization, lockfile evidence, vulnerability
review, and L06/Agent 45 approval. Runtime and build dependencies receive more
scrutiny than test-only dependencies.

## Initial approved test dependencies

| Package | Exact version | Scope | Features | License | MSRV | Purpose |
| --- | --- | --- | --- | --- | --- | --- |
| `serde_json` | 1.0.151 | Test only | `std`; defaults disabled | MIT OR Apache-2.0 | 1.71 | Parse schema and fixture JSON |
| `jsonschema` | 0.50.0 | Test only | No features; defaults disabled | MIT | 1.85 | Full Draft 2020-12 conformance validation |
| `sha2` | 0.11.0 | Test only | Defaults disabled | MIT OR Apache-2.0 | 1.85 | Reproduce published SHA-256 identity vectors |

## Initial approved runtime dependencies

| Package | Exact version | Scope | Features | License | MSRV evidence | Purpose |
| --- | --- | --- | --- | --- | --- | --- |
| `base64` | 0.22.1 | `context-workspace` | `std`; defaults disabled | MIT OR Apache-2.0 | Verified by the project Rust 1.96 gate | Canonical unpadded base64url native-path units |
| `cap-std` | 4.0.2 | `context-workspace` | No features; defaults disabled | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | Upstream documents Rust 1.70 with defaults; verified by the project Rust 1.96 gate | Capability-relative, cross-platform filesystem access resistant to path escapes |
| `sha2` | 0.11.0 | `context-workspace` and conformance tests | Defaults disabled | MIT OR Apache-2.0 | Verified by the project Rust 1.96 gate | Domain-separated workspace and content identities |
| `rusqlite` | 0.40.2 | `context-store` | `bundled`, `limits`; defaults disabled | MIT | Verified by the project Rust 1.96 gate | Transactional isolated cache with a pinned bundled SQLite and runtime limits |
| `serde` | 1.0.229 | `context-core` | `derive`, `std`; defaults disabled | MIT OR Apache-2.0 | Verified by the project Rust 1.96 gate | Contract serialization with explicit derives |
| `serde_json` | 1.0.151 | `context-core` and conformance tests | `std`; defaults disabled | MIT OR Apache-2.0 | 1.71 upstream | Strict JSON value and wire encoding |
| `serde_json_canonicalizer` | 0.3.2 | `context-core` | Defaults disabled | MIT | Verified by the project Rust 1.96 gate | RFC 8785 JCS bytes for identities and hard packet accounting |
| `toml` | 1.1.4+spec-1.1.0 | `context-cli` | `parse`, `serde`, `std`; defaults disabled | MIT OR Apache-2.0 | Upstream Rust 1.85; project Rust 1.96 gate required | Strictly parse the user-supplied Codex local MCP configuration for read-only validation; no serializer, preserve-order, debug, fast-hash, network, or file-loading feature is enabled |
| `tree-sitter` | 0.26.12 | `context-structural` worker only | `std`; defaults disabled; WASM disabled | MIT | Upstream 1.77; project Rust 1.96 gate required | Bounded concrete-syntax parsing behind the ADR-0010 process boundary |
| `tree-sitter-javascript` | 0.25.0 | `context-structural` worker only | Defaults disabled | MIT | Project Rust 1.96 gate required | Pinned JavaScript/JSX grammar |
| `tree-sitter-go` | 0.25.0 | `context-structural` worker only | Defaults disabled | MIT | Project Rust 1.96 gate required | Pinned Go grammar |
| `tree-sitter-java` | 0.23.5 | `context-structural` worker only | Defaults disabled | MIT | Project Rust 1.96 gate required | Pinned Java grammar |
| `tree-sitter-json` | 0.24.8 | `context-structural` worker only | Defaults disabled | MIT | Project Rust 1.96 gate required | Pinned strict-JSON grammar |
| `tree-sitter-python` | 0.25.0 | `context-structural` worker only | Defaults disabled | MIT | Project Rust 1.96 gate required | Pinned Python grammar |
| `tree-sitter-rust` | 0.24.2 | `context-structural` worker only | Defaults disabled | MIT | Project Rust 1.96 gate required | Pinned Rust grammar |
| `tree-sitter-toml-ng` | 0.7.0 | `context-structural` worker only | Defaults disabled | MIT | Project Rust 1.96 gate required | Pinned TOML grammar; native parser remains inside the isolated worker |
| `tree-sitter-typescript` | 0.23.2 | `context-structural` worker only | Defaults disabled | MIT | Project Rust 1.96 gate required | Pinned TypeScript/TSX grammars |
| `tree-sitter-yaml` | 0.7.2 | `context-structural` worker only | Defaults disabled | MIT | Project Rust 1.96 gate required | Pinned YAML grammar; raw direct mapping keys only, with the native parser inside the isolated worker |

`cap-std` is admitted because ambient `std::fs` canonicalize/open sequences
cannot close symlink-swap races portably. Ambient authority is used exactly once
when an explicitly supplied root is opened; subsequent reads are relative to the
held directory capability. Network and UTF-8 path features are not enabled.

`rusqlite` defaults are disabled. The `bundled` feature avoids an unknown system
SQLite and supplies FTS5; `limits` permits defensive SQLite runtime ceilings.
`bundled-full`, `load_extension`, `loadable_extension`, WASM, SQLCipher, and
serialization/integration features are prohibited. SQL is project-authored and
callers never receive a raw SQL or FTS query surface.

The canonicalizer is used only after typed/schema-constrained construction.
Untrusted raw JSON is never canonicalized directly, preventing duplicate-key
normalization from becoming an acceptance path. All identity-bearing numeric
values remain safe integers or canonical decimal strings per ADR-0009.

Tree-sitter and its grammar crates compile native C and therefore expand the
supply-chain and memory-safety boundary. They are admitted only inside the
short-lived structural worker defined by ADR-0010. Dynamic grammar loading,
WASM, grammar downloads, repository configuration, compiler plugins, language
servers, and in-process control-plane parsing remain prohibited. Worker bytes
are treated as hostile and become graph facts only after full response,
identity, span, provenance, ordering, and limit validation.

`jsonschema` default features are prohibited because they add HTTP, file, and TLS
resolvers. The test harness supplies every schema through an in-memory prepared
registry and must work with network access denied.

Transitive dependencies remain subject to the lockfile, advisory, license, and
duplicate review. Approval of these dependencies does not pre-approve
additional grammars, regex engines, CLI parsers, serialization derives,
hashing, or runtime crates.

## Allowed license baseline

Apache-2.0, Apache-2.0 WITH LLVM-exception, MIT, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, and compatible
permissive combinations may be considered. Copyleft, source-available, unknown,
unlicensed, deprecated, abandoned, or native/vendored dependencies require
explicit escalation; this list is a review baseline, not automatic approval.

## Update and exception rules

- Exact direct versions remain pinned during pre-release contract work.
- Security fixes may advance a dependency after the same review and full matrix.
- Advisory or license exceptions require owner, rationale, affected surface,
  compensating control, expiry date, and M04 approval.
- Unused, duplicate-major, or default-feature expansion fails review unless
  justified by measured need.
