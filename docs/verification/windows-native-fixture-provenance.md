# Windows Native Feasibility Fixture Provenance Review

- Date: 2026-08-31
- Scope: ADR-0092 profile and capability-preflight conformance fixtures
- Decision: approved for synthetic contract testing

All three fixtures were authored specifically for this repository and are
licensed Apache-2.0. Exact SHA-256 identities are recorded in
`tests/conformance/v1/windows-native-fixture-provenance.json`.

The set contains JSON only. It contains no executable artifact, repository or
customer source, credential, SID, user path, provider response, network
capture, malware, or third-party content. The native Rust probe is reviewed
source, is compiled only on the fresh hosted Windows job, and is not committed
as an executable artifact.
