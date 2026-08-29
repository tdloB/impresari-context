# Hostile-Repository Contract Fixture Provenance Review

- Review date: 2026-08-29.
- Scope: ADR-0073 HRA-0 schema and resource-profile conformance only.
- Verdict: approved for contract conformance; not approved as detection,
  scanner-quality, malware, or runtime-execution evidence.
- Machine-readable manifest:
  [`tests/conformance/v1/hostile-repository-fixture-provenance.json`](../../tests/conformance/v1/hostile-repository-fixture-provenance.json).

## Review result

Every HRA-0 fixture is original, synthetic JSON authored for Impresari Context
and licensed under the repository's Apache-2.0 terms. No LeanCTX, Graft,
scanner-vendor, malware-corpus, third-party-repository, private, customer, or
user source was copied or adapted. The fixtures contain no executable artifact,
live malware, malware signature, credential, provider response, or uploadable
repository content.

The path `scripts/install.ps1` and all hashes are inert labels inside JSON. No
PowerShell bytes or parser input are present. The fixtures are consumed only by
offline schema validators. Their checked SHA-256 values are verified by the
Rust conformance suite so later replacement requires an explicit manifest
update and review.

## Permitted claims

The fixture set demonstrates only that:

- valid HRA-0 records satisfy the closed Draft 2020-12 schemas;
- false safety claims and ordinary-host execution authority are rejected;
- missing analyzer coverage can be represented without running an analyzer;
- the frozen profile denies mutation, processes, analyzers, networking,
  uploads, deep hostile-format parsing, and ordinary-host execution.

It does not demonstrate malware detection, false-positive or false-negative
rates, analyzer isolation, safe repository execution, or repository safety.
Those claims require separately approved later phases and independently
reviewed corpora.
