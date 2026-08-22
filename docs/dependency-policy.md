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

`cap-std` is admitted because ambient `std::fs` canonicalize/open sequences
cannot close symlink-swap races portably. Ambient authority is used exactly once
when an explicitly supplied root is opened; subsequent reads are relative to the
held directory capability. Network and UTF-8 path features are not enabled.

`jsonschema` default features are prohibited because they add HTTP, file, and TLS
resolvers. The test harness supplies every schema through an in-memory prepared
registry and must work with network access denied.

Transitive dependencies remain subject to the lockfile, advisory, license, and
duplicate review. Approval of these test dependencies does not pre-approve
SQLite, Tree-sitter, regex engines, CLI parsers, serialization derives, hashing,
or any runtime crate.

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
