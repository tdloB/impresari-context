# IAR-0 Protocol And Synthetic Worker Verification

- Date: 2026-08-29
- Decision: ADR-0074
- Scope: authority-neutral protocol model only

## Delivered

- Six closed Draft 2020-12 schemas for capability, manifest, resource profile,
  request, result, and failure records.
- Fixed `iar-protocol-synthetic-v1` profile with a committed SHA-256 sidecar.
- Original-synthetic positive and negative fixtures with exact provenance.
- A pure Rust protocol crate with canonical identities, bounded one-message
  framing, exact in-memory artifact verification, complete no-op accounting,
  and deterministic simulated crash, timeout, input-mutation, output-flood, and
  malformed-output failures.

## Enforced invariants

- Requests contain no paths, commands, network destinations, credentials, or
  added authority.
- Repository-supplied manifests and mismatched pins fail closed.
- Results are all-or-nothing, account for every artifact, and contain zero
  findings in IAR-0.
- Failures are source-free and cannot claim partial coverage or authority.
- Truncated, trailing, oversized, malformed, and non-canonical messages fail
  before synthetic work.

## Explicit non-claims

IAR-0 performs no filesystem access, staging, subprocess launch, OS sandboxing,
scanner/parser execution, network access, upload, quarantine, or repository
execution. Passing IAR-0 does not demonstrate malware detection or analyzer
process isolation. Those claims require later increments and evidence.

## Reproduction

Run:

```sh
cargo test -p context-analyzer-protocol
cargo test -p context-conformance --test schema_conformance
ruby scripts/check-contracts.rb
```
