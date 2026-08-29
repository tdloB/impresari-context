# ADR-0073 HRA-0/HRA-1 Traceability

- Scope: static, evidence-only contracts and bounded read-only inventory.
- Authorization: founder-approved HRA-0 and separately approved HRA-1 on
  2026-08-29.
- Runtime status: HRA-1 inventory is implemented. HRA-2 observations, ADR-0074
  analyzers, and ADR-0075 quarantine execution remain unapproved and absent.

| Requirement | Authoritative artifact | Verification |
| --- | --- | --- |
| Artifact inventory | `security-artifact-inventory.schema.json` | Valid bounded Windows-oriented fixture; closed schema and fixed collection limits |
| Exact observed and untrusted-derived findings | `security-finding.schema.json` | Observed fixture requires exact evidence; derived classes require analyzer provenance |
| Canonical coverage independent of findings | `analyzer-coverage.schema.json` | Unavailable mandatory analyzer fixture remains explicit; completed state requires provenance and freshness |
| Immutable assessment without safety authority | `repository-security-assessment.schema.json` | Valid partial assessment plus rejected `safety_claimed: true` fixture |
| Deterministic policy data | `repository-admission-policy.schema.json` | Closed rule/effect fields and external-human exception owner |
| Four-state admission decision | `repository-admission-decision.schema.json` | Closed decision enum; rejected ordinary-host execution authority fixture |
| Fixed resource and authority boundary | `hostile-repository-resource-profile.schema.json` and `profiles/v1/hra-static-contract-v1.json` | Exact fixture equality and committed SHA-256 sidecar |
| Fixture provenance | `hostile-repository-fixture-provenance.json` | Per-file SHA-256 verification and all prohibited provenance flags false |
| Threat coverage | `docs/security/threat-model.md` SEC-T-042 through SEC-T-044 and SEC-REQ-017 | Repository security and policy checks |
| Evaluation gates | `docs/product/evaluation-prd.md` ADR-0073 HRA-0 hard gates | Full local and hosted conformance suites |
| Runtime inventory | `crates/context-admission/src/lib.rs` | Schema validation, exact snapshot/hash/length binding, deterministic cross-platform classification, explicit exclusions, source immutability, stale-snapshot failure, and symlink non-following unit tests |

## Authority audit

The profile fixes source mutation, process execution, analyzer execution,
network access, artifact upload, deep hostile-format parsing, and ordinary-host
execution to false. The assessment and decision schemas also fix safety claims,
ordinary-host execution authorization, and added authority to false. HRA-1 adds
only the `context-admission` library crate and reuses the existing guarded
read-only workspace capability. It adds no command, transport, provider,
credential, parser dependency, finding, policy decision, or execution
capability.
