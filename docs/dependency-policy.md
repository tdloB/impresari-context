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

`jsonschema` default features are prohibited because they add HTTP, file, and TLS
resolvers. The test harness supplies every schema through an in-memory prepared
registry and must work with network access denied.

Transitive dependencies remain subject to the lockfile, advisory, license, and
duplicate review. Approval of these test dependencies does not pre-approve
SQLite, Tree-sitter, regex engines, CLI parsers, serialization derives, hashing,
or any runtime crate.

## Allowed license baseline

Apache-2.0, MIT, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0, and compatible
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
