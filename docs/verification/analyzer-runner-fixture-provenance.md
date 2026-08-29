# Analyzer Runner Fixture Provenance Review

- Date: 2026-08-29
- Scope: ADR-0074 IAR-0 conformance fixtures
- Decision: approved for synthetic contract testing

Every IAR-0 fixture was authored specifically for this repository, is licensed
Apache-2.0, and is recorded with an exact SHA-256 in
`tests/conformance/v1/analyzer-runner-fixture-provenance.json`.

The set contains only JSON contract examples. It contains no executable
artifact, malware, live signature, third-party source, private/customer source,
credential, network capture, or provider response. Automated conformance tests
verify both every recorded digest and exact coverage of all IAR-0 fixtures.
