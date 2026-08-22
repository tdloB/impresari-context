# Slice D Controlled Extensibility Traceability

## Milestone D1 — Declarative contracts and output quarantine

Status: implemented and gated; no general extension runtime or transport is
authorized.

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Versioned operation contracts | Manifest kind covers parser, retriever, analyzer, exporter, and transport under exact contract `1.0.0` | Strict Serde and closed Draft 2020-12 schema |
| Integrity-pinned manifests | Exact SHA-256 artifact digest and explicit local pin policy | Valid/invalid manifest and exact-pin tests |
| Denied capabilities | Workspace, cache, process, network, environment, model, persistence, and artifact execution remain denied/unimplemented | Decision matrix and constant-false authority fields |
| Output normalization | Bounded closed envelopes must match manifest identity, version, kind, digest, and declared fields | Accepted-output tests with untrusted-derived classification |
| Quarantine | Invalid, oversized, unauthorized, spoofed, or authority-claiming output becomes metadata-only quarantine | Adversarial output tests; raw text absence assertion |
| Canonical integrity boundary | Extension output cannot write canonical stores or claim exact-source authority | No dependency from extension crate to stores/workspaces; authority mismatch tests |
| Frozen release evaluation | OS adapter output must equal direct neutral-core output; adversarial extension envelopes must quarantine without raw retention or authority | `consumer_extension_evaluation` in the mandatory evaluation gate |

## Explicitly Deferred Gates

- General extension artifact loading or execution.
- Filesystem, cache, process, environment, model, or network grants.
- Publisher signatures, trust roots, revocation, or automatic updates.
- MCP/HTTP or other remotely reachable transport.

Those items require their own ADR and the `SEC-F-002`/`SEC-F-003` analysis
before implementation. A declared extension kind is not runtime authorization.
