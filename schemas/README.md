# Schemas

`v1/` contains the first JSON Schema 2020-12 contract draft and its registry.
Schemas use offline relative references even though each has a stable absolute
`$id`. The IDs use the reserved `.invalid` namespace and do not claim a domain;
consumers resolve them through the bundled registry. Objects are closed by
default, identity-relevant counters use canonical
decimal strings, and path/hash/version rules follow ADR-0005 and ADR-0009.

Run `ruby scripts/check-contracts.rb` for the dependency-free phase-one checks.
It validates JSON syntax, schema IDs/references/registry coverage, supported
schema keywords, and the fixture verdict manifest. It is intentionally a strict
project-subset checker, not a general JSON Schema implementation. A pinned full
Draft 2020-12 validator and independent cross-language conformance remain gates
before Rust serialization or a stable public contract.
